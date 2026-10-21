//! Opt-in component observations, not device latency or two-hour PCM qualification.
use rust_app::{
    EffectiveEnvelopeCache, GainFadeSettings, GainRegionAction, OWNERSHIP_FRAME_TICKS, Project,
    TICKS_PER_SECOND, TemporalEvidence, TemporalEvidenceGrid, TemporalOwnershipCache,
    TemporalSettings, TimeRange,
};
use std::{hint::black_box, sync::Arc, time::Instant};

fn summary(mut samples: Vec<f64>) -> serde_json::Value {
    let runs = samples.clone();
    samples.sort_by(f64::total_cmp);
    serde_json::json!({
        "runs": samples.len(),
        "median_ms": samples[samples.len() / 2],
        "p95_ms": samples[(samples.len() * 95).div_ceil(100) - 1],
        "max_ms": samples[samples.len() - 1],
        "runs_ms": runs,
    })
}

#[test]
#[ignore = "repeated source-bound warm composition timing; run explicitly with --ignored --nocapture"]
fn measure_source_bound_warm_edits() {
    let mut scenarios = vec![];
    for (label, tracks, seconds, rate, half_period) in [
        ("native-grid", 3, 60_u64, 48_000_u32, 64),
        ("long-sparse", 8, 7_200, 8, 128),
        ("long-dense", 8, 7_200, 8, 8),
    ] {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("warm-edits-{}-{stamp}-{label}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let mut paths = vec![];
        for track in 0..tracks {
            let path = root.join(format!("track-{track}.wav"));
            let mut writer =
                bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(rate, 16))
                    .unwrap()
                    .audio_frame_writer()
                    .unwrap();
            writer
                .write_frames(&vec![0_i16; (seconds * u64::from(rate)) as usize])
                .unwrap();
            writer.end().unwrap();
            paths.push(path);
        }
        let mut project = Project::create(root.join("owned.cleanslate"), paths).unwrap();
        project.set_supplied_alignment(true).unwrap();
        let saved = std::fs::read(project.path()).unwrap();
        let duration = seconds * TICKS_PER_SECOND;
        let frame_count = duration.div_ceil(OWNERSHIP_FRAME_TICKS) as usize;
        let evidence = (0..tracks)
            .map(|track| {
                (0..frame_count)
                    .map(|frame| {
                        let value = if (frame / half_period + track) % 2 == 0 {
                            0.95
                        } else {
                            0.01
                        };
                        TemporalEvidence {
                            probability: Some(value),
                            raw_speech: Some(value),
                            assigned_activity: Some(value),
                            preservation_reason: None,
                        }
                    })
                    .collect()
            })
            .collect();
        let grid = TemporalEvidenceGrid::new(
            project.sources.iter().map(|s| s.track_id).collect(),
            duration,
            evidence,
            vec![0.95; frame_count],
        )
        .unwrap();
        let ownership = TemporalOwnershipCache::default()
            .optimize_project_evidence(
                &project,
                &grid,
                TemporalSettings {
                    preserved_gap_frames: 0,
                    ..TemporalSettings::default()
                },
                None,
            )
            .unwrap();
        let mut settings = GainFadeSettings {
            attenuation: 0.25,
            fade_in: OWNERSHIP_FRAME_TICKS * 2,
            fade_out: OWNERSHIP_FRAME_TICKS * 2,
            minimum_gap: 0,
        };
        let mut cache = EffectiveEnvelopeCache::default();
        let started = Instant::now();
        let initial = cache.update(&project, &ownership, settings, None).unwrap();
        let initial_ms = started.elapsed().as_secs_f64() * 1000.0;
        assert!(initial.analysis_current);
        let mut dial = vec![];
        let mut drafts = vec![];
        let mut copies = vec![];
        let mut reused_tracks = vec![];
        for iteration in 0..25 {
            settings.attenuation = if iteration % 2 == 0 { 0.125 } else { 0.25 };
            let started = Instant::now();
            let accepted = cache.update(&project, &ownership, settings, None).unwrap();
            dial.push(started.elapsed().as_secs_f64() * 1000.0);
            let started = Instant::now();
            // Match the app's two deliveries; these remain private Vec snapshots.
            black_box(accepted.envelopes.clone());
            black_box(accepted.envelopes.clone());
            copies.push(started.elapsed().as_secs_f64() * 1000.0);
            let builds = cache.automatic_builds();
            let gain = if iteration % 2 == 0 { 0.4 } else { 0.6 };
            let draft = project
                .region_gain_edit(
                    project.sources[0].track_id,
                    TimeRange::new(TICKS_PER_SECOND, 2 * TICKS_PER_SECOND).unwrap(),
                    GainRegionAction::Attenuate(gain),
                )
                .unwrap();
            let started = Instant::now();
            let preview = cache
                .preview_edit(&project, &ownership, settings, &draft, None)
                .unwrap();
            drafts.push(started.elapsed().as_secs_f64() * 1000.0);
            reused_tracks.push(preview.reused_tracks);
            assert_eq!(preview.envelopes[0].gain_at_time(TICKS_PER_SECOND), gain);
            assert!(Arc::ptr_eq(cache.last_completed().unwrap(), &accepted));
            assert_eq!(cache.automatic_builds(), builds);
            assert!(Arc::ptr_eq(&preview.automatic[0], &accepted.automatic[0]));
            assert!(project.manual_gain_edits().is_empty());
            black_box(preview);
        }
        assert_eq!(cache.automatic_builds(), 26);
        assert_eq!(std::fs::read(project.path()).unwrap(), saved);
        scenarios.push(serde_json::json!({
            "scenario": label, "tracks": tracks, "seconds": seconds,
            "source_rate_hz": rate, "ownership_frames_per_track": frame_count,
            "ownership_regions": ownership.regions().len(),
            "activity_half_period_frames": half_period,
            "initial_composition_ms": initial_ms,
            "warm_dial": summary(dial), "warm_draft": summary(drafts),
            "two_envelope_vec_deliveries": summary(copies),
            "automatic_builds": cache.automatic_builds(),
            "drafts_reuse_automatic": true,
            "draft_reused_track_counts": reused_tracks,
            "inference_invoked": false,
        }));
        std::fs::remove_dir_all(root).unwrap();
    }
    let report = serde_json::to_string_pretty(&serde_json::json!({
        "version": 1, "purpose": "source-bound synchronous composition component timings",
        "source_commit": option_env!("CLEANSLATE_BUILD_COMMIT").unwrap_or("unrecorded"),
        "debug_assertions": cfg!(debug_assertions),
        "limits": "Synthetic evidence and silent owned files. Long fixtures use 8 Hz PCM only to exercise genuine two-hour source-bound coordinates cheaply. No inference, device, GUI input feedback, import or export timing; no two-hour ordinary-rate PCM qualification.",
        "scenarios": scenarios,
    })).unwrap();
    if let Ok(path) = std::env::var("CLEANSLATE_WARM_EDIT_REPORT") {
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap()
            .write_all(report.as_bytes())
            .unwrap();
    }
    println!("{report}");
}
