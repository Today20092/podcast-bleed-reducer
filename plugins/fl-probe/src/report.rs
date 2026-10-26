use crate::probe::Observation;
use crate::shared_audio::ShareReport;
use crossbeam::queue::ArrayQueue;
use std::{
    fs::OpenOptions,
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const QUEUE_SIZE: usize = 4096;
const MAX_ROWS: u64 = 200_000;
static CLOCK: OnceLock<Instant> = OnceLock::new();

#[derive(Clone, Copy)]
pub struct Event {
    pub observation: Observation,
    pub callback: u64,
    pub elapsed_us: u64,
    pub offline: bool,
    pub timing_source: &'static str,
    pub activation_mode: &'static str,
    pub host_api: &'static str,
    pub manual_export: bool,
    pub processing_active: bool,
    pub share: ShareReport,
}

pub struct Reporter {
    queue: Arc<ArrayQueue<Event>>,
    stop: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
    worker: Option<JoinHandle<()>>,
    pub origin: Instant,
}

impl Reporter {
    pub fn start(id: u64) -> io::Result<Self> {
        let directory = std::env::var_os("LOCALAPPDATA")
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .ok_or_else(|| io::Error::other("LOCALAPPDATA must be an absolute directory"))?
            .join(if cfg!(cleanslate_podcast) {
                "CleanSlate/Podcast/Reports"
            } else {
                "CleanSlate/PluginProbe"
            });
        std::fs::create_dir_all(&directory)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("probe-{}-{stamp}-{id}.csv", std::process::id())))?;
        let queue = Arc::new(ArrayQueue::<Event>::new(QUEUE_SIZE));
        let stop = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicU64::new(0));
        let (events, stopping, lost) = (queue.clone(), stop.clone(), dropped.clone());
        let worker = thread::Builder::new().name("cleanslate-probe-report".into()).spawn(move || {
            let mut writer = BufWriter::new(file);
            let mut rows = 0_u64;
            let mut omitted = 0_u64;
            let mut failed = false;
            let header = format!("# {} {}; pid={}; instance={id}; PCM shared in memory only; no audio logged\ncallback,elapsed_us,group,member,start,frames,rate_hz,playing,offline,epoch,discontinuity,peer_start,peer_frames,status,timing_source,host_api,activation_mode,manual_export,processing_active,share_status,shared_start,paired_frames,audio_queue_dropped,peer_queue_depth\n",
                <crate::FlProbe as nice_plug::prelude::Plugin>::NAME, env!("CARGO_PKG_VERSION"), std::process::id());
            if writer.write_all(header.as_bytes()).is_err() { return; }
            loop {
                while let Some(event) = events.pop() {
                    if rows >= MAX_ROWS || failed { omitted += 1; continue; }
                    let o = event.observation;
                    failed = writeln!(writer, "{},{},{},{},{},{},{},{},{},{},{},{},{},{:?},{},{},{},{},{},{:?},{},{},{},{}",
                        event.callback, event.elapsed_us, o.group, o.member,
                        o.start.map_or_else(String::new, |s| s.to_string()), o.frames, f32::from_bits(o.rate),
                        o.playing, event.offline, o.epoch, o.discontinuity,
                        o.peer_start.map_or_else(String::new, |s| s.to_string()), o.peer_frames, o.status, event.timing_source,
                        event.host_api, event.activation_mode, event.manual_export, event.processing_active, event.share.status,
                        event.share.start.map_or_else(String::new, |s| s.to_string()), event.share.paired_frames,
                        event.share.queue_dropped, event.share.peer_depth).is_err();
                    rows += 1;
                }
                if writer.flush().is_err() { failed = true; }
                if stopping.load(Relaxed) && events.is_empty() { break; }
                thread::sleep(Duration::from_millis(25));
            }
            let _ = writeln!(writer, "# rows={rows}; queue_dropped={}; rows_omitted={omitted}; io_failed={failed}", lost.load(Relaxed));
            let _ = writer.flush();
        })?;
        Ok(Self {
            queue,
            stop,
            dropped,
            worker: Some(worker),
            origin: *CLOCK.get_or_init(Instant::now),
        })
    }

    pub fn push(&self, event: Event) {
        if self.queue.push(event).is_err() {
            self.dropped.fetch_add(1, Relaxed);
        }
    }
}

impl Drop for Reporter {
    fn drop(&mut self) {
        self.stop.store(true, Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_is_bounded_and_overflow_is_counted() {
        let reporter = Reporter {
            queue: Arc::new(ArrayQueue::new(1)),
            stop: Arc::new(AtomicBool::new(false)),
            dropped: Arc::new(AtomicU64::new(0)),
            worker: None,
            origin: Instant::now(),
        };
        let mut probe = crate::probe::Probe::new();
        let event = Event {
            observation: probe.observe(&crate::probe::GROUPS_STATE, 0, 1, None, 1, 48000, false),
            callback: 0,
            elapsed_us: 0,
            offline: false,
            timing_source: "missing",
            activation_mode: "Realtime",
            host_api: "CLAP",
            manual_export: false,
            processing_active: false,
            share: ShareReport::default(),
        };
        reporter.push(event);
        reporter.push(event);
        assert_eq!(reporter.queue.len(), 1);
        assert_eq!(reporter.dropped.load(Relaxed), 1);
    }
}
