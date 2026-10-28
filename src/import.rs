//! Transient recording jobs. Provisional audio never enters a Project.
use crate::{
    AudioSource, RenderBlocker, SourceMetadata, SourceStamp, project::SourceIdentity,
    waveform::WaveformCache,
};
use std::{
    io::{Read, Seek, SeekFrom},
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
        crate::worker_lifetime::spawn(move || {
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
    let mut builders: Vec<_> = sources
        .iter()
        .map(|_| crate::waveform::WaveformBuilder::default())
        .collect();
    let mut counts = vec![0_u64; sources.len()];
    let mut last_refinement = Instant::now();
    let decoded =
        crate::audio_io::decode_recording_chunks(sources, Some(cancelled), |_, channels| {
            for (index, samples) in channels.iter().enumerate() {
                counts[index] += samples.len() as u64;
                if cached.is_none() {
                    builders[index].push(samples);
                }
            }
            if cached.is_none() && last_refinement.elapsed() >= Duration::from_millis(500) {
                if let Some(overview) = &mut overview {
                    for (waveform, builder) in overview.iter_mut().zip(&builders) {
                        builder.refine_preview(waveform);
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
        builders
            .into_iter()
            .zip(&metadata)
            .map(|(builder, metadata)| builder.finish(metadata.grid.sample_rate))
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
    let mut reader = match crate::audio_io::open_wave(&sources[0].path) {
        Ok(reader) => reader,
        Err(bwavfile::Error::IOError(error)) => return Err(error.into()),
        Err(_) => return Ok(None),
    };
    let format = reader.format()?;
    let channels = usize::from(format.channel_count);
    if channels == 0
        || sources
            .iter()
            .any(|s| usize::from(s.channel.unwrap_or(0)) >= channels)
    {
        return Err(RenderBlocker("Selected channel is out of range".into()));
    }
    if format.bits_per_sample == 64 {
        // bwavfile's float64 frame reader panics; preview raw IEEE windows only.
        if usize::from(format.block_alignment) != channels * 8 {
            return Err(RenderBlocker("Invalid float64 frame alignment".into()));
        }
        let file_length = reader.inner.seek(SeekFrom::End(0))?;
        reader.inner.seek(SeekFrom::Start(12))?;
        let data_start = loop {
            crate::prepared_analysis::check_cancelled(Some(cancelled))?;
            let mut chunk = [0_u8; 8];
            reader.inner.read_exact(&mut chunk)?;
            let start = reader.inner.stream_position()?;
            if &chunk[..4] == b"data" {
                break start;
            }
            let length = u64::from(u32::from_le_bytes(chunk[4..].try_into().unwrap()));
            let next = start
                .checked_add(length + length % 2)
                .filter(|next| *next <= file_length)
                .ok_or_else(|| RenderBlocker("Truncated WAV preview metadata".into()))?;
            reader.inner.seek(SeekFrom::Start(next))?;
        };
        let mut raw = vec![0_u8; 128 * channels * 8];
        return sampled_preview(
            sources,
            metadata,
            channels,
            cancelled,
            |start, length, buffer| {
                let offset = (start as u64)
                    .checked_mul(u64::from(format.block_alignment))
                    .and_then(|offset| data_start.checked_add(offset))
                    .ok_or_else(|| RenderBlocker("WAV preview offset overflow".into()))?;
                reader.inner.seek(SeekFrom::Start(offset))?;
                let bytes = &mut raw[..length * channels * 8];
                reader.inner.read_exact(bytes)?;
                for (sample, bytes) in buffer.iter_mut().zip(bytes.as_chunks::<8>().0) {
                    *sample = f64::from_le_bytes(*bytes) as f32;
                }
                Ok(length)
            },
        )
        .map(Some);
    }
    let mut frames = reader.audio_frame_reader()?;
    sampled_preview(
        sources,
        metadata,
        channels,
        cancelled,
        |start, _, buffer| {
            frames.locate(start as u64)?;
            Ok(frames.read_frames(buffer)? as usize)
        },
    )
    .map(Some)
}

fn sampled_preview(
    sources: &[AudioSource],
    metadata: &[SourceMetadata],
    channels: usize,
    cancelled: &AtomicBool,
    mut read: impl FnMut(usize, usize, &mut [f32]) -> Result<usize, RenderBlocker>,
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let count = metadata[0].grid.frame_count as usize;
    let bucket = count.div_ceil(2048).max(1);
    let mut buffer = vec![0.0_f32; 128 * channels];
    let mut peaks = vec![Vec::new(); sources.len()];
    for start in (0..count).step_by(bucket) {
        crate::prepared_analysis::check_cancelled(Some(cancelled))?;
        let length = 128.min(count - start);
        let read = read(start, length, &mut buffer[..length * channels])?;
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
    Ok(peaks
        .into_iter()
        .zip(metadata)
        .map(|(p, m)| WaveformCache::coarse(p, *m, bucket))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn import_worker_start_failure_is_actionable_and_retryable() {
        let sources = vec![AudioSource::mono("source-kept.wav".into())];
        let mut job = Job::failed(1, sources.clone(), "previous failure".into());
        job.retry();
        crate::worker_lifetime::fail_next_spawn();
        job.start();
        assert!(job.error.as_deref().is_some_and(|error| error.contains("start import")));
        assert!(job.events.is_none(), "failed launch cannot leave a busy receiver");
        assert_eq!(job.sources, sources);
        job.retry();
        assert!(job.error.is_none());
        assert!(job.events.is_none());
        assert!(job.status.starts_with("Queued"));
    }

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
                let reopened = crate::waveform::load_source_waveforms(&sources, |_, _| {}).unwrap();
                assert_eq!(
                    serde_json::to_value(waveforms).unwrap(),
                    serde_json::to_value(reopened).unwrap()
                );
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

    #[test]
    fn source_changed_after_preview_cannot_be_accepted() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("changed.wav");
        crate::audio_io::multichannel_tests::write_poly(&path, 3, 32, 30_001);
        let sources = crate::wav_channel_sources(&path).unwrap();
        let (sender, receiver) = mpsc::sync_channel(0);
        let worker =
            std::thread::spawn(move || prepare(&sources, &AtomicBool::new(false), &sender));
        assert!(matches!(receiver.recv().unwrap(), Event::Metadata(_)));
        // The rendezvous preview keeps the worker before hashing/validation.
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"changed")
            .unwrap();
        assert!(matches!(receiver.recv().unwrap(), Event::Preview(_)));
        assert!(
            !receiver
                .iter()
                .any(|event| matches!(event, Event::Ready(_, _)))
        );
        assert!(worker.join().unwrap().is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn float64_recording_publishes_preview_before_validation() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("float64.wav");
        crate::audio_io::multichannel_tests::write_poly(&path, 3, 64, 30_001);
        let (events, result) = run(crate::wav_channel_sources(&path).unwrap());
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
            assert!(
                waveforms
                    .iter()
                    .all(|waveform| waveform.sample_count == 30_001)
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selected_float32_chunks_match_native_frame_decoder_across_partial_blocks() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("float32-oracle.wav");
        crate::audio_io::multichannel_tests::write_poly(&path, 3, 32, 30_001);
        let sources = crate::wav_channel_sources(&path).unwrap();
        let sources = vec![sources[2].clone(), sources[0].clone()];
        let reader = bwavfile::WaveReader::open(&path).unwrap();
        let mut expected = vec![0.0_f32; 30_001 * 3];
        reader
            .audio_frame_reader()
            .unwrap()
            .read_frames(&mut expected)
            .unwrap();
        let mut offset = 0;
        let decoded = crate::audio_io::decode_recording_chunks(&sources, None, |rate, channels| {
            assert_eq!(rate, 48_000);
            for (samples, channel) in channels.iter().zip([2, 0]) {
                assert!(
                    samples
                        .iter()
                        .enumerate()
                        .all(|(index, sample)| *sample == expected[(offset + index) * 3 + channel])
                );
            }
            offset += channels[0].len();
            Ok(())
        })
        .unwrap();
        assert_eq!(offset, 30_001);
        assert_eq!(decoded.metadata, sources[0].metadata(None).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }
}
