//! Session regressions use isolated cache roots and real PCM files, without a DAW.
use super::*;
use crate::FlProbe;
use rust_app::GainEnvelope;
use std::time::Instant;

const RATE: u32 = 8000;

#[test]
fn analysis_gain_view_clips_exact_bounds_and_maps_only_visible_members() {
    let fixture = Fixture::new(3);
    let engine = fixture.engine();
    direct_capture(engine, 2, 0, 4096, false);
    let mut member_map = [None; MEMBERS];
    member_map[2] = Some(0);
    *engine.analysis.write() = Some(Arc::new(Published {
        revision: engine.revision.load(SeqCst),
        analysis: Analysis {
            origin: 500,
            end: 2548,
            rate: RATE,
            envelopes: vec![GainEnvelope::constant_at_rate(2048, RATE, 0.25).unwrap()],
        },
        hashes: vec![Vec::new(); MEMBERS],
        member_map,
    }));
    let view = engine.analysis_view(0, 4096, 4, 0b101).unwrap();
    assert!(engine.analysis_view_is_current(&view));
    assert_eq!(view.tracks.len(), 1);
    assert_eq!(view.tracks[0].member, 2);
    assert_eq!(
        view.tracks[0].columns,
        vec![
            GainColumn {
                start: 500,
                end: 1024,
                detail: GainDetail::Bounds(0.25, 0.25)
            },
            GainColumn {
                start: 1024,
                end: 2048,
                detail: GainDetail::Bounds(0.25, 0.25)
            },
            GainColumn {
                start: 2048,
                end: 2548,
                detail: GainDetail::Bounds(0.25, 0.25)
            },
        ],
    );
    assert!(
        engine
            .analysis_view(0, 4096, 4, 0b001)
            .unwrap()
            .tracks
            .is_empty()
    );
    let tiny = engine.analysis_view(500, 503, 2048, 0b100).unwrap();
    assert_eq!(tiny.tracks[0].columns.len(), 3);
    assert_eq!(tiny.tracks[0].columns[0].start, 500);
    assert_eq!(tiny.tracks[0].columns[2].end, 503);
}

#[test]
fn analysis_gain_view_refuses_dense_detail_and_zoom_recovers_exact_extrema() {
    let fixture = Fixture::new(1);
    let engine = fixture.engine();
    let envelope = display_test_envelope(&fixture);
    publish_display_analysis(engine, 0, 0, envelope);
    let dense = engine.analysis_view(0, 8192, 1, 1).unwrap();
    assert_eq!(
        dense.tracks[0].columns,
        vec![GainColumn {
            start: 0,
            end: 8192,
            detail: GainDetail::TooDense,
        }]
    );
    let detailed = engine.analysis_view(0, 8192, 32, 1).unwrap();
    assert_eq!(detailed.tracks[0].columns.len(), 32);
    // Independent linear ramps: 0.25 -> 0.75 in 256 samples, with the
    // right endpoint belonging to the next half-open column.
    assert_eq!(
        detailed.tracks[0].columns[0].detail,
        GainDetail::Bounds(0.25, 0.7480469)
    );
    assert_eq!(
        detailed.tracks[0].columns[1].detail,
        GainDetail::Bounds(0.25195313, 0.75)
    );
}

pub(crate) fn display_test_envelope(fixture: &Fixture) -> GainEnvelope {
    use rust_app::{
        EnvelopePoint, ManualGainEdit, SourceBinding, SourceTimebase, TemporalEvidence,
        TemporalEvidenceGrid, TemporalOwnershipCache, TemporalSettings,
    };
    let engine = fixture.engine();
    direct_capture(engine, 0, 0, 8192, false);
    let path = engine.path.join("gain-display.wav");
    snapshot_wav(
        &mut File::open(engine.path.join("mic-0.pcm")).unwrap(),
        &path,
        0,
        8192,
        RATE,
        1,
        None,
    )
    .unwrap();
    let mut project =
        rust_app::Project::create(engine.path.join("gain-display.cleanslate"), vec![path]).unwrap();
    project.set_supplied_alignment(true).unwrap();
    let source = &project.sources[0];
    let source_grid = SourceTimebase::new(RATE, 8192).unwrap();
    let duration = source_grid.duration().unwrap();
    let frames = duration.div_ceil(rust_app::OWNERSHIP_FRAME_TICKS) as usize;
    let evidence = TemporalEvidenceGrid::new(
        vec![source.track_id],
        duration,
        vec![vec![
            TemporalEvidence {
                probability: Some(0.01),
                raw_speech: Some(0.01),
                assigned_activity: Some(0.01),
                preservation_reason: None
            };
            frames
        ]],
        vec![0.01; frames],
    )
    .unwrap();
    let ownership = TemporalOwnershipCache::default()
        .optimize_project_evidence(&project, &evidence, TemporalSettings::default(), None)
        .unwrap();
    let edits = vec![ManualGainEdit {
        id: 1,
        source: SourceBinding {
            track: source.track_id,
            fingerprint: source.fingerprint.clone(),
        },
        points: (0..=32)
            .map(|index| EnvelopePoint {
                time: source_grid
                    .time_at_frame(index * 256, rust_app::Rounding::Floor)
                    .unwrap(),
                gain: if index % 2 == 0 { 0.25 } else { 0.75 },
            })
            .collect(),
        interpolation: Default::default(),
    }];
    project.replace_manual_gain_edits(edits).unwrap();
    let effective = rust_app::EffectiveEnvelopeCache::default()
        .update(
            &project,
            &ownership,
            rust_app::GainFadeSettings {
                attenuation: 0.5,
                fade_in: 0,
                fade_out: 0,
                minimum_gap: 0,
            },
            None,
        )
        .unwrap();
    effective.envelopes[0].clone()
}

#[test]
fn analysis_gain_view_omits_unverified_columns_and_rejects_stale_capture_identity() {
    let mut fixture = Fixture::new(1);
    direct_capture(fixture.engine(), 0, 0, 4096, false);
    publish_display_analysis(
        fixture.engine(),
        0,
        0,
        GainEnvelope::constant_at_rate(4096, RATE, 0.25).unwrap(),
    );
    fixture.engine().state.lock().tracks[0]
        .verified
        .remove(1024, 2048);
    let view = fixture.engine().analysis_view(0, 4096, 4, 1).unwrap();
    assert_eq!(
        view.tracks[0]
            .columns
            .iter()
            .map(|column| (column.start, column.end))
            .collect::<Vec<_>>(),
        vec![(0, 1024), (2048, 3072), (3072, 4096)]
    );
    fixture.with_worker_lock(|| {
        assert!(fixture.engine().analysis_view(0, 4096, 4, 1).is_none());
    });
    let other = Fixture::new(1);
    assert!(!other.engine().analysis_view_is_current(&view));
    fixture.reopen();
    assert!(!fixture.engine().analysis_view_is_current(&view));
    assert!(fixture.engine().analysis_view(0, 4096, 4, 1).is_none());
    direct_capture(fixture.engine(), 0, 0, 4096, false);
    publish_display_analysis(
        fixture.engine(),
        0,
        0,
        GainEnvelope::constant_at_rate(4096, RATE, 0.25).unwrap(),
    );
    let current = fixture.engine().analysis_view(0, 4096, 4, 1).unwrap();
    fixture
        .engine()
        .set_settings(Settings {
            reduction_db: -18.0,
            ..Default::default()
        })
        .unwrap();
    assert!(!fixture.engine().analysis_view_is_current(&current));
    assert!(fixture.engine().analysis_view(0, 4096, 4, 1).is_none());
}

pub(crate) fn publish_display_analysis(
    engine: &SessionCapture,
    member: usize,
    origin: i64,
    envelope: GainEnvelope,
) {
    let mut member_map = [None; MEMBERS];
    member_map[member] = Some(0);
    *engine.analysis.write() = Some(Arc::new(Published {
        revision: engine.revision.load(SeqCst),
        analysis: Analysis {
            origin,
            end: origin + envelope.sample_count() as i64,
            rate: engine.rate,
            envelopes: vec![envelope],
        },
        hashes: vec![Vec::new(); MEMBERS],
        member_map,
    }));
}

#[test]
fn cancel_request_returns_while_capture_storage_is_locked() {
    let fixture = Fixture::new(1);
    let (send, receive) = std::sync::mpsc::channel();
    let engine = fixture.engine().clone();
    let mut completed = false;
    let mut request = None;
    fixture.with_worker_lock(|| {
        request = Some(thread::spawn(move || {
            engine.cancel_analysis();
            send.send(()).unwrap();
        }));
        // A liveness timeout, not a claimed UI latency measurement: the lock stays held.
        completed = receive.recv_timeout(Duration::from_secs(5)).is_ok();
    });
    request.unwrap().join().unwrap();
    assert!(
        completed,
        "Cancel must not wait for capture/snapshot I/O to release its state lock"
    );
}

#[test]
fn conflicting_examples_require_explicit_replacement() {
    let fixture = Fixture::new(2);
    let engine = fixture.engine();
    for member in 0..2 {
        direct_capture(engine, member, 0, 8000, false);
    }
    engine.label(0, 4000, Some(1)).unwrap();
    assert!(engine.label(2000, 6000, Some(2)).is_err());
    assert_eq!(
        engine.view(None, None, 32).unwrap().labels,
        vec![LabelRange {
            start: 0,
            end: 4000,
            mask: 1
        }]
    );
    engine.replace_label(2000, 6000, 2).unwrap();
    assert_eq!(
        engine.view(None, None, 32).unwrap().labels,
        vec![
            LabelRange {
                start: 0,
                end: 2000,
                mask: 1
            },
            LabelRange {
                start: 2000,
                end: 6000,
                mask: 2
            },
        ]
    );
    assert!(engine.replace_label(4000, 9000, 1).is_err());
    assert!(engine.replace_label(2000, 6000, 4).is_err());
    engine.label(2000, 6000, None).unwrap();
    assert_eq!(engine.view(None, None, 32).unwrap().labels.len(), 1);
}

#[test]
fn cancelling_explicit_analysis_retains_examples_and_source_capture() {
    let fixture = Fixture::new(1);
    let engine = fixture.engine();
    direct_capture(engine, 0, 0, 8000, false);
    engine.label(0, 4000, Some(0)).unwrap();
    assert!(!engine.view(None, None, 32).unwrap().busy);
    engine.analyze(Settings::default()).unwrap();
    engine.cancel_analysis();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let view = engine.view(None, None, 32);
        if let Some(view) = view
            && !view.busy
        {
            assert!(!view.ready);
            assert_eq!(view.labels.len(), 1);
            assert!(view.tracks[0].verified);
            assert!(view.message.to_lowercase().contains("cancelled"));
            break;
        }
        assert!(Instant::now() < deadline, "cancelled analysis must drain");
        thread::sleep(Duration::from_millis(5));
    }
    engine.analyze(Settings::default()).unwrap();
    engine
        .set_settings(Settings {
            reduction_db: -18.0,
            ..Settings::default()
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(view) = engine.view(None, None, 32)
            && !view.busy
        {
            assert!(!view.ready);
            assert!(
                view.message.to_lowercase().contains("changed"),
                "{}",
                view.message
            );
            assert!(!view.message.to_lowercase().contains("cancelled"));
            break;
        }
        assert!(Instant::now() < deadline, "invalidated analysis must drain");
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn waveform_long_history_redraw_measurement() {
    let fixture = Fixture::new(8);
    {
        let mut state = fixture.engine().state.lock();
        for track in &mut state.tracks {
            track.coverage.insert(0, i64::from(RATE) * 7200).unwrap();
            track.peaks = vec![
                Peak {
                    min: -0.5,
                    max: 0.5,
                    valid: true
                };
                225000
            ];
            track.rebuild_peak_levels();
        }
    }
    for (name, end) in [
        ("two-hours", i64::from(RATE) * 7200),
        ("ten-seconds", i64::from(RATE) * 10),
    ] {
        let start = Instant::now();
        for _ in 0..40 {
            let view = fixture.engine().view(Some(0), Some(end), 1000).unwrap();
            assert_eq!(view.tracks.len(), 8);
            std::hint::black_box(view);
        }
        eprintln!(
            "waveform-view {name}: {:.3} ms/frame (8 tracks, 1000 columns, 40 frames)",
            start.elapsed().as_secs_f64() * 25.0
        );
    }
}

#[test]
fn peak_summaries_match_exact_ranges_after_sparse_edits_and_rebuild() {
    let mut track = Track::empty();
    for index in [0, 1, 7, 8, 63, 64, 511, 512, 8191, 9000, 7, 511] {
        track.set_peak(
            index,
            Peak {
                min: -(index as f32 + 1.0),
                max: index as f32 / 2.0,
                valid: true,
            },
        );
    }
    for rebuild in [false, true] {
        if rebuild {
            track.rebuild_peak_levels();
        }
        for start in (0..9200).step_by(37) {
            for length in [1, 7, 8, 33, 64, 225, 512, 9000] {
                let end = (start + length).min(track.peaks.len());
                let expected = track
                    .peaks
                    .get(start..end)
                    .unwrap_or(&[])
                    .iter()
                    .copied()
                    .fold(Peak::default(), Peak::merge);
                let actual = track.peak_range(start, start + length);
                assert_eq!(
                    (actual.valid, actual.min, actual.max),
                    (expected.valid, expected.min, expected.max),
                    "{start}..{} rebuild={rebuild}",
                    start + length
                );
            }
        }
    }
}

pub(crate) struct Fixture {
    engine: Option<Arc<SessionCapture>>,
    root: PathBuf,
    key: String,
    rate: u32,
    params: Vec<Arc<ProbeParams>>,
}

impl Fixture {
    pub(crate) fn new(members: usize) -> Self {
        Self::new_with_rate(members, RATE)
    }

    fn new_with_rate(members: usize, rate: u32) -> Self {
        let key = new_key();
        let root = std::env::temp_dir().join(format!("cleanslate-session-regression-{key}"));
        let engine = SessionCapture::create(
            key.clone(),
            rate,
            root.clone(),
            Vec::new(),
            Settings::default(),
            Vec::new(),
        )
        .unwrap();
        let params = (0..members)
            .map(|_| FlProbe::default().params.clone())
            .collect::<Vec<_>>();
        let fixture = Self {
            engine: Some(engine),
            root,
            key,
            rate,
            params,
        };
        fixture.register_members();
        fixture
    }

    pub(crate) fn engine(&self) -> &Arc<SessionCapture> {
        self.engine.as_ref().unwrap()
    }

    pub(crate) fn with_worker_lock(&self, test: impl FnOnce()) {
        let _busy = self.engine().state.lock();
        test();
    }

    pub(crate) fn unverify_display_range(&self, member: usize, start: i64, end: i64) {
        self.engine().state.lock().tracks[member]
            .verified
            .remove(start, end);
    }

    fn register_members(&self) {
        for (member, params) in self.params.iter().enumerate() {
            self.engine().owners[member].store(member as u64 + 1, SeqCst);
            self.engine().members.lock()[member] = Arc::downgrade(params);
        }
    }

    fn close(&mut self) {
        if let Some(engine) = self.engine.take() {
            let weak = Arc::downgrade(&engine);
            drop(engine);
            let deadline = Instant::now() + Duration::from_secs(3);
            while weak.strong_count() != 0 && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            assert_eq!(weak.strong_count(), 0, "capture worker retained the cache");
        }
    }

    fn reopen(&mut self) {
        self.engine().save().unwrap();
        let (labels, settings, evidence) = {
            let state = self.engine().state.lock();
            (state.labels.clone(), state.settings, state.evidence.clone())
        };
        self.close();
        self.engine = Some(
            SessionCapture::create(
                self.key.clone(),
                self.rate,
                self.root.clone(),
                labels,
                settings,
                evidence,
            )
            .unwrap(),
        );
        self.register_members();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.close();
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn sample(position: i64, reversed: bool) -> f32 {
    let positive = position.rem_euclid(2) == 0;
    if positive ^ reversed { 0.5 } else { -0.5 }
}

pub(crate) fn direct_capture(
    engine: &SessionCapture,
    member: usize,
    start: i64,
    end: i64,
    reversed: bool,
) {
    let mut position = start;
    while position < end {
        let mut packet = Packet {
            member,
            owner: member as u64 + 1,
            start: position,
            count: ((end - position) as usize).min(BLOCK),
            channels: 1,
            samples: [[0.0; 2]; BLOCK],
        };
        for offset in 0..packet.count {
            packet.samples[offset][0] = sample(position + offset as i64, reversed);
        }
        engine.write_packet(packet).unwrap();
        position += packet.count as i64;
    }
}

// Shared deterministic visual/performance fixture: room, distinct speech-like bursts,
// low background and sharp edges. Generated PCM is not acoustic-quality evidence.
pub(crate) fn visual_capture(engine: &SessionCapture, member: usize) {
    for start in (0..64000).step_by(BLOCK) {
        let mut packet = Packet {
            member,
            owner: member as u64 + 1,
            start: start as i64,
            count: (64000 - start).min(BLOCK),
            channels: 1,
            samples: [[0.0; 2]; BLOCK],
        };
        for offset in 0..packet.count {
            let index = start + offset;
            let t = index as f32 / 8000.0;
            let phase = (t * 1.7 + member as f32 * 0.43).fract();
            let burst = if index >= 8000 && phase < 0.58 {
                (phase * std::f32::consts::PI / 0.58).sin().powi(2)
            } else {
                0.0
            };
            let carrier = (t * (173.0 + member as f32 * 31.0) * std::f32::consts::TAU).sin();
            packet.samples[offset][0] = carrier * (0.012 + burst * (0.28 + member as f32 * 0.035));
        }
        engine.write_packet(packet).unwrap();
    }
}

fn install_constant_analysis(engine: &SessionCapture, samples: &[f32]) {
    assert_eq!(samples.len(), BLOCK);
    let mut bytes = Vec::with_capacity(BLOCK * 4);
    for value in samples {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    let mut hashes = vec![Vec::new(); MEMBERS];
    hashes[0].push(Sha256::digest(&bytes).into());
    let mut member_map = [None; MEMBERS];
    member_map[0] = Some(0);
    *engine.analysis.write() = Some(Arc::new(Published {
        revision: engine.revision.load(SeqCst),
        analysis: Analysis {
            origin: 0,
            end: BLOCK as i64,
            rate: RATE,
            envelopes: vec![GainEnvelope::constant_at_rate(BLOCK, RATE, 0.25).unwrap()],
        },
        hashes,
        member_map,
    }));
}

#[test]
fn loops_and_seeks_fill_source_time_without_erasing_or_churning_unchanged_audio() {
    let fixture = Fixture::new(1);
    let engine = fixture.engine();
    let seconds = i64::from(RATE);
    direct_capture(engine, 0, 10 * seconds, 30 * seconds, false);
    assert!(
        engine.state.lock().tracks[0]
            .coverage
            .covers(10 * seconds, 30 * seconds)
    );
    let revision = engine.revision.load(SeqCst);

    direct_capture(engine, 0, 0, 30 * seconds, false);
    direct_capture(engine, 0, 10 * seconds, 30 * seconds, false);

    let state = engine.state.lock();
    assert!(state.tracks[0].coverage.covers(0, 30 * seconds));
    assert!(state.tracks[0].verified.covers(0, 30 * seconds));
    assert_eq!(state.tracks[0].coverage.0.len(), 1);
    assert_eq!(engine.revision.load(SeqCst), revision);
    assert!(state.tracks[0].peaks.iter().filter(|p| p.valid).count() > 900);
    assert_eq!(
        fs::metadata(engine.path.join("mic-0.pcm")).unwrap().len(),
        30 * u64::from(RATE) * 4
    );
}

#[test]
fn pcm_changes_with_identical_waveform_extrema_invalidate_analysis_and_examples() {
    let fixture = Fixture::new(1);
    let engine = fixture.engine();
    direct_capture(engine, 0, 0, (BLOCK * 2) as i64, false);
    engine.label(0, (BLOCK * 2) as i64, Some(1)).unwrap();
    let before = engine.state.lock().tracks[0]
        .peaks
        .iter()
        .map(|peak| (peak.min, peak.max, peak.valid))
        .collect::<Vec<_>>();
    let revision = engine.revision.load(SeqCst);

    direct_capture(engine, 0, 0, (BLOCK * 2) as i64, true);

    let state = engine.state.lock();
    let after = state.tracks[0]
        .peaks
        .iter()
        .map(|peak| (peak.min, peak.max, peak.valid))
        .collect::<Vec<_>>();
    assert_eq!(
        before, after,
        "extrema alone must not establish source identity"
    );
    assert!(engine.revision.load(SeqCst) > revision);
    assert!(state.review.covers(0, (BLOCK * 2) as i64));
    drop(state);
    assert!(
        engine
            .analyze(Settings::default())
            .unwrap_err()
            .contains("changed")
    );
}

#[test]
fn reopening_retains_waveforms_but_requires_verified_pcm_before_reusing_examples() {
    let mut fixture = Fixture::new(1);
    direct_capture(fixture.engine(), 0, 0, (BLOCK * 2) as i64, false);
    fixture
        .engine()
        .label(0, (BLOCK * 2) as i64, Some(1))
        .unwrap();
    fixture.reopen();
    {
        let state = fixture.engine().state.lock();
        assert!(state.tracks[0].coverage.covers(0, (BLOCK * 2) as i64));
        assert!(state.tracks[0].verified.0.is_empty());
        assert!(state.tracks[0].peaks.iter().any(|p| p.valid));
        assert!(state.review.covers(0, (BLOCK * 2) as i64));
        assert_eq!(state.labels.len(), 1);
    }
    let revision = fixture.engine().revision.load(SeqCst);
    direct_capture(fixture.engine(), 0, 0, (BLOCK * 2) as i64, false);
    let state = fixture.engine().state.lock();
    assert!(state.tracks[0].verified.covers(0, (BLOCK * 2) as i64));
    assert!(
        state.review.0.is_empty(),
        "unchanged full PCM must validate the saved label digest"
    );
    assert_eq!(fixture.engine().revision.load(SeqCst), revision);
}

#[test]
fn cached_render_requires_current_exact_pcm_and_monitoring_reconstructs_original() {
    let fixture = Fixture::new(1);
    let samples = (0..BLOCK)
        .map(|p| sample(p as i64, false))
        .collect::<Vec<_>>();
    install_constant_analysis(fixture.engine(), &samples);
    let mut handle = CaptureHandle::new(fixture.engine().clone(), 0, 1);
    let mut unknown = samples.clone();
    handle.render(&mut [&mut unknown], Some(0), 0, true, ListenMode::Processed);
    assert_eq!(
        unknown, samples,
        "analysis alone does not verify the current source"
    );

    let mut input = samples.clone();
    handle.capture(Some(0), &[&mut input], true);
    let mut processed = samples.clone();
    handle.render(
        &mut [&mut processed],
        Some(0),
        0,
        true,
        ListenMode::Processed,
    );
    assert!(
        processed
            .iter()
            .zip(&samples)
            .all(|(&out, &original)| out == original * 0.25)
    );
    let mut removed = samples.clone();
    handle.render(&mut [&mut removed], Some(0), 0, true, ListenMode::Removed);
    assert!(
        processed
            .iter()
            .zip(&removed)
            .zip(&samples)
            .all(|((&p, &r), &o)| p + r == o)
    );

    handle.capture(Some(BLOCK as i64), &[&mut input], false);
    let mut stopped = samples.clone();
    handle.render(&mut [&mut stopped], Some(0), 0, true, ListenMode::Processed);
    assert_eq!(stopped, samples, "stopping invalidates source verification");

    let mut mismatch = samples.iter().map(|value| -*value).collect::<Vec<_>>();
    handle.capture(Some(0), &[&mut mismatch], true);
    let mut output = mismatch.clone();
    handle.render(&mut [&mut output], Some(0), 0, true, ListenMode::Processed);
    assert_eq!(output, mismatch);
    let mut missing_time = samples.clone();
    handle.render(
        &mut [&mut missing_time],
        None,
        0,
        true,
        ListenMode::Processed,
    );
    assert_eq!(missing_time, samples);
}

#[test]
fn cache_writer_is_exclusive_and_departing_member_cannot_write_another_slot() {
    let fixture = Fixture::new(2);
    let duplicate = SessionCapture::create(
        fixture.key.clone(),
        RATE,
        fixture.root.clone(),
        Vec::new(),
        Settings::default(),
        Vec::new(),
    );
    assert!(
        duplicate.is_err(),
        "one persistent cache must have one writer"
    );
    direct_capture(fixture.engine(), 1, 0, BLOCK as i64, false);
    let handle = CaptureHandle::new(fixture.engine().clone(), 0, 1);
    handle.unbind();
    direct_capture(fixture.engine(), 0, 0, BLOCK as i64, true);
    let state = fixture.engine().state.lock();
    assert!(state.tracks[0].coverage.0.is_empty());
    assert!(state.tracks[1].coverage.covers(0, BLOCK as i64));
    assert_eq!(fixture.engine().owners[0].load(SeqCst), 0);
    assert_eq!(fixture.engine().owners[1].load(SeqCst), 2);
}

#[test]
fn unregistering_checkpoints_the_accepted_pcm_tail_before_releasing_its_slot() {
    let fixture = Fixture::new(1);
    let mut handle = CaptureHandle::new(fixture.engine().clone(), 0, 1);
    let mut input = vec![0.25; BLOCK];
    handle.capture(Some(0), &[&mut input], true);
    handle.unbind();
    let state = fixture.engine().state.lock();
    assert!(state.tracks[0].coverage.covers(0, BLOCK as i64));
    assert_eq!(fixture.engine().owners[0].load(SeqCst), 0);
    let manifest: Manifest =
        serde_json::from_reader(File::open(fixture.engine().path.join("capture.json")).unwrap())
            .unwrap();
    assert!(manifest.tracks[0].coverage.covers(0, BLOCK as i64));
}

#[test]
fn identical_replay_rebuilds_a_missing_waveform_index_without_invalidating_audio() {
    let fixture = Fixture::new(1);
    direct_capture(fixture.engine(), 0, 0, BLOCK as i64, false);
    let revision = fixture.engine().revision.load(SeqCst);
    {
        let mut state = fixture.engine().state.lock();
        state.tracks[0].peaks.clear();
        state.tracks[0].peak_file = None;
        fs::remove_file(fixture.engine().path.join("mic-0.peaks")).unwrap();
    }
    direct_capture(fixture.engine(), 0, 0, BLOCK as i64, false);
    let state = fixture.engine().state.lock();
    assert!(state.tracks[0].peaks.iter().any(|peak| peak.valid));
    assert_eq!(fixture.engine().revision.load(SeqCst), revision);
    assert!(fixture.engine().path.join("mic-0.peaks").is_file());
}

#[test]
fn captured_stereo_session_analyzes_publishes_sparse_members_and_validates_rendered_pcm() {
    const NATIVE_RATE: u32 = 48000;
    const FRAMES: usize = NATIVE_RATE as usize * 4;
    const SLOTS: [usize; 2] = [2, 6];
    let fixture = Fixture::new_with_rate(2, NATIVE_RATE);
    let engine = fixture.engine();
    {
        let mut members = engine.members.lock();
        for slot in 0..MEMBERS {
            engine.owners[slot].store(0, SeqCst);
            members[slot] = Weak::new();
        }
        for (speaker, &slot) in SLOTS.iter().enumerate() {
            engine.owners[slot].store(speaker as u64 + 1, SeqCst);
            members[slot] = Arc::downgrade(&fixture.params[speaker]);
        }
    }
    let mut handles: [CaptureHandle; 2] = std::array::from_fn(|speaker| {
        CaptureHandle::new(engine.clone(), SLOTS[speaker], speaker as u64 + 1)
    });

    // Use the successful bridge fixture's varied direct/bleed relationships.
    // Opposite-polarity stereo catches accidental downmixing; this is not a sound-quality test.
    let mut random = 7654321_u32;
    let noise = (0..64000)
        .map(|_| {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            ((random >> 8) as f32 / 16777216.0 - 0.5) * 0.2
        })
        .collect::<Vec<_>>();
    let streams: [Vec<f32>; 2] = std::array::from_fn(|speaker| {
        (0..FRAMES)
            .map(|frame| {
                let index = frame / 3;
                let active_a = (8000..24000).contains(&index) || (48000..60000).contains(&index);
                let active_b = (32000..44000).contains(&index) || (52000..60000).contains(&index);
                let a = noise[index] * if active_a { 1.0 } else { 0.0001 };
                let b = noise[(index + 700) % noise.len()] * if active_b { 1.0 } else { 0.0001 };
                if speaker == 0 {
                    a + b * 0.1
                } else {
                    b + a * 0.2
                }
            })
            .collect()
    });
    let origin = i64::from(NATIVE_RATE) * 10 + 137;
    for offset in (0..FRAMES).step_by(BLOCK) {
        let count = (FRAMES - offset).min(BLOCK);
        for speaker in 0..2 {
            let mut left = streams[speaker][offset..offset + count].to_vec();
            let mut right = left.iter().map(|value| -*value).collect::<Vec<_>>();
            handles[speaker].capture(Some(origin + offset as i64), &[&mut left, &mut right], true);
        }
        if (offset / BLOCK).is_multiple_of(16) {
            engine.flush().unwrap();
        }
    }
    engine.flush().unwrap();
    let capture_deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let complete = {
            let state = engine.state.lock();
            SLOTS.iter().all(|&slot| {
                state.tracks[slot]
                    .verified
                    .covers(origin, origin + FRAMES as i64)
            })
        };
        if complete {
            break;
        }
        assert!(
            Instant::now() < capture_deadline,
            "the capture actor did not consume all accepted PCM"
        );
        thread::sleep(Duration::from_millis(5));
    }

    for (start, end, mask) in [
        (0, NATIVE_RATE / 4, 0),
        (NATIVE_RATE / 2, NATIVE_RATE * 3 / 2, 1 << SLOTS[0]),
        (NATIVE_RATE * 2, NATIVE_RATE * 11 / 4, 1 << SLOTS[1]),
        (
            NATIVE_RATE * 13 / 4,
            NATIVE_RATE * 29 / 8,
            (1 << SLOTS[0]) | (1 << SLOTS[1]),
        ),
    ] {
        engine
            .label(
                origin + i64::from(start),
                origin + i64::from(end),
                Some(mask),
            )
            .unwrap();
    }
    engine.analyze(Settings::default()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(view) = engine.view(None, None, 32) {
            if view.ready {
                break;
            }
            if !view.busy || Instant::now() >= deadline {
                engine.invalidate();
                for handle in &handles {
                    handle.unbind();
                }
                panic!(
                    "the coupled Session analysis did not publish: {}",
                    view.message
                );
            }
        }
        if Instant::now() >= deadline {
            engine.invalidate();
            for handle in &handles {
                handle.unbind();
            }
            panic!("the coupled Session analysis did not respond within 20 seconds");
        }
        thread::sleep(Duration::from_millis(10));
    }
    let analysis = {
        let guard = engine.analysis.read();
        let published = guard.as_ref().unwrap();
        assert_eq!(published.analysis.origin, origin);
        assert_eq!(published.analysis.end, origin + FRAMES as i64);
        assert_eq!(published.analysis.rate, NATIVE_RATE);
        assert_eq!(published.analysis.envelopes.len(), 2);
        for (dense, &slot) in SLOTS.iter().enumerate() {
            assert_eq!(published.member_map[slot], Some(dense));
            assert!(!published.hashes[slot].is_empty());
            assert_eq!(published.analysis.envelopes[dense].sample_count(), FRAMES);
        }
        for slot in 0..MEMBERS {
            if !SLOTS.contains(&slot) {
                assert_eq!(published.member_map[slot], None);
                assert!(published.hashes[slot].is_empty());
            }
        }
        published.analysis.clone()
    };

    // Mic 7 is explicitly absent during Mic 3's solo example, well away from its fades.
    let block_start = (origin + i64::from(NATIVE_RATE)) / BLOCK as i64 * BLOCK as i64;
    let offset = (block_start - origin) as usize;
    let original_left = streams[1][offset..offset + BLOCK].to_vec();
    let original_right = original_left
        .iter()
        .map(|value| -*value)
        .collect::<Vec<_>>();
    assert!(analysis.gain_at(1, block_start + BLOCK as i64 / 2) < 0.9);
    let mut unverified_left = original_left.clone();
    let mut unverified_right = original_right.clone();
    handles[1].render(
        &mut [&mut unverified_left, &mut unverified_right],
        Some(block_start),
        0,
        true,
        ListenMode::Processed,
    );
    assert_eq!(
        unverified_left, original_left,
        "publication alone must not verify this replay"
    );
    assert_eq!(unverified_right, original_right);

    let mut input_left = original_left.clone();
    let mut input_right = original_right.clone();
    handles[1].capture(
        Some(block_start),
        &[&mut input_left, &mut input_right],
        true,
    );
    let mut processed_left = original_left.clone();
    let mut processed_right = original_right.clone();
    handles[1].render(
        &mut [&mut processed_left, &mut processed_right],
        Some(block_start),
        0,
        true,
        ListenMode::Processed,
    );
    for n in 0..BLOCK {
        let gain = analysis.gain_at(1, block_start + n as i64);
        assert_eq!(processed_left[n], original_left[n] * gain);
        assert_eq!(processed_right[n], original_right[n] * gain);
    }
    assert_ne!(
        processed_left, original_left,
        "validated cached processing must actually attenuate"
    );

    let mut missing = original_left.clone();
    handles[1].render(&mut [&mut missing], None, 0, true, ListenMode::Processed);
    assert_eq!(missing, original_left);
    handles[1].capture(
        Some(block_start),
        &[&mut input_left, &mut input_right],
        false,
    );
    let mut stopped = original_left.clone();
    handles[1].render(
        &mut [&mut stopped],
        Some(block_start),
        0,
        true,
        ListenMode::Processed,
    );
    assert_eq!(stopped, original_left);

    let revision = engine.revision.load(SeqCst);
    let mut changed_left = original_left
        .iter()
        .map(|value| -*value)
        .collect::<Vec<_>>();
    let mut changed_right = original_right
        .iter()
        .map(|value| -*value)
        .collect::<Vec<_>>();
    handles[1].capture(
        Some(block_start),
        &[&mut changed_left, &mut changed_right],
        true,
    );
    assert!(engine.revision.load(SeqCst) > revision);
    let original_changed = changed_left.clone();
    handles[1].render(
        &mut [&mut changed_left, &mut changed_right],
        Some(block_start),
        0,
        true,
        ListenMode::Processed,
    );
    assert_eq!(
        changed_left, original_changed,
        "source edits must immediately preserve original audio"
    );
    for handle in &handles {
        handle.unbind();
    }
}
