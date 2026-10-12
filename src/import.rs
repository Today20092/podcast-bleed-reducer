//! Transient recording jobs. Provisional audio never enters a Project.
use crate::{
    AudioSource, RenderBlocker, SourceMetadata, SourceStamp, project::SourceIdentity,
    waveform::WaveformCache,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

pub(crate) enum Event {
    Metadata(Vec<WaveformCache>),
    Preview(Vec<WaveformCache>),
    Progress(&'static str, u64, u64),
    Ready(Vec<SourceIdentity>, Vec<WaveformCache>),
    Finished(Result<(), String>),
}

pub(crate) struct Job {
    pub id: u64,
    pub sources: Vec<AudioSource>,
    pub waveforms: Vec<WaveformCache>,
    pub status: String,
    pub error: Option<String>,
    pub accepted: bool,
    pub track_ids: Vec<crate::TrackId>,
    pub cancelled: Arc<AtomicBool>,
    pub events: Option<mpsc::Receiver<Event>>,
}

impl Drop for Job {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl Job {
    pub fn failed(id: u64, sources: Vec<AudioSource>, error: String) -> Self {
        let metadata = SourceMetadata {
            grid: crate::SourceTimebase::new(48_000, 0).expect("placeholder grid"),
            channels: 1,
            timing_origin: None,
            gapless_timing: false,
        };
        let waveforms = sources
            .iter()
            .map(|_| WaveformCache::skeleton(metadata))
            .collect();
        Self {
            id,
            sources,
            waveforms,
            status: "Failed · Retry or Remove".into(),
            error: Some(error),
            accepted: false,
            track_ids: vec![],
            cancelled: Arc::new(AtomicBool::new(false)),
            events: None,
        }
    }
    pub fn from_metadata(
        id: u64,
        sources: Vec<AudioSource>,
        metadata: Vec<SourceMetadata>,
    ) -> Self {
        let waveforms = metadata.into_iter().map(WaveformCache::skeleton).collect();
        Self {
            id,
            sources,
            waveforms,
            status: "Queued".into(),
            error: None,
            accepted: false,
            track_ids: vec![],
            cancelled: Arc::new(AtomicBool::new(false)),
            events: None,
        }
    }

    pub fn start(&mut self) {
        let (sender, events) = mpsc::sync_channel(2);
        self.events = Some(events);
        self.status = "Sampling whole-recording preview".into();
        let sources = self.sources.clone();
        let cancelled = self.cancelled.clone();
        std::thread::spawn(move || {
            let result = prepare(&sources, &cancelled, &sender).map_err(|e| e.to_string());
            let _ = sender.send(Event::Finished(result));
        });
    }

    pub fn retry(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        self.cancelled = Arc::new(AtomicBool::new(false));
        self.events = None;
        self.error = None;
        self.status = "Queued · rechecking source metadata".into();
    }
}

fn send(sender: &mpsc::SyncSender<Event>, event: Event) -> Result<(), RenderBlocker> {
    sender
        .send(event)
        .map_err(|_| RenderBlocker("Import cancelled".into()))
}

fn prepare(
    sources: &[AudioSource],
    cancelled: &AtomicBool,
    sender: &mpsc::SyncSender<Event>,
) -> Result<(), RenderBlocker> {
    let started = Instant::now();
    let stamps = sources
        .iter()
        .map(SourceStamp::read_source)
        .collect::<Result<Vec<_>, _>>()?;
    let metadata = sources
        .iter()
        .map(|s| s.metadata(Some(cancelled)))
        .collect::<Result<Vec<_>, _>>()?;
    crate::engine::validate_source_metadata(&metadata)?;
    send(
        sender,
        Event::Metadata(
            metadata
                .iter()
                .copied()
                .map(WaveformCache::skeleton)
                .collect(),
        ),
    )?;
    let mut overview = coarse_preview(sources, &metadata, cancelled)?;
    if let Some(overview) = &overview {
        send(sender, Event::Preview(overview.clone()))?;
    }
    let preview_ms = started.elapsed().as_millis();
    let mut last = Instant::now() - Duration::from_secs(1);
    let fingerprint =
        crate::project::fingerprint_progress(&sources[0].path, Some(cancelled), |done, total| {
            if last.elapsed() >= Duration::from_millis(100) || done == total {
                let _ = sender.try_send(Event::Progress("Checking source identity", done, total));
                last = Instant::now();
            }
        })?;
    let hash_ms = started.elapsed().as_millis();
    let fingerprints = sources
        .iter()
        .map(|s| s.fingerprint_from_container(&fingerprint))
        .collect::<Vec<_>>();
    let cached = fingerprints
        .iter()
        .zip(&metadata)
        .map(|(h, m)| crate::waveform::import_cache(h, *m))
        .collect::<Option<Vec<_>>>();
    if let Some(cached) = &cached {
        send(sender, Event::Preview(cached.clone()))?;
    }
    let mut peaks = vec![Vec::new(); sources.len()];
    let mut extrema = vec![(f32::INFINITY, f32::NEG_INFINITY); sources.len()];
    let mut used = vec![0; sources.len()];
    let mut counts = vec![0_u64; sources.len()];
    let mut last_refinement = Instant::now();
    let decoded =
        crate::audio_io::decode_recording_chunks(sources, Some(cancelled), |_, channels| {
            for (index, samples) in channels.iter().enumerate() {
                counts[index] += samples.len() as u64;
                if cached.is_none() {
                    for &sample in samples {
                        extrema[index].0 = extrema[index].0.min(sample);
                        extrema[index].1 = extrema[index].1.max(sample);
                        used[index] += 1;
                        if used[index] == 256 {
                            peaks[index].push(extrema[index]);
                            extrema[index] = (f32::INFINITY, f32::NEG_INFINITY);
                            used[index] = 0;
                        }
                    }
                }
            }
            if cached.is_none() && last_refinement.elapsed() >= Duration::from_millis(500) {
                if let Some(overview) = &mut overview {
                    for (waveform, peaks) in overview.iter_mut().zip(&peaks) {
                        waveform.refine_prefix(peaks);
                    }
                    let _ = sender.try_send(Event::Preview(overview.clone()));
                }
                last_refinement = Instant::now();
            }
            if last.elapsed() >= Duration::from_millis(100) {
                let done = counts.iter().sum();
                let total = metadata.iter().map(|m| m.grid.frame_count).sum();
                let _ = sender.try_send(Event::Progress(
                    if cached.is_some() {
                        "Validating audio · cached detail"
                    } else {
                        "Validating audio · refining detail"
                    },
                    done,
                    total,
                ));
                last = Instant::now();
            }
            Ok(())
        })?;
    if metadata.iter().any(|m| *m != decoded.metadata)
        || counts
            .iter()
            .zip(&metadata)
            .any(|(n, m)| *n != m.grid.frame_count)
    {
        return Err(RenderBlocker(
            "Decoded source disagrees with its native metadata".into(),
        ));
    }
    let waveforms = cached.unwrap_or_else(|| {
        peaks
            .into_iter()
            .enumerate()
            .map(|(index, mut peaks)| {
                if used[index] != 0 {
                    peaks.push(extrema[index]);
                }
                WaveformCache::from_peaks(
                    peaks,
                    counts[index] as usize,
                    metadata[index].grid.sample_rate,
                )
            })
            .collect()
    });
    let validation_ms = started.elapsed().as_millis();
    let current = sources
        .iter()
        .map(SourceStamp::read_source)
        .collect::<Result<Vec<_>, _>>()?;
    if current != stamps {
        return Err(RenderBlocker(
            "Source changed during import; Retry to inspect it again.".into(),
        ));
    }
    crate::prepared_analysis::check_cancelled(Some(cancelled))?;
    let identities = sources
        .iter()
        .cloned()
        .zip(&metadata)
        .zip(&fingerprints)
        .zip(stamps)
        .map(|(((s, m), h), stamp)| SourceIdentity::prepared(s, *m, h.clone(), stamp))
        .collect();
    send(sender, Event::Ready(identities, waveforms.clone()))?;
    for ((fingerprint, metadata), waveform) in
        fingerprints.into_iter().zip(metadata).zip(&waveforms)
    {
        crate::prepared_analysis::check_cancelled(Some(cancelled))?;
        crate::waveform::save_import_cache(fingerprint, metadata, waveform);
    }
    eprintln!(
        "IMPORT_STAGES preview_ms={preview_ms} hash_ms={} validation_ms={} total_ms={} container_hashes=1 selected_channels={}",
        hash_ms - preview_ms,
        validation_ms - hash_ms,
        started.elapsed().as_millis(),
        sources.len()
    );
    Ok(())
}

fn coarse_preview(
    sources: &[AudioSource],
    metadata: &[SourceMetadata],
    cancelled: &AtomicBool,
) -> Result<Option<Vec<WaveformCache>>, RenderBlocker> {
    let Ok(mut reader) = bwavfile::WaveReader::open(&sources[0].path) else {
        return Ok(None);
    };
    let format = reader.format()?;
    // bwavfile cannot read float64; its safe media decoder supplies full detail below.
    if format.bits_per_sample == 64 {
        return Ok(None);
    }
    let count = metadata[0].grid.frame_count as usize;
    let bucket = count.div_ceil(2048).max(1);
    let channels = usize::from(format.channel_count);
    if channels == 0
        || sources
            .iter()
            .any(|s| usize::from(s.channel.unwrap_or(0)) >= channels)
    {
        return Err(RenderBlocker("Selected channel is out of range".into()));
    }
    let mut frames = reader.audio_frame_reader()?;
    let mut buffer = vec![0.0_f32; 128 * channels];
    let mut peaks = vec![Vec::new(); sources.len()];
    for start in (0..count).step_by(bucket) {
        crate::prepared_analysis::check_cancelled(Some(cancelled))?;
        frames.locate(start as u64)?;
        let length = 128.min(count - start);
        let read = frames.read_frames(&mut buffer[..length * channels])? as usize;
        if read != length {
            return Err(RenderBlocker("Truncated audio in preview".into()));
        }
        for (index, source) in sources.iter().enumerate() {
            let mut peak = (f32::INFINITY, f32::NEG_INFINITY);
            for frame in buffer[..read * channels].chunks_exact(channels) {
                let value = frame[usize::from(source.channel.unwrap_or(0))];
                if !value.is_finite() {
                    return Err(RenderBlocker("Source contains invalid samples".into()));
                }
                peak = (peak.0.min(value), peak.1.max(value));
            }
            peaks[index].push(peak);
        }
    }
    Ok(Some(
        peaks
            .into_iter()
            .zip(metadata)
            .map(|(p, m)| WaveformCache::coarse(p, *m, bucket))
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(sources: Vec<AudioSource>) -> (Vec<Event>, Result<(), String>) {
        let (sender, receiver) = mpsc::sync_channel(2);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker = std::thread::spawn(move || {
            prepare(&sources, &cancelled, &sender).map_err(|e| e.to_string())
        });
        let events = receiver.iter().collect();
        (events, worker.join().unwrap())
    }

    #[test]
    fn whole_recording_preview_precedes_shared_validation_and_cache_reuse() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("shared.wav");
        crate::audio_io::multichannel_tests::write_poly(&path, 3, 32, 30_001);
        let sources = crate::wav_channel_sources(&path).unwrap();
        for _ in 0..2 {
            let (events, result) = run(sources.clone());
            result.unwrap();
            let preview = events
                .iter()
                .position(|e| matches!(e, Event::Preview(_)))
                .unwrap();
            let ready = events
                .iter()
                .position(|e| matches!(e, Event::Ready(_, _)))
                .unwrap();
            assert!(preview < ready);
            if let Event::Preview(waveforms) = &events[preview] {
                assert_eq!(waveforms.len(), 3);
                assert!(waveforms.iter().all(|w| w.sample_count == 30_001));
            }
            if let Event::Ready(identities, waveforms) = &events[ready] {
                assert_eq!(identities.len(), 3);
                for (source, identity) in sources.iter().zip(identities) {
                    assert_eq!(source.fingerprint(None).unwrap(), identity.fingerprint);
                    assert_eq!(identity.metadata, source.metadata(None).unwrap());
                }
                assert!(waveforms.iter().all(|w| w.sample_count == 30_001));
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_and_truncated_recordings_never_emit_ready() {
        let root = crate::audio_io::multichannel_tests::root();
        let bad = root.join("bad.wav");
        std::fs::write(&bad, b"not a supported recording").unwrap();
        let (events, result) = run(vec![AudioSource::mono(bad)]);
        assert!(result.is_err());
        assert!(!events.iter().any(|e| matches!(e, Event::Ready(_, _))));
        let truncated = root.join("truncated.wav");
        crate::audio_io::multichannel_tests::write_poly(&truncated, 3, 32, 30_001);
        let inputs = crate::wav_channel_sources(&truncated).unwrap();
        let length = std::fs::metadata(&truncated).unwrap().len();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&truncated)
            .unwrap()
            .set_len(length - 31)
            .unwrap();
        let (events, result) = run(inputs);
        assert!(result.is_err());
        assert!(!events.iter().any(|e| matches!(e, Event::Ready(_, _))));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancellation_and_dropped_receiver_release_worker_without_ready() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("cancel.wav");
        crate::audio_io::multichannel_tests::write_poly(&path, 3, 32, 30_001);
        let sources = crate::wav_channel_sources(&path).unwrap();
        for cancel in [true, false] {
            let (sender, receiver) = mpsc::sync_channel(1);
            let flag = Arc::new(AtomicBool::new(false));
            let worker_flag = flag.clone();
            let inputs = sources.clone();
            let worker = std::thread::spawn(move || prepare(&inputs, &worker_flag, &sender));
            receiver.recv_timeout(Duration::from_secs(5)).unwrap();
            if cancel {
                flag.store(true, Ordering::Relaxed);
            }
            drop(receiver);
            assert!(worker.join().unwrap().is_err());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
