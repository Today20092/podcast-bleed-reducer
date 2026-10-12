use rust_app::{ExportEncoding, ExportLayout, ExportSettings, Project};

#[path = "common/supervised_fixture.rs"]
mod fixture;

#[test]
fn encoding_is_project_local_and_old_projects_default_to_preserve() {
    let (root, mut project, _) = fixture::session();
    assert_eq!(project.export_encoding(), ExportEncoding::PreserveSource);
    project.set_export_encoding(ExportEncoding::Pcm24).unwrap();
    assert_eq!(
        Project::open(project.path()).unwrap().export_encoding(),
        ExportEncoding::Pcm24
    );
    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(project.path()).unwrap()).unwrap();
    saved.as_object_mut().unwrap().remove("export_encoding");
    std::fs::write(project.path(), serde_json::to_vec(&saved).unwrap()).unwrap();
    assert_eq!(
        Project::open(project.path()).unwrap().export_encoding(),
        ExportEncoding::PreserveSource
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn estimates_count_every_native_frame_and_channel() {
    let (root, project, _) = fixture::session();
    let mut settings = ExportSettings {
        layout: ExportLayout::Multichannel,
        encoding: ExportEncoding::Pcm24,
    };
    assert_eq!(settings.estimated_audio_bytes(&project).unwrap(), 384_000);
    settings.encoding = ExportEncoding::Float32;
    assert_eq!(settings.estimated_audio_bytes(&project).unwrap(), 512_000);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn multichannel_matches_separate_float_render_in_editor_order() {
    let (root, mut project, audio) = fixture::session();
    prepare_current(&root, &mut project, &audio);
    let gate = rust_app::RenderGeneration::default();
    for layout in [ExportLayout::SeparateMono, ExportLayout::Multichannel] {
        let destination = root.join(format!("{layout:?}"));
        rust_app::prepare_export_with_settings(
            &project,
            &destination,
            ExportSettings {
                layout,
                encoding: ExportEncoding::Float32,
            },
            None,
        )
        .unwrap()
        .publish(&project, &gate, gate.current(), None)
        .unwrap();
    }
    let mut combined =
        bwavfile::WaveReader::open(root.join("Multichannel/session-clean.wav")).unwrap();
    assert_eq!(combined.format().unwrap().channel_count, 2);
    assert_eq!(combined.frame_length().unwrap(), 64_000);
    let mut joined = vec![0.0_f32; 128_000];
    combined
        .audio_frame_reader()
        .unwrap()
        .read_frames(&mut joined)
        .unwrap();
    for channel in 0..2 {
        let reader = bwavfile::WaveReader::open(
            root.join(format!("SeparateMono/speaker-{channel}-clean.wav")),
        )
        .unwrap();
        let mut mono = vec![0.0_f32; 64_000];
        reader
            .audio_frame_reader()
            .unwrap()
            .read_frames(&mut mono)
            .unwrap();
        assert!(
            mono.iter()
                .zip(joined.as_chunks::<2>().0.iter())
                .all(|(sample, frame)| *sample == frame[channel])
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

fn prepare_current(
    root: &std::path::Path,
    project: &mut Project,
    audio: &[rust_app::PreparedAnalysisAudio],
) {
    let progress = std::sync::atomic::AtomicU8::new(0);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let analysis = rust_app::EditorAnalysisWorker::default()
        .analyze_acoustic_evidence(project, audio, &progress, &cancel)
        .unwrap();
    let effective = analysis
        .install(
            project,
            &mut rust_app::EffectiveEnvelopeCache::default(),
            None,
        )
        .unwrap();
    let snapshot =
        rust_app::SupervisedRenderSnapshot::capture(project, &analysis, &effective).unwrap();
    let gate = rust_app::RenderGeneration::default();
    rust_app::prepare_supervised_float_render(
        project,
        &analysis,
        &effective,
        &snapshot,
        &root.join("render"),
        &progress,
        None,
    )
    .unwrap()
    .publish(project, &gate, gate.current(), None)
    .unwrap()
    .accept(project)
    .unwrap();
}

#[test]
fn layout_blocks_mixed_rates_grids_encodings_and_oversized_riff() {
    let (root, mut project, _) = fixture::session_at_rates([16_000, 48_000]);
    let settings = ExportSettings {
        layout: ExportLayout::Multichannel,
        encoding: ExportEncoding::PreserveSource,
    };
    assert!(
        settings
            .validate_layout(&project)
            .unwrap_err()
            .to_string()
            .contains("sample rates")
    );
    project.sources[1].metadata.grid = project.sources[0].metadata.grid;
    project.sources[1].metadata.grid.frame_count += 1;
    assert!(
        settings
            .validate_layout(&project)
            .unwrap_err()
            .to_string()
            .contains("frame counts")
    );
    project.sources[1].metadata.grid = project.sources[0].metadata.grid;
    project.sources[1].metadata.timing_origin = Some(1);
    assert!(settings.validate_layout(&project).is_err());
    std::fs::remove_dir_all(root).unwrap();

    let (root, mut project, _) = fixture::session();
    for source in &mut project.sources {
        source.metadata.grid.frame_count = 600_000_000;
    }
    assert!(
        settings
            .validate_layout(&project)
            .unwrap_err()
            .to_string()
            .contains("RIFF")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_pcm24_delivery_can_fit_when_its_private_float_intermediate_exceeds_riff() {
    let (root, mut project, _) = profile_session(["pcm24", "pcm24"], false);
    for source in &mut project.sources {
        source.metadata.grid.frame_count = 1_100_000_000;
    }
    let settings = ExportSettings {
        layout: ExportLayout::SeparateMono,
        encoding: ExportEncoding::Pcm24,
    };
    settings.validate_layout(&project).unwrap();
    assert_eq!(
        settings.estimated_audio_bytes(&project).unwrap(),
        6_600_000_000
    );
    assert!(
        ExportSettings {
            encoding: ExportEncoding::Float32,
            ..settings
        }
        .validate_layout(&project)
        .unwrap_err()
        .to_string()
        .contains("RIFF")
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn profile_session(
    profiles: [&str; 2],
    overload: bool,
) -> (
    std::path::PathBuf,
    Project,
    Vec<rust_app::PreparedAnalysisAudio>,
) {
    let (root, old, _) = fixture::session();
    let mut paths = Vec::new();
    for (i, profile) in profiles.into_iter().enumerate() {
        let reader = bwavfile::WaveReader::open(&old.sources[i].path).unwrap();
        let mut samples = vec![0.0_f32; 64_000];
        reader
            .audio_frame_reader()
            .unwrap()
            .read_frames(&mut samples)
            .unwrap();
        samples[..64].fill(0.0);
        if overload {
            samples[64] = 1.25;
            samples[65] = -1.25;
        }
        let path = root.join(format!(
            "track-{i}-{}.{}",
            profile,
            if profile == "aiff" { "aiff" } else { "wav" }
        ));
        if profile == "float64" {
            let bytes = (samples.len() * 8) as u32;
            let mut file = Vec::new();
            file.extend(b"RIFF");
            file.extend((36 + bytes).to_le_bytes());
            file.extend(b"WAVEfmt ");
            file.extend(16_u32.to_le_bytes());
            file.extend(3_u16.to_le_bytes());
            file.extend(1_u16.to_le_bytes());
            file.extend(16_000_u32.to_le_bytes());
            file.extend(128_000_u32.to_le_bytes());
            file.extend(8_u16.to_le_bytes());
            file.extend(64_u16.to_le_bytes());
            file.extend(b"data");
            file.extend(bytes.to_le_bytes());
            for sample in samples {
                file.extend(f64::from(sample).to_le_bytes());
            }
            std::fs::write(&path, file).unwrap();
        } else if profile == "aiff" {
            let bytes = (samples.len() * 2) as u32;
            let mut file = Vec::new();
            file.extend(b"FORM");
            file.extend((46 + bytes).to_be_bytes());
            file.extend(b"AIFFCOMM");
            file.extend(18_u32.to_be_bytes());
            file.extend(1_u16.to_be_bytes());
            file.extend((samples.len() as u32).to_be_bytes());
            file.extend(16_u16.to_be_bytes());
            file.extend([0x40, 0x0c, 0xfa, 0, 0, 0, 0, 0, 0, 0]);
            file.extend(b"SSND");
            file.extend((8 + bytes).to_be_bytes());
            file.extend([0; 8]);
            for sample in samples {
                file.extend(((sample * 32768.0).round() as i16).to_be_bytes());
            }
            std::fs::write(&path, file).unwrap();
        } else {
            let bits = match profile {
                "pcm16" => 16,
                "pcm24" => 24,
                _ => 32,
            };
            let mut format = bwavfile::WaveFmt::new_pcm_mono(16_000, bits);
            if profile == "float32" {
                format.tag = 3;
            }
            let mut writer = bwavfile::WaveWriter::create(&path, format)
                .unwrap()
                .audio_frame_writer()
                .unwrap();
            writer.write_frames(&samples).unwrap();
            writer.end().unwrap();
        }
        paths.push(path);
    }
    let mut project = Project::create(root.join("profiles.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    for label in old.reference_labels() {
        project
            .add_reference_label(label.range, label.active_speakers.clone(), &[])
            .unwrap();
    }
    project
        .protect_range(0, 4 * rust_app::TICKS_PER_SECOND)
        .unwrap();
    let audio = project
        .source_paths()
        .iter()
        .map(|p| {
            rust_app::prepare_analysis_audio_in(p, &root.join("profile-copies"), None).unwrap()
        })
        .collect();
    (root, project, audio)
}

fn read_samples(path: &std::path::Path) -> (bwavfile::WaveFmt, Vec<f32>) {
    let mut reader = bwavfile::WaveReader::open(path).unwrap();
    let format = reader.format().unwrap();
    let mut samples =
        vec![0.0_f32; reader.frame_length().unwrap() as usize * usize::from(format.channel_count)];
    reader
        .audio_frame_reader()
        .unwrap()
        .read_frames(&mut samples)
        .unwrap();
    (format, samples)
}

#[test]
fn explicit_mute_remains_exact_silence_in_integer_float_and_multichannel_delivery() {
    let (root, mut project, audio) = fixture::session();
    project
        .set_parameters(rust_app::AutomixParameters {
            mute: true,
            open_ms: 0.0,
            close_ms: 0.0,
            minimum_gap_ms: 0.0,
            ..project.parameters()
        })
        .unwrap();
    prepare_current(&root, &mut project, &audio);
    let gate = rust_app::RenderGeneration::default();
    for layout in [ExportLayout::SeparateMono, ExportLayout::Multichannel] {
        for encoding in [ExportEncoding::Pcm24, ExportEncoding::Float32] {
            let destination = root.join(format!("mute-{layout:?}-{encoding:?}"));
            rust_app::prepare_export_with_settings(
                &project,
                &destination,
                ExportSettings { layout, encoding },
                None,
            )
            .unwrap()
            .publish(&project, &gate, gate.current(), None)
            .unwrap();
            let paths = if layout == ExportLayout::Multichannel {
                vec![destination.join("session-clean.wav")]
            } else {
                vec![
                    destination.join("speaker-0-clean.wav"),
                    destination.join("speaker-1-clean.wav"),
                ]
            };
            for path in paths {
                let (format, samples) = read_samples(&path);
                let count = 4000 * usize::from(format.channel_count);
                assert!(
                    samples[..count].iter().all(|s| *s == 0.0),
                    "mute room-tone plateau: {path:?}"
                );
                assert!(
                    samples[count..].iter().any(|s| *s != 0.0),
                    "known speech stays audible"
                );
            }
        }
    }
    project
        .set_parameters(rust_app::AutomixParameters {
            mute: false,
            attenuation_db: -120.0,
            ..project.parameters()
        })
        .unwrap();
    assert!(
        rust_app::prepare_export_with_settings(
            &project,
            &root.join("stale"),
            ExportSettings {
                layout: ExportLayout::SeparateMono,
                encoding: ExportEncoding::Float32
            },
            None
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn separate_encodings_preserve_profiles_and_overrides_with_one_bounded_dither() {
    for (profiles, expected) in [
        (["pcm16", "pcm24"], [(1, 16), (1, 24)]),
        (["pcm32", "float32"], [(1, 32), (3, 32)]),
        (["float64", "aiff"], [(3, 32), (3, 32)]),
    ] {
        let (root, mut project, audio) = profile_session(profiles, false);
        prepare_current(&root, &mut project, &audio);
        let gate = rust_app::RenderGeneration::default();
        for encoding in [
            ExportEncoding::PreserveSource,
            ExportEncoding::Pcm24,
            ExportEncoding::Float32,
        ] {
            let destination = root.join(format!("{encoding:?}"));
            let settings = ExportSettings {
                layout: ExportLayout::SeparateMono,
                encoding,
            };
            rust_app::prepare_export_with_settings(&project, &destination, settings, None)
                .unwrap()
                .publish(&project, &gate, gate.current(), None)
                .unwrap();
            for (i, &(tag, bits)) in expected.iter().enumerate() {
                let stem = project.sources[i]
                    .path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let (format, delivered) =
                    read_samples(&destination.join(format!("{stem}-clean.wav")));
                let expected = match encoding {
                    ExportEncoding::PreserveSource => (tag, bits),
                    ExportEncoding::Pcm24 => (1, 24),
                    ExportEncoding::Float32 => (3, 32),
                };
                assert_eq!((format.tag, format.bits_per_sample), expected);
                assert_eq!(format.sample_rate, 16_000);
                assert_eq!(delivered.len(), 64_000);
                assert!(delivered[..64].iter().all(|s| *s == 0.0));
                let (_, original) =
                    read_samples(&root.join(format!("render/{:02}-{stem}.wav", i + 1)));
                if format.tag == 3 {
                    assert_eq!(delivered, original);
                } else {
                    let lsb = 2.0_f64.powi(1 - i32::from(format.bits_per_sample));
                    assert!(
                        delivered
                            .iter()
                            .zip(&original)
                            .all(|(a, b)| (f64::from(*a) - f64::from(*b)).abs()
                                <= 1.5 * lsb + f64::from(f32::EPSILON))
                    );
                    if format.bits_per_sample < 32 {
                        assert_ne!(delivered, original);
                    }
                }
            }
        }
        let preserve = ExportSettings {
            layout: ExportLayout::Multichannel,
            encoding: ExportEncoding::PreserveSource,
        };
        if expected[0] != expected[1] {
            assert!(
                preserve
                    .validate_layout(&project)
                    .unwrap_err()
                    .to_string()
                    .contains("encodings differ")
            );
        } else {
            preserve.validate_layout(&project).unwrap();
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn integer_overload_blocks_both_layouts_and_float_retry_keeps_levels() {
    let (root, mut project, audio) = profile_session(["float32", "float32"], true);
    prepare_current(&root, &mut project, &audio);
    let gate = rust_app::RenderGeneration::default();
    for layout in [ExportLayout::SeparateMono, ExportLayout::Multichannel] {
        let destination = root.join(format!("overload-{layout:?}"));
        let settings = ExportSettings {
            layout,
            encoding: ExportEncoding::Pcm24,
        };
        let error = rust_app::prepare_export_with_settings(&project, &destination, settings, None)
            .err()
            .unwrap();
        assert!(error.to_string().contains("32-bit float"));
        assert!(!destination.exists());
        let settings = ExportSettings {
            encoding: ExportEncoding::Float32,
            ..settings
        };
        rust_app::prepare_export_with_settings(&project, &destination, settings, None)
            .unwrap()
            .publish(&project, &gate, gate.current(), None)
            .unwrap();
        let name = if layout == ExportLayout::Multichannel {
            "session-clean.wav"
        } else {
            "track-0-float32-clean.wav"
        };
        let (_, samples) = read_samples(&destination.join(name));
        let channels = if layout == ExportLayout::Multichannel {
            2
        } else {
            1
        };
        assert_eq!(samples[64 * channels], 1.25);
        assert_eq!(samples[65 * channels], -1.25);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn both_layouts_reject_obsolete_tampered_cancelled_or_colliding_deliveries() {
    use std::sync::atomic::AtomicBool;
    let (root, mut current, audio) = fixture::session();
    prepare_current(&root, &mut current, &audio);
    let gate = rust_app::RenderGeneration::default();
    for layout in [ExportLayout::SeparateMono, ExportLayout::Multichannel] {
        let settings = ExportSettings {
            layout,
            encoding: ExportEncoding::Float32,
        };
        let early = root.join(format!("early-{layout:?}"));
        assert!(
            rust_app::prepare_export_with_settings(
                &current,
                &early,
                settings,
                Some(&AtomicBool::new(true))
            )
            .is_err()
        );
        assert!(!early.exists());
        for kind in 0..8 {
            let project = Project::open(current.path()).unwrap();
            let destination = root.join(format!("blocked-{layout:?}-{kind}"));
            let expected = gate.current();
            let private =
                rust_app::prepare_export_with_settings(&project, &destination, settings, None)
                    .unwrap();
            let mut restore = None;
            match kind {
                1 => gate.invalidate(),
                2 => {
                    let mut edited = project.clone();
                    edited.set_export_encoding(ExportEncoding::Pcm24).unwrap();
                }
                3 => {
                    let prefix = format!(".blocked-{layout:?}-{kind}.");
                    let staging = std::fs::read_dir(&root)
                        .unwrap()
                        .map(|p| p.unwrap().path())
                        .find(|p| {
                            p.file_name()
                                .unwrap()
                                .to_string_lossy()
                                .starts_with(&prefix)
                        })
                        .unwrap();
                    let name = if layout == ExportLayout::Multichannel {
                        "session-clean.wav"
                    } else {
                        "speaker-0-clean.wav"
                    };
                    std::fs::write(staging.join(name), b"tampered").unwrap();
                }
                4..=6 => {
                    let path = match kind {
                        4 => root.join("render/01-speaker-0.wav"),
                        5 => project.sources[0].path.clone(),
                        _ => root.join("render/report.json"),
                    };
                    let bytes = std::fs::read(&path).unwrap();
                    std::fs::write(&path, b"changed").unwrap();
                    restore = Some((path, bytes));
                }
                7 => {
                    std::fs::create_dir(&destination).unwrap();
                    std::fs::write(destination.join("keep"), b"keep").unwrap();
                }
                _ => {}
            }
            assert!(
                private
                    .publish(&project, &gate, expected, Some(&AtomicBool::new(kind == 0)))
                    .is_err(),
                "{layout:?} failure {kind}"
            );
            if kind == 7 {
                assert_eq!(std::fs::read(destination.join("keep")).unwrap(), b"keep");
            } else {
                assert!(!destination.exists());
            }
            if let Some((path, bytes)) = restore {
                std::fs::write(path, bytes).unwrap();
            }
            if kind == 2 {
                current
                    .set_export_encoding(ExportEncoding::PreserveSource)
                    .unwrap();
            }
        }
    }
    assert!(
        !std::fs::read_dir(&root).unwrap().any(|p| p
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp"))
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn shared_integer_encodings_and_explicit_overrides_match_channel_for_channel() {
    for profile in ["pcm16", "pcm24"] {
        let (root, mut project, audio) = profile_session([profile, profile], false);
        prepare_current(&root, &mut project, &audio);
        let gate = rust_app::RenderGeneration::default();
        for encoding in [
            ExportEncoding::PreserveSource,
            ExportEncoding::Pcm24,
            ExportEncoding::Float32,
        ] {
            for layout in [ExportLayout::SeparateMono, ExportLayout::Multichannel] {
                let settings = ExportSettings { encoding, layout };
                rust_app::prepare_export_with_settings(
                    &project,
                    &root.join(format!("{encoding:?}-{layout:?}")),
                    settings,
                    None,
                )
                .unwrap()
                .publish(&project, &gate, gate.current(), None)
                .unwrap();
            }
            let (format, joined) =
                read_samples(&root.join(format!("{encoding:?}-Multichannel/session-clean.wav")));
            assert_eq!(format.channel_count, 2);
            for channel in 0..2 {
                let (_, separate) = read_samples(&root.join(format!(
                    "{encoding:?}-SeparateMono/track-{channel}-{profile}-clean.wav"
                )));
                assert!(
                    separate
                        .iter()
                        .zip(joined.as_chunks::<2>().0.iter())
                        .all(|(sample, frame)| *sample == frame[channel])
                );
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn selected_wav_channels_export_in_editor_order_with_compatible_broadcast_timing() {
    let (root, old, _) = fixture::session();
    let channels: Vec<_> = old
        .source_paths()
        .iter()
        .map(|p| read_samples(p).1)
        .collect();
    let path = root.join("broadcast.wav");
    let mut format = bwavfile::WaveFmt::new_pcm_stereo(16_000, 32);
    format.tag = 3;
    let mut writer = bwavfile::WaveWriter::create(&path, format).unwrap();
    let metadata = bwavfile::Bext {
        description: "Recorder channels".into(),
        originator: "CleanSlate test".into(),
        originator_reference: "fixture".into(),
        origination_date: "2026-10-11".into(),
        origination_time: "12:00:00".into(),
        time_reference: 16_000,
        version: 0,
        umid: None,
        loudness_value: None,
        loudness_range: None,
        max_true_peak_level: None,
        max_momentary_loudness: None,
        max_short_term_loudness: None,
        coding_history: "A=PCM,F=16000".into(),
    };
    writer.write_broadcast_metadata(&metadata).unwrap();
    let mut frames = writer.audio_frame_writer().unwrap();
    let interleaved: Vec<_> = channels[0]
        .iter()
        .zip(&channels[1])
        .flat_map(|(&a, &b)| [b, a])
        .collect();
    frames.write_frames(&interleaved).unwrap();
    frames.end().unwrap();
    let sources = [1, 0].map(|channel| rust_app::AudioSource {
        path: path.clone(),
        channel: Some(channel),
        name: Some(format!("speaker-{}", 1 - channel)),
    });
    let mut project =
        Project::create_from_sources(root.join("channels.cleanslate"), sources.to_vec()).unwrap();
    for label in old.reference_labels() {
        project
            .add_reference_label(label.range, label.active_speakers.clone(), &[])
            .unwrap();
    }
    project
        .protect_range(0, 4 * rust_app::TICKS_PER_SECOND)
        .unwrap();
    let audio: Vec<_> = sources
        .iter()
        .map(|s| {
            rust_app::prepare_source_analysis_audio_in(s, &root.join("channel-copies"), None)
                .unwrap()
        })
        .collect();
    prepare_current(&root, &mut project, &audio);
    let gate = rust_app::RenderGeneration::default();
    for layout in [ExportLayout::SeparateMono, ExportLayout::Multichannel] {
        let settings = ExportSettings {
            layout,
            encoding: ExportEncoding::PreserveSource,
        };
        rust_app::prepare_export_with_settings(
            &project,
            &root.join(format!("{layout:?}")),
            settings,
            None,
        )
        .unwrap()
        .publish(&project, &gate, gate.current(), None)
        .unwrap();
    }
    let mut reader =
        bwavfile::WaveReader::open(root.join("Multichannel/session-clean.wav")).unwrap();
    assert_eq!(
        reader
            .broadcast_extension()
            .unwrap()
            .unwrap()
            .time_reference,
        16_000
    );
    let (_, joined) = read_samples(&root.join("Multichannel/session-clean.wav"));
    for (i, channel) in [2, 1].into_iter().enumerate() {
        let (_, separate) =
            read_samples(&root.join(format!("SeparateMono/broadcast-ch{channel}-clean.wav")));
        assert_eq!(separate, channels[i]);
        assert!(
            separate
                .iter()
                .zip(joined.as_chunks::<2>().0.iter())
                .all(|(s, f)| *s == f[i])
        );
    }
    // Same verified timestamp, conflicting descriptive BWF metadata still cannot be copied into one file.
    let other = root.join("other.wav");
    let mut writer =
        bwavfile::WaveWriter::create(&other, bwavfile::WaveFmt::new_pcm_mono(16_000, 16)).unwrap();
    let mut conflicting = metadata;
    conflicting.description = "Other recorder".into();
    writer.write_broadcast_metadata(&conflicting).unwrap();
    let mut writer = writer.audio_frame_writer().unwrap();
    writer.write_frames(&channels[0]).unwrap();
    writer.end().unwrap();
    let mut inputs = vec![sources[0].clone()];
    inputs.push(rust_app::AudioSource::mono(other));
    let project = Project::create_from_sources(root.join("conflict.cleanslate"), inputs).unwrap();
    let error = ExportSettings {
        layout: ExportLayout::Multichannel,
        encoding: ExportEncoding::Float32,
    }
    .validate_layout(&project)
    .unwrap_err();
    assert!(error.to_string().contains("metadata conflicts"));
    std::fs::remove_dir_all(root).unwrap();
}
