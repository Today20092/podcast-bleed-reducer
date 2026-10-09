#[path = "common/supervised_fixture.rs"]
mod fixture;

#[test]
fn mixed_native_exports_and_monitored_playback_share_the_effective_time_evaluator() {
    for rates in [[44100, 96000], [48000, 47999]] {
        let (root, mut project, audio, evidence) = fixture::session_at_rates(rates);
        let progress = std::sync::atomic::AtomicU8::new(0);
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let analysis = rust_app::EditorAnalysisWorker::default()
            .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancel)
            .unwrap();
        let draft = project
            .region_gain_edit(
                project.sources[0].track_id,
                rust_app::TimeRange::new(
                    rust_app::TICKS_PER_SECOND * 3,
                    rust_app::TICKS_PER_SECOND * 13 / 4,
                )
                .unwrap(),
                rust_app::GainRegionAction::Attenuate(0.25),
            )
            .unwrap();
        project.commit_gain_edit(draft).unwrap();
        let effective = analysis
            .install(
                &mut project,
                &mut rust_app::EffectiveEnvelopeCache::default(),
                Some(&evidence.runtime),
                None,
            )
            .unwrap();
        let snapshot = rust_app::SupervisedRenderSnapshot::capture(
            &project,
            &analysis,
            &effective,
            &evidence.runtime,
        )
        .unwrap();
        let original_sound = snapshot.sound_identity.clone();
        let destination = root.join("native");
        let gate = rust_app::RenderGeneration::default();
        rust_app::prepare_supervised_render(
            &project,
            &analysis,
            &effective,
            &snapshot,
            &destination,
            &progress,
            None,
        )
        .unwrap()
        .publish(&project, &evidence.runtime, &gate, gate.current(), None)
        .unwrap()
        .accept(&mut project)
        .unwrap();
        for (track, &rate) in rates.iter().enumerate() {
            let path = destination.join(format!("{:02}-speaker-{track}.wav", track + 1));
            let metadata = rust_app::SourceMetadata::read(&path).unwrap();
            assert_eq!(metadata.grid.sample_rate, rate);
            assert_eq!(metadata.grid.frame_count, u64::from(rate) * 4);
            let mut reader = bwavfile::WaveReader::open(&path)
                .unwrap()
                .audio_frame_reader()
                .unwrap();
            let mut original = bwavfile::WaveReader::open(&project.sources[track].path)
                .unwrap()
                .audio_frame_reader()
                .unwrap();
            let mut delivered = vec![0.0; rate as usize * 4];
            let mut input = delivered.clone();
            reader.read_frames(&mut delivered).unwrap();
            original.read_frames(&mut input).unwrap();
            for frame in 0..delivered.len() {
                let time = metadata
                    .grid
                    .time_at_frame(frame as u64, rust_app::Rounding::Floor)
                    .unwrap();
                let gain = effective.envelopes[track].gain_at_time(time);
                assert_eq!(gain, effective.envelopes[track].gain_at(frame));
                assert!((delivered[frame] - input[frame] * gain).abs() < 1e-6);
            }
            let bounds = effective.envelopes[track]
                .gain_bounds(rate as usize * 3, rate as usize * 13 / 4)
                .unwrap();
            if track == 0 {
                assert_eq!(bounds, (0.25, 0.25));
            }
        }
        let device =
            rust_app::PreparedAudio::prepare_for_rate(&project.source_paths(), 32000, &cancel)
                .unwrap();
        let device_pcm = device.read_frames(96000..96016).unwrap();
        let mut playback = device
            .playback(
                effective.envelopes.clone(),
                rust_app::AuditionMode::Original,
                0.5,
                rust_app::TICKS_PER_SECOND * 3,
                device.duration(),
            )
            .unwrap();
        playback.set_mode(rust_app::AuditionMode::Processed);
        playback.replace_envelopes(effective.envelopes.clone());
        assert_eq!(playback.position_frame(), 96000);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let sample = playback.next().unwrap();
            if playback.position_frame() == 96001 {
                let time = rust_app::TICKS_PER_SECOND * 3;
                let expected = (device_pcm[0][0] * effective.envelopes[0].gain_at_time(time)
                    + device_pcm[1][0] * effective.envelopes[1].gain_at_time(time))
                    * 0.5;
                assert!((sample - expected).abs() < 1e-6);
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        playback.set_mode(rust_app::AuditionMode::Removed);
        let position = playback.position_frame();
        playback.set_monitor(0.1).unwrap();
        playback.replace_envelopes(effective.envelopes.clone());
        assert_eq!(playback.position_frame(), position);
        assert_eq!(
            rust_app::SupervisedRenderSnapshot::capture(
                &project,
                &analysis,
                &effective,
                &evidence.runtime
            )
            .unwrap()
            .sound_identity,
            original_sound
        );
        drop(playback);
        drop(device);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn accepted_envelopes_render_directly_and_obsolete_private_completion_cannot_publish() {
    let (root, mut project, audio, evidence) = fixture::session();
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    let progress = std::sync::atomic::AtomicU8::new(0);
    let analysis = rust_app::EditorAnalysisWorker::default()
        .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancelled)
        .unwrap();
    let effective = analysis
        .install(
            &mut project,
            &mut rust_app::EffectiveEnvelopeCache::default(),
            Some(&evidence.runtime),
            None,
        )
        .unwrap();
    let original = std::fs::read(&project.sources[0].path).unwrap();
    let generation = rust_app::RenderGeneration::default();
    let snapshot = rust_app::SupervisedRenderSnapshot::capture(
        &project,
        &analysis,
        &effective,
        &evidence.runtime,
    )
    .unwrap();
    let destination = root.join("accepted-render");
    let prepared = rust_app::prepare_supervised_render(
        &project,
        &analysis,
        &effective,
        &snapshot,
        &destination,
        &progress,
        Some(&cancelled),
    )
    .unwrap();
    assert!(
        !destination.exists(),
        "Worker completion is private until a current publication gate"
    );
    let artifact = prepared
        .publish(
            &project,
            &evidence.runtime,
            &generation,
            generation.current(),
            Some(&cancelled),
        )
        .unwrap();
    artifact.accept(&mut project).unwrap();
    let reopened_review =
        rust_app::ReviewWorkspace::open(&destination.join("report.json"), &project.source_paths())
            .unwrap();
    assert_eq!(
        reopened_review.warnings,
        rust_app::ReviewWorkspace::from_ownership(&analysis.temporal).warnings
    );
    let mut rendered = bwavfile::WaveReader::open(destination.join("01-speaker-0.wav"))
        .unwrap()
        .audio_frame_reader()
        .unwrap();
    let mut source = bwavfile::WaveReader::open(&project.sources[0].path)
        .unwrap()
        .audio_frame_reader()
        .unwrap();
    let mut output = vec![0.0; 64000];
    let mut input = vec![0.0; 64000];
    rendered.read_frames(&mut output).unwrap();
    source.read_frames(&mut input).unwrap();
    for frame in 0..64000 {
        assert!(
            (output[frame] - input[frame] * effective.envelopes[0].gain_at(frame)).abs() < 1e-6
        );
    }
    let obsolete_destination = root.join("obsolete-render");
    let old_generation = generation.current();
    let prepared = rust_app::prepare_supervised_render(
        &project,
        &analysis,
        &effective,
        &snapshot,
        &obsolete_destination,
        &progress,
        None,
    )
    .unwrap();
    generation.invalidate();
    assert!(
        prepared
            .publish(
                &project,
                &evidence.runtime,
                &generation,
                old_generation,
                None
            )
            .is_err()
    );
    assert!(!obsolete_destination.exists());
    assert_eq!(std::fs::read(&project.sources[0].path).unwrap(), original);
    assert!(destination.join("01-speaker-0.wav").exists());
    let accepted_project = project.clone();
    for change in 0..4 {
        let failed_destination = root.join(format!("changed-{change}"));
        let prepared = rust_app::prepare_supervised_render(
            &project,
            &analysis,
            &effective,
            &snapshot,
            &failed_destination,
            &progress,
            None,
        )
        .unwrap();
        let mut runtime = evidence.runtime.clone();
        let cancelled = std::sync::atomic::AtomicBool::new(change == 0);
        match change {
            1 => runtime.recipe_sha256.push('0'),
            2 => {
                let draft = project
                    .region_gain_edit(
                        project.sources[0].track_id,
                        rust_app::TimeRange::new(
                            3 * rust_app::TICKS_PER_SECOND,
                            4 * rust_app::TICKS_PER_SECOND,
                        )
                        .unwrap(),
                        rust_app::GainRegionAction::Attenuate(0.2),
                    )
                    .unwrap();
                project.commit_gain_edit(draft).unwrap();
            }
            3 => {
                let mut changed = original.clone();
                *changed.last_mut().unwrap() ^= 1;
                std::fs::write(&project.sources[0].path, changed).unwrap();
            }
            _ => {}
        }
        assert!(
            prepared
                .publish(
                    &project,
                    &runtime,
                    &generation,
                    generation.current(),
                    Some(&cancelled)
                )
                .is_err()
        );
        assert!(!failed_destination.exists());
        assert!(destination.join("01-speaker-0.wav").exists());
        project = accepted_project.clone();
        project.save().unwrap();
    }
    std::fs::write(&project.sources[0].path, original).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn current_delivery_rejects_finite_rendered_payload_replacement_after_acceptance() {
    let (root, mut project, audio, evidence) = fixture::session();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    let analysis = rust_app::EditorAnalysisWorker::default()
        .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancelled)
        .unwrap();
    let effective = analysis
        .install(
            &mut project,
            &mut rust_app::EffectiveEnvelopeCache::default(),
            Some(&evidence.runtime),
            None,
        )
        .unwrap();
    let snapshot = rust_app::SupervisedRenderSnapshot::capture(
        &project,
        &analysis,
        &effective,
        &evidence.runtime,
    )
    .unwrap();
    let directory = root.join("render");
    let generation = rust_app::RenderGeneration::default();
    rust_app::prepare_supervised_render(
        &project, &analysis, &effective, &snapshot, &directory, &progress, None,
    )
    .unwrap()
    .publish(
        &project,
        &evidence.runtime,
        &generation,
        generation.current(),
        None,
    )
    .unwrap()
    .accept(&mut project)
    .unwrap();
    let reloaded = rust_app::Project::open(project.path()).unwrap();
    rust_app::validate_export(&reloaded).unwrap();
    let mut format = bwavfile::WaveFmt::new_pcm_mono(16000, 32);
    format.tag = 3;
    let mut writer = bwavfile::WaveWriter::create(directory.join("01-speaker-0.wav"), format)
        .unwrap()
        .audio_frame_writer()
        .unwrap();
    writer.write_frames(&vec![0.2; 64000]).unwrap();
    writer.end().unwrap();
    assert!(
        rust_app::validate_export(&reloaded).is_err(),
        "A valid same-grid WAV replacement is not the accepted deliverable"
    );
    assert!(rust_app::export(&reloaded, &root.join("delivery"), None).is_err());
    assert!(!root.join("delivery").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_acceptance_keeps_a_published_payload_that_was_modified_for_recovery() {
    let (root, mut project, audio, evidence) = fixture::session();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let analysis = rust_app::EditorAnalysisWorker::default()
        .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancel)
        .unwrap();
    let effective = analysis
        .install(
            &mut project,
            &mut rust_app::EffectiveEnvelopeCache::default(),
            Some(&evidence.runtime),
            None,
        )
        .unwrap();
    let snapshot = rust_app::SupervisedRenderSnapshot::capture(
        &project,
        &analysis,
        &effective,
        &evidence.runtime,
    )
    .unwrap();
    let destination = root.join("recovery");
    let generation = rust_app::RenderGeneration::default();
    let artifact = rust_app::prepare_supervised_render(
        &project,
        &analysis,
        &effective,
        &snapshot,
        &destination,
        &progress,
        None,
    )
    .unwrap()
    .publish(
        &project,
        &evidence.runtime,
        &generation,
        generation.current(),
        None,
    )
    .unwrap();
    let changed = destination.join("01-speaker-0.wav");
    std::fs::write(&changed, b"preserve this changed payload for recovery").unwrap();
    let mut parameters = project.parameters();
    parameters.attenuation_db = -12.0;
    project.set_parameters(parameters).unwrap();
    assert!(artifact.accept(&mut project).is_err());
    assert_eq!(
        std::fs::read(&changed).unwrap(),
        b"preserve this changed payload for recovery"
    );
    assert!(destination.join("report.json").exists());
    std::fs::remove_dir_all(root).unwrap();
}
