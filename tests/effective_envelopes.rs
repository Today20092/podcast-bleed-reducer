use rust_app::{
    AutomaticEnvelope, GainFadeSettings, OWNERSHIP_FRAME_TICKS, SourceTimebase, TemporalEvidence,
    TemporalEvidenceGrid, TemporalOwnershipCache, TemporalSettings, TrackId,
};

fn ownership() -> std::sync::Arc<rust_app::TemporalOwnership> {
    ownership_with_offset(0)
}

#[test]
fn zero_fades_and_gap_mute_absence_without_touching_speech() {
    let result = ownership();
    let grid = SourceTimebase::new(48000, 153600).unwrap();
    let envelope = AutomaticEnvelope::from_ownership(
        &result,
        TrackId(1),
        grid,
        GainFadeSettings {
            attenuation: 0.0,
            fade_in: 0,
            fade_out: 0,
            minimum_gap: 0,
        },
        None,
    )
    .unwrap();
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 42), 0.0);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 30), 1.0);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 85), 0.0);
}
fn ownership_with_offset(offset: usize) -> std::sync::Arc<rust_app::TemporalOwnership> {
    let rows = (0..100)
        .map(|frame| {
            let active = (20 + offset..40 + offset).contains(&frame)
                || (44 + offset..50 + offset).contains(&frame);
            let value = if active { 0.95 } else { 0.01 };
            TemporalEvidence {
                probability: Some(value),
                raw_speech: Some(value),
                assigned_activity: Some(value),
                preservation_reason: None,
            }
        })
        .collect();
    let grid = TemporalEvidenceGrid::new(
        vec![TrackId(1)],
        OWNERSHIP_FRAME_TICKS * 100,
        vec![rows],
        vec![0.01; 100],
    )
    .unwrap();
    TemporalOwnershipCache::default()
        .optimize_evidence(&grid, TemporalSettings::default(), None)
        .unwrap()
}

#[test]
fn default_gap_hold_preserves_brief_absence_between_speech() {
    let result = ownership();
    for rate in [44100, 48000, 96000] {
        let grid = SourceTimebase::new(rate, u64::from(rate) * 32 / 10).unwrap();
        let envelope = AutomaticEnvelope::from_ownership(
            &result,
            TrackId(1),
            grid,
            GainFadeSettings::default(),
            None,
        )
        .unwrap();
        assert_eq!(
            envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 42),
            1.0,
            "a 128 ms bounded gap must be ignored by the default hold"
        );
        assert!(
            envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 85) < 0.2,
            "long absence must still attenuate"
        );
        let short_hold = AutomaticEnvelope::from_ownership(
            &result,
            TrackId(1),
            grid,
            GainFadeSettings {
                minimum_gap: rust_app::TICKS_PER_SECOND / 100,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        assert!(
            short_hold.gain_at_time(OWNERSHIP_FRAME_TICKS * 42) < 0.55,
            "lowering the hold to 10 ms must expose the same 128 ms gap"
        );
        let equal_hold = AutomaticEnvelope::from_ownership(
            &result,
            TrackId(1),
            grid,
            GainFadeSettings {
                minimum_gap: OWNERSHIP_FRAME_TICKS * 4,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        assert!(
            equal_hold.gain_at_time(OWNERSHIP_FRAME_TICKS * 42) < 0.55,
            "a gap exactly the selected duration qualifies for attenuation"
        );
    }
}

#[test]
fn finite_fades_fit_short_absent_gaps_and_preserve_speech_endpoints() {
    let result = ownership();
    let settings = GainFadeSettings {
        attenuation: 0.25,
        fade_in: OWNERSHIP_FRAME_TICKS * 4,
        fade_out: OWNERSHIP_FRAME_TICKS * 4,
        minimum_gap: 0,
    };
    let grid = SourceTimebase::new(48000, 153600).unwrap();
    let envelope =
        AutomaticEnvelope::from_ownership(&result, TrackId(1), grid, settings, None).unwrap();
    assert_eq!(envelope.gain_at_time(0), 1.0);
    assert_eq!(envelope.gain_at_time(settings.fade_out), 0.25);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 16), 0.25);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 20), 1.0);
    for frame in (20..40).chain(44..50) {
        assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * frame), 1.0);
    }
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 40), 1.0);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 42), 0.25);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 44), 1.0);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 54), 0.25);
    let interior = envelope.gain_at_time(settings.fade_out / 2);
    assert_eq!(interior, 0.625, "S-shaped easing has a symmetric midpoint");
    assert!(
        envelope.segment_count() < 12,
        "representation must scale with edges, not samples"
    );
}

#[test]
fn long_requested_fades_do_not_erase_known_absence() {
    let result = ownership();
    let settings = GainFadeSettings {
        attenuation: 0.1,
        fade_in: OWNERSHIP_FRAME_TICKS * 10,
        fade_out: OWNERSHIP_FRAME_TICKS * 35,
        minimum_gap: 0,
    };
    let grid = SourceTimebase::new(48000, 153600).unwrap();
    let envelope =
        AutomaticEnvelope::from_ownership(&result, TrackId(1), grid, settings, None).unwrap();
    let start = OWNERSHIP_FRAME_TICKS * 40;
    let end = OWNERSHIP_FRAME_TICKS * 44;
    let meeting = start + (end - start) * 35 / 45;
    assert_eq!(envelope.gain_at_time(start), 1.0);
    assert_eq!(envelope.gain_at_time(meeting), 0.1);
    assert_eq!(envelope.gain_at_time(end), 1.0);
    assert!(envelope.gain_at_time((start + end) / 2) < 0.5);
    for frame in (20..40).chain(44..50) {
        assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * frame), 1.0);
    }
}

fn project() -> (std::path::PathBuf, rust_app::Project) {
    project_at_stamp(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    )
}

fn project_at_stamp(stamp: u128) -> (std::path::PathBuf, rust_app::Project) {
    static NEXT_PROJECT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "envelopes-{}-{}-{}",
        std::process::id(),
        stamp,
        NEXT_PROJECT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("mono.wav");
    let mut writer =
        bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(48000, 16))
            .unwrap()
            .audio_frame_writer()
            .unwrap();
    writer.write_frames(&vec![0_i16; 153600]).unwrap();
    writer.end().unwrap();
    let mut project =
        rust_app::Project::create(root.join("project.cleanslate"), vec![path]).unwrap();
    project.set_supplied_alignment(true).unwrap();
    (root, project)
}

#[test]
fn parallel_fixtures_keep_sources_independent_when_the_clock_repeats() {
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| project_at_stamp(0));
        let second = scope.spawn(|| project_at_stamp(0));
        (first.join().unwrap(), second.join().unwrap())
    });
    assert_ne!(first.0, second.0);
    std::fs::remove_dir_all(first.0).unwrap();
    rust_app::SourceMetadata::read(&second.1.sources[0].path).unwrap();
    std::fs::remove_dir_all(second.0).unwrap();
}

#[test]
fn live_gap_setting_persists_and_human_absence_and_manual_gain_remain_authoritative() {
    let (root, mut project) = project();
    let seed = ownership();
    let rows = (0..100)
        .map(|frame| {
            seed.evidence()
                .evidence_at(frame * OWNERSHIP_FRAME_TICKS)
                .unwrap()[0]
                .clone()
        })
        .collect();
    let grid = TemporalEvidenceGrid::new(
        vec![project.sources[0].track_id],
        seed.duration,
        vec![rows],
        vec![0.01; 100],
    )
    .unwrap();
    let current = TemporalOwnershipCache::default()
        .optimize_project_evidence(
            &project,
            &grid,
            TemporalSettings {
                preserved_gap_frames: 0,
                ..Default::default()
            },
            None,
        )
        .unwrap();
    let mut cache = rust_app::EffectiveEnvelopeCache::default();
    let before = cache
        .update(
            &project,
            &current,
            rust_app::project_fade_settings(&project).unwrap(),
            None,
        )
        .unwrap();
    let middle = OWNERSHIP_FRAME_TICKS * 42;
    assert_eq!(before.envelopes[0].gain_at_time(middle), 1.0);
    project
        .set_parameters(rust_app::AutomixParameters {
            minimum_gap_ms: 10.0,
            ..project.parameters()
        })
        .unwrap();
    let after = cache
        .update(
            &project,
            &current,
            rust_app::project_fade_settings(&project).unwrap(),
            None,
        )
        .unwrap();
    assert!(after.envelopes[0].gain_at_time(middle) < 0.55);
    assert_ne!(before.cache_key, after.cache_key);
    assert!(
        after.analysis_current,
        "gain hold changes must not require training again"
    );
    assert_eq!(
        rust_app::Project::open(project.path())
            .unwrap()
            .parameters()
            .minimum_gap_ms,
        10.0
    );
    for invalid in [-1.0, 60001.0, f32::NAN] {
        assert!(
            project
                .set_parameters(rust_app::AutomixParameters {
                    minimum_gap_ms: invalid,
                    ..project.parameters()
                })
                .is_err()
        );
    }
    project
        .set_parameters(rust_app::AutomixParameters {
            minimum_gap_ms: 5000.0,
            ..project.parameters()
        })
        .unwrap();
    let range =
        rust_app::TimeRange::new(OWNERSHIP_FRAME_TICKS * 40, OWNERSHIP_FRAME_TICKS * 48).unwrap();
    project
        .replace_reference_labels(vec![rust_app::ReferenceLabel {
            id: 1,
            range,
            active_speakers: vec![],
            reference_track: None,
            sources: vec![rust_app::SourceBinding {
                track: project.sources[0].track_id,
                fingerprint: project.sources[0].fingerprint.clone(),
            }],
        }])
        .unwrap();
    let truth = TemporalOwnershipCache::default()
        .optimize_project_evidence(
            &project,
            &grid,
            TemporalSettings {
                preserved_gap_frames: 0,
                ..Default::default()
            },
            None,
        )
        .unwrap();
    let labeled = cache
        .update(
            &project,
            &truth,
            rust_app::project_fade_settings(&project).unwrap(),
            None,
        )
        .unwrap();
    let unheld = cache
        .update(
            &project,
            &truth,
            GainFadeSettings {
                minimum_gap: rust_app::TICKS_PER_SECOND / 100,
                ..rust_app::project_fade_settings(&project).unwrap()
            },
            None,
        )
        .unwrap();
    for frame in 0..100 {
        let time = OWNERSHIP_FRAME_TICKS * frame;
        assert_eq!(
            labeled.envelopes[0].gain_at_time(time),
            unheld.envelopes[0].gain_at_time(time),
            "human room tone must have the same envelope with a 5 second hold as with 10 ms"
        );
    }
    assert!(
        labeled.envelopes[0].gain_at_time(OWNERSHIP_FRAME_TICKS * 47) < 0.2,
        "the room-tone gap must still reach substantial attenuation inside its fitted fades"
    );
    let draft = project
        .region_gain_edit(
            project.sources[0].track_id,
            range,
            rust_app::GainRegionAction::Attenuate(0.4),
        )
        .unwrap();
    project.commit_gain_edit(draft).unwrap();
    let manual = cache
        .update(
            &project,
            &truth,
            rust_app::project_fade_settings(&project).unwrap(),
            None,
        )
        .unwrap();
    assert_eq!(manual.envelopes[0].gain_at_time(middle), 0.4);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_grid_rounding_preserves_every_boundary_sample_including_arbitrary_rates() {
    let ownership = ownership_with_offset(1);
    let settings = GainFadeSettings {
        attenuation: 0.125,
        fade_in: OWNERSHIP_FRAME_TICKS * 4,
        fade_out: OWNERSHIP_FRAME_TICKS * 4,
        minimum_gap: 0,
    };
    for rate in [44100, 48000, 96000, 47999] {
        let grid = SourceTimebase::new(rate, u64::from(rate) * 32 / 10).unwrap();
        let auto = AutomaticEnvelope::from_ownership(&ownership, TrackId(1), grid, settings, None)
            .unwrap();
        for (start, end) in [(21, 41), (45, 51)] {
            let range = rust_app::TimeRange::new(
                OWNERSHIP_FRAME_TICKS * start,
                OWNERSHIP_FRAME_TICKS * end,
            )
            .unwrap()
            .native_frames(grid)
            .unwrap();
            for sample in range {
                assert_eq!(
                    auto.gain_at_time(
                        grid.time_at_frame(sample as u64, rust_app::Rounding::Floor)
                            .unwrap()
                    ),
                    1.0,
                    "rate {rate}, native sample {sample}"
                );
            }
        }
        assert_eq!(auto.gain_at_time(grid.duration().unwrap()), 1.0);
    }
    assert!(
        AutomaticEnvelope::from_ownership(
            &ownership,
            TrackId(1),
            SourceTimebase::new(48000, 153600).unwrap(),
            GainFadeSettings {
                fade_in: 0,
                ..settings
            },
            None
        )
        .is_ok()
    );
}

#[test]
fn a_bound_session_protection_does_not_expand_when_a_track_is_added() {
    let (root, mut project) = project();
    project
        .protect_range(0, OWNERSHIP_FRAME_TICKS * 100)
        .unwrap();
    let new_path = root.join("new.wav");
    std::fs::copy(&project.sources[0].path, &new_path).unwrap();
    project.add_sources(vec![new_path]).unwrap();
    project.set_supplied_alignment(true).unwrap();
    let rows = vec![
        vec![
            TemporalEvidence {
                probability: Some(0.01),
                raw_speech: Some(0.01),
                assigned_activity: Some(0.01),
                preservation_reason: None
            };
            100
        ];
        2
    ];
    let grid = TemporalEvidenceGrid::new(
        project.sources.iter().map(|s| s.track_id).collect(),
        OWNERSHIP_FRAME_TICKS * 100,
        rows,
        vec![0.01; 100],
    )
    .unwrap();
    let ownership = TemporalOwnershipCache::default()
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    let settings = GainFadeSettings {
        attenuation: 0.25,
        fade_in: OWNERSHIP_FRAME_TICKS,
        fade_out: OWNERSHIP_FRAME_TICKS,
        minimum_gap: 0,
    };
    let result = rust_app::EffectiveEnvelopeCache::default()
        .update(&project, &ownership, settings, None)
        .unwrap();
    assert_eq!(
        result.envelopes[0].gain_at_time(OWNERSHIP_FRAME_TICKS * 20),
        1.0
    );
    assert_eq!(
        result.envelopes[1].gain_at_time(OWNERSHIP_FRAME_TICKS * 20),
        0.25,
        "old preserve constraints only protect their bound tracks"
    );
    project
        .replace_manual_gain_edits(vec![rust_app::ManualGainEdit {
            id: 1,
            source: rust_app::SourceBinding {
                track: project.sources[1].track_id,
                fingerprint: project.sources[1].fingerprint.clone(),
            },
            interpolation: rust_app::ManualInterpolation::Linear,
            points: vec![
                rust_app::EnvelopePoint {
                    time: OWNERSHIP_FRAME_TICKS * 20,
                    gain: 0.5,
                },
                rust_app::EnvelopePoint {
                    time: OWNERSHIP_FRAME_TICKS * 24,
                    gain: 0.5,
                },
            ],
        }])
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn failed_gain_save_and_cancelled_drag_leave_file_state_and_undo_unchanged() {
    use std::os::windows::fs::OpenOptionsExt;
    let (root, mut project) = project();
    let range =
        rust_app::TimeRange::new(OWNERSHIP_FRAME_TICKS * 20, OWNERSHIP_FRAME_TICKS * 24).unwrap();
    let draft = project
        .region_gain_edit(
            project.sources[0].track_id,
            range,
            rust_app::GainRegionAction::Attenuate(0.2),
        )
        .unwrap();
    let prior = std::fs::read(project.path()).unwrap();
    let cancelled = draft.clone();
    drop(cancelled);
    assert_eq!(std::fs::read(project.path()).unwrap(), prior);
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(project.path())
        .unwrap();
    assert!(project.commit_gain_edit(draft.clone()).is_err());
    assert_eq!(std::fs::read(project.path()).unwrap(), prior);
    assert!(project.manual_gain_edits().is_empty());
    assert!(!project.undo_gain_edit().unwrap());
    drop(lock);
    project.commit_gain_edit(draft.clone()).unwrap();
    assert!(
        project.commit_gain_edit(draft).is_err(),
        "an obsolete drag cannot replace newer manual records"
    );
    project.undo_gain_edit().unwrap();
    assert!(project.manual_gain_edits().is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn touching_manual_constraints_have_explicit_right_endpoint_ownership_and_exact_bounds() {
    let (root, mut project) = project();
    let seed = ownership();
    let rows = (0..100)
        .map(|i| {
            seed.evidence()
                .evidence_at(i * OWNERSHIP_FRAME_TICKS)
                .unwrap()[0]
                .clone()
        })
        .collect();
    let grid = TemporalEvidenceGrid::new(
        vec![project.sources[0].track_id],
        seed.duration,
        vec![rows],
        vec![0.01; 100],
    )
    .unwrap();
    let current = TemporalOwnershipCache::default()
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    let mut draft = project
        .region_gain_edit(
            project.sources[0].track_id,
            rust_app::TimeRange::new(OWNERSHIP_FRAME_TICKS * 56, OWNERSHIP_FRAME_TICKS * 60)
                .unwrap(),
            rust_app::GainRegionAction::Attenuate(0.5),
        )
        .unwrap();
    draft.constraints.manual_edits[0].points[1].gain = 0.25;
    project.commit_gain_edit(draft).unwrap();
    let second = project
        .region_gain_edit(
            project.sources[0].track_id,
            rust_app::TimeRange::new(OWNERSHIP_FRAME_TICKS * 60, OWNERSHIP_FRAME_TICKS * 64)
                .unwrap(),
            rust_app::GainRegionAction::Attenuate(0.75),
        )
        .unwrap();
    project.commit_gain_edit(second).unwrap();
    let protection = project
        .region_gain_edit(
            project.sources[0].track_id,
            rust_app::TimeRange::new(OWNERSHIP_FRAME_TICKS * 64, OWNERSHIP_FRAME_TICKS * 68)
                .unwrap(),
            rust_app::GainRegionAction::Preserve,
        )
        .unwrap();
    project.commit_gain_edit(protection).unwrap();
    let settings = GainFadeSettings {
        attenuation: 0.125,
        fade_in: OWNERSHIP_FRAME_TICKS,
        fade_out: OWNERSHIP_FRAME_TICKS,
        minimum_gap: 0,
    };
    let mut cache = rust_app::EffectiveEnvelopeCache::default();
    let effective = cache.update(&project, &current, settings, None).unwrap();
    let envelope = &effective.envelopes[0];
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 60), 0.75);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 64), 1.0);
    assert_eq!(envelope.gain_at_time(OWNERSHIP_FRAME_TICKS * 68), 0.125);
    for (start, end) in [
        (0, 5000),
        (24575, 30721),
        (82943, 92161),
        (92159, 104449),
        (104448, 110000),
    ] {
        let expected = (start..end)
            .map(|sample| envelope.gain_at(sample))
            .fold((1.0_f32, 0.0_f32), |(lo, hi), g| (lo.min(g), hi.max(g)));
        assert_eq!(envelope.gain_bounds(start, end).unwrap(), expected);
    }
    let mut replace = project.begin_gain_edit();
    replace.constraints.manual_edits.clear();
    replace.constraints.protections.clear();
    project.commit_gain_edit(replace).unwrap();
    project.protect_track(0).unwrap();
    let whole = cache.update(&project, &current, settings, None).unwrap();
    assert_eq!(whole.envelopes[0].gain_bounds(0, 153600), Some((1.0, 1.0)));
    assert_eq!(whole.envelopes[0].gain_at(0), 1.0);
    assert_eq!(whole.envelopes[0].gain_at(153599), 1.0);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn previews_and_cancelled_jobs_do_not_install_and_changed_sources_require_explicit_rebind() {
    use std::sync::{Arc, atomic::AtomicBool};
    let (root, mut project) = project();
    let seed = ownership();
    let rows = (0..100)
        .map(|i| {
            seed.evidence()
                .evidence_at(i * OWNERSHIP_FRAME_TICKS)
                .unwrap()[0]
                .clone()
        })
        .collect();
    let grid = TemporalEvidenceGrid::new(
        vec![project.sources[0].track_id],
        seed.duration,
        vec![rows],
        vec![0.01; 100],
    )
    .unwrap();
    let current = TemporalOwnershipCache::default()
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    let settings = GainFadeSettings {
        attenuation: 0.25,
        fade_in: OWNERSHIP_FRAME_TICKS * 4,
        fade_out: OWNERSHIP_FRAME_TICKS * 4,
        minimum_gap: 0,
    };
    let mut cache = rust_app::EffectiveEnvelopeCache::default();
    let accepted = cache.update(&project, &current, settings, None).unwrap();
    let start = OWNERSHIP_FRAME_TICKS * 20;
    let draft = project
        .region_gain_edit(
            project.sources[0].track_id,
            rust_app::TimeRange::new(start, start + OWNERSHIP_FRAME_TICKS * 4).unwrap(),
            rust_app::GainRegionAction::Attenuate(0.4),
        )
        .unwrap();
    let preview = cache
        .preview_edit(&project, &current, settings, &draft, None)
        .unwrap();
    assert_eq!(preview.envelopes[0].gain_at_time(start), 0.4);
    assert!(Arc::ptr_eq(cache.last_completed().unwrap(), &accepted));
    assert!(project.manual_gain_edits().is_empty());
    assert!(
        cache
            .preview_edit(
                &project,
                &current,
                settings,
                &draft,
                Some(&AtomicBool::new(true))
            )
            .is_err()
    );
    assert!(
        cache
            .update(
                &project,
                &current,
                GainFadeSettings {
                    attenuation: 0.125,
                    ..settings
                },
                Some(&AtomicBool::new(true))
            )
            .is_err()
    );
    assert!(Arc::ptr_eq(cache.last_completed().unwrap(), &accepted));
    assert_eq!(cache.automatic_builds(), 1);
    project.commit_gain_edit(draft).unwrap();
    let edited = cache.update(&project, &current, settings, None).unwrap();
    let records = project.manual_gain_edits().to_vec();
    let path = project.sources[0].path.clone();
    let track = project.sources[0].track_id;
    let mut writer =
        bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(48000, 16))
            .unwrap()
            .audio_frame_writer()
            .unwrap();
    writer.write_frames(&vec![100_i16; 153600]).unwrap();
    writer.end().unwrap();
    assert!(cache.update(&project, &current, settings, None).is_err());
    assert!(Arc::ptr_eq(cache.last_completed().unwrap(), &edited));
    assert_eq!(project.manual_gain_edits(), records);
    let mut reopened = rust_app::Project::open(project.path()).unwrap();
    reopened.relink(0, path).unwrap();
    assert_eq!(reopened.pending_source_reviews(), &[track]);
    assert!(cache.update(&reopened, &current, settings, None).is_err());
    reopened.review_and_rebind_source(track).unwrap();
    reopened.set_supplied_alignment(true).unwrap();
    assert!(
        cache.update(&reopened, &current, settings, None).is_err(),
        "reviewed edits cannot authorize old automatic analysis on changed audio"
    );
    let fresh = TemporalOwnershipCache::default()
        .optimize_project_evidence(&reopened, &grid, TemporalSettings::default(), None)
        .unwrap();
    let rebuilt = cache.update(&reopened, &fresh, settings, None).unwrap();
    assert_eq!(rebuilt.envelopes[0].gain_at_time(start), 0.4);
    assert_eq!(reopened.manual_gain_edits()[0].points, records[0].points);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_drag_is_one_atomic_gain_command_and_conflicting_protection_requires_replacement() {
    use rust_app::{
        BoundProtection, EnvelopePoint, ManualGainEdit, ManualInterpolation, SourceBinding,
        TimeRange,
    };
    let (root, mut project) = project();
    let track = project.sources[0].track_id;
    let source = SourceBinding {
        track,
        fingerprint: project.sources[0].fingerprint.clone(),
    };
    let mut draft = project.begin_gain_edit();
    draft.constraints.manual_edits.push(ManualGainEdit {
        id: 1,
        source: source.clone(),
        interpolation: ManualInterpolation::Linear,
        points: vec![
            EnvelopePoint {
                time: OWNERSHIP_FRAME_TICKS * 20,
                gain: 0.5,
            },
            EnvelopePoint {
                time: OWNERSHIP_FRAME_TICKS * 24,
                gain: 0.25,
            },
        ],
    });
    let unchanged = std::fs::read(project.path()).unwrap();
    draft.constraints.manual_edits[0].points[0].gain = 0.4;
    assert_eq!(
        std::fs::read(project.path()).unwrap(),
        unchanged,
        "drag previews are not permanent commands"
    );
    project.commit_gain_edit(draft).unwrap();
    let saved = project.manual_gain_edits().to_vec();
    assert!(project.undo_gain_edit().unwrap());
    assert!(project.manual_gain_edits().is_empty());
    assert!(!project.undo_gain_edit().unwrap());
    assert!(project.redo_gain_edit().unwrap());
    assert_eq!(project.manual_gain_edits(), saved);
    let mut conflict = project.begin_gain_edit();
    conflict.constraints.protections.push(BoundProtection {
        track: Some(track),
        range: TimeRange::new(OWNERSHIP_FRAME_TICKS * 20, OWNERSHIP_FRAME_TICKS * 24).unwrap(),
        sources: vec![source],
    });
    assert!(project.commit_gain_edit(conflict.clone()).is_err());
    assert_eq!(project.manual_gain_edits(), saved);
    conflict.constraints.manual_edits.clear();
    project.commit_gain_edit(conflict).unwrap();
    assert!(project.manual_gain_edits().is_empty());
    assert_eq!(project.recorded_protections().len(), 1);
    project.undo_gain_edit().unwrap();
    assert_eq!(project.manual_gain_edits(), saved);
    assert!(project.recorded_protections().is_empty());
    assert_eq!(
        rust_app::Project::open(project.path())
            .unwrap()
            .manual_gain_edits(),
        saved
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn manual_points_win_and_shared_evaluation_survives_reload_and_new_automatic_results() {
    use rust_app::{
        EffectiveEnvelopeCache, EnvelopePoint, ManualGainEdit, ManualInterpolation, SourceBinding,
    };
    let (root, mut project) = project();
    let seed = ownership();
    let rows = (0..100)
        .map(|i| {
            seed.evidence()
                .evidence_at(i * OWNERSHIP_FRAME_TICKS)
                .unwrap()[0]
                .clone()
        })
        .collect();
    let grid = TemporalEvidenceGrid::new(
        vec![project.sources[0].track_id],
        seed.duration,
        vec![rows],
        vec![0.01; 100],
    )
    .unwrap();
    let current = TemporalOwnershipCache::default()
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    let settings = GainFadeSettings {
        attenuation: 0.25,
        fade_in: OWNERSHIP_FRAME_TICKS * 4,
        fade_out: OWNERSHIP_FRAME_TICKS * 4,
        minimum_gap: 0,
    };
    let mut cache = EffectiveEnvelopeCache::default();
    let before = cache.update(&project, &current, settings, None).unwrap();
    let start = OWNERSHIP_FRAME_TICKS * 20;
    let end = OWNERSHIP_FRAME_TICKS * 24;
    project
        .replace_manual_gain_edits(vec![ManualGainEdit {
            id: 1,
            source: SourceBinding {
                track: project.sources[0].track_id,
                fingerprint: project.sources[0].fingerprint.clone(),
            },
            interpolation: ManualInterpolation::Linear,
            points: vec![
                EnvelopePoint {
                    time: start,
                    gain: 0.5,
                },
                EnvelopePoint {
                    time: end,
                    gain: 0.25,
                },
            ],
        }])
        .unwrap();
    let after = cache.update(&project, &current, settings, None).unwrap();
    let envelope = &after.envelopes[0];
    assert_eq!(envelope.gain_at_time(start), 0.5);
    assert_eq!(envelope.gain_at_time((start + end) / 2), 0.375);
    assert_eq!(
        envelope.gain_at_time(end),
        1.0,
        "half-open override restores automatic at its end"
    );
    assert_ne!(after.cache_key, before.cache_key);
    assert!(after.analysis_current);
    assert_eq!(
        cache.automatic_builds(),
        1,
        "manual changes reuse the automatic sparse layer"
    );
    let reopened = rust_app::Project::open(project.path()).unwrap();
    let restored = EffectiveEnvelopeCache::default()
        .update(&reopened, &current, settings, None)
        .unwrap();
    assert_eq!(restored.cache_key, after.cache_key);
    for frame in 0..153600 {
        assert_eq!(
            restored.envelopes[0].gain_at(frame),
            envelope.gain_at(frame)
        );
    }
    let mut cursor = 0;
    for frame in (0..153600).step_by(77).chain(100..1000) {
        assert_eq!(
            envelope.gain_at_sequential(frame, &mut cursor),
            envelope.gain_at(frame)
        );
    }
    let expected = (30720..36864)
        .map(|sample| envelope.gain_at(sample))
        .fold((1.0_f32, 0.0_f32), |(lo, hi), g| (lo.min(g), hi.max(g)));
    assert_eq!(envelope.gain_bounds(30720, 36864).unwrap(), expected);
    project.replace_manual_gain_edits(vec![]).unwrap();
    let removed = cache.update(&project, &current, settings, None).unwrap();
    assert_eq!(removed.envelopes[0].gain_at_time(start), 1.0);
    assert_eq!(removed.cache_key, before.cache_key);
    assert_eq!(cache.automatic_builds(), 1);
    project
        .replace_manual_gain_edits(reopened.manual_gain_edits().to_vec())
        .unwrap();
    let manual_records = project.manual_gain_edits().to_vec();
    project
        .add_reference_label(
            rust_app::TimeRange::new(start, start + rust_app::TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    let stale = cache.update(&project, &current, settings, None).unwrap();
    assert!(!stale.analysis_current);
    assert_eq!(stale.envelopes[0].gain_at_time(start), 0.5);
    let fresh = TemporalOwnershipCache::default()
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    let reanalyzed = cache.update(&project, &fresh, settings, None).unwrap();
    assert!(reanalyzed.analysis_current);
    assert_eq!(
        cache.automatic_builds(),
        2,
        "fresh ownership invalidates the automatic layer"
    );
    assert_eq!(reanalyzed.envelopes[0].gain_at_time(start), 0.5);
    assert_eq!(project.manual_gain_edits(), manual_records);
    project.replace_manual_gain_edits(vec![]).unwrap();
    let exposed = cache.update(&project, &fresh, settings, None).unwrap();
    assert_eq!(
        exposed.envelopes[0].gain_at_time(start),
        0.25,
        "removing an override reveals the fresh automatic result"
    );
    std::fs::remove_dir_all(root).unwrap();
}
