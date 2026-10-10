#[path = "common/supervised_fixture.rs"]
mod fixture;

#[test]
fn multichannel_sources_train_and_deliver_the_same_native_samples_as_audition() {
    use rust_app::{Project, TICKS_PER_SECOND, TimeRange};
    use std::sync::atomic::{AtomicBool, AtomicU8};
    let (root, mono_project, _) = fixture::session();
    let mut signals = Vec::new();
    for source in &mono_project.sources {
        let mut reader = bwavfile::WaveReader::open(&source.path)
            .unwrap()
            .audio_frame_reader()
            .unwrap();
        let mut samples = vec![0_f32; 64_000];
        assert_eq!(reader.read_frames(&mut samples).unwrap(), 64_000);
        signals.push(samples);
    }
    let poly = root.join("recorder.wav");
    let mut format = bwavfile::WaveFmt::new_pcm_mono(16_000, 32);
    format.tag = 3;
    format.channel_count = 2;
    format.block_alignment *= 2;
    format.bytes_per_second *= 2;
    let mut writer = bwavfile::WaveWriter::create(&poly, format)
        .unwrap()
        .audio_frame_writer()
        .unwrap();
    let interleaved: Vec<_> = (0..64_000)
        .flat_map(|frame| [signals[0][frame], signals[1][frame]])
        .collect();
    writer.write_frames(&interleaved).unwrap();
    writer.end().unwrap();
    let original = std::fs::read(&poly).unwrap();
    let mut project = Project::create_from_sources(
        root.join("poly.cleanslate"),
        rust_app::wav_channel_sources(&poly).unwrap(),
    )
    .unwrap();
    project.require_alignment().unwrap();
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    for (track, (start, end)) in [
        (TICKS_PER_SECOND / 2, TICKS_PER_SECOND * 3 / 2),
        (2 * TICKS_PER_SECOND, TICKS_PER_SECOND * 11 / 4),
    ]
    .into_iter()
    .enumerate()
    {
        project
            .add_reference_label(
                TimeRange::new(start, end).unwrap(),
                vec![project.sources[track].speaker_id],
                &[],
            )
            .unwrap();
    }
    let progress = AtomicU8::new(0);
    let cancel = AtomicBool::new(false);
    let analysis = rust_app::EditorAnalysisWorker::default()
        .analyze(&project, &root.join("poly-cache"), &progress, &cancel)
        .unwrap();
    let effective = analysis
        .install(
            &mut project,
            &mut rust_app::EffectiveEnvelopeCache::default(),
            None,
        )
        .unwrap();
    let snapshot =
        rust_app::SupervisedRenderSnapshot::capture(&project, &analysis, &effective).unwrap();
    let destination = root.join("delivery");
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
    .publish(&project, &gate, gate.current(), None)
    .unwrap()
    .accept(&mut project)
    .unwrap();
    let prepared = rust_app::PreparedAudio::prepare_sources_for_rate(
        &project.audio_sources(),
        None,
        16_000,
        &cancel,
    )
    .unwrap();
    for (track, signal) in signals.iter().enumerate() {
        let delivered = destination.join(format!("{:02}-Channel {}.wav", track + 1, track + 1));
        let metadata = rust_app::SourceMetadata::read(&delivered).unwrap();
        assert_eq!(metadata, project.sources[track].metadata);
        let mut reader = bwavfile::WaveReader::open(delivered)
            .unwrap()
            .audio_frame_reader()
            .unwrap();
        let mut samples = vec![0_f32; 64_000];
        reader.read_frames(&mut samples).unwrap();
        let audition = prepared.read_frames(0..64_000).unwrap();
        assert_eq!(&audition[track], signal);
        for (frame, (&sample, &input)) in samples.iter().zip(signal).enumerate() {
            let time = metadata
                .grid
                .time_at_frame(frame as u64, rust_app::Rounding::Floor)
                .unwrap();
            assert!(
                (sample - input * effective.envelopes[track].gain_at_time(time)).abs() < 0.000001
            );
        }
    }
    assert_eq!(std::fs::read(poly).unwrap(), original);
    Project::open(project.path())
        .unwrap()
        .validate_sources()
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
