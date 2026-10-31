use super::*;
use nice_plug::context::process::SendEventError;
use nice_plug::midi::PluginNoteEvent;
use nice_plug::plugin::ParamValue;
use std::collections::BTreeMap;

#[test]
fn a_new_experimental_gate_starts_bypassed() {
    let gate = SmartGate::default();
    assert!(gate.params.bypass.value());
}

#[test]
fn experimental_arming_requires_a_prepared_active_gate() {
    let mut gate = SmartGate::default();
    gate.arm_experimental();
    assert!(
        !gate.experimental_armed(),
        "an unprepared gate cannot accept experimental arming"
    );
    assert!(gate.activate(
        &SmartGate::AUDIO_IO_LAYOUTS[1],
        &BufferConfig {
            sample_rate: 48_000.0,
            min_buffer_size: None,
            max_buffer_size: 1536,
            process_mode: ProcessMode::Offline,
        },
        &mut ActivationHost
    ));
    assert!(!gate.experimental_armed(), "activation is not consent");
    gate.arm_experimental();
    assert!(gate.experimental_armed());
    gate.deactivate();
    assert!(!gate.experimental_armed());
    gate.arm_experimental();
    assert!(
        !gate.experimental_armed(),
        "deactivated gate must require successful preparation before rearming"
    );
}

#[test]
fn editor_arming_requires_a_click_and_keeps_bypass_enabled() {
    let mut gate = SmartGate::default();
    assert!(gate.activate(
        &SmartGate::AUDIO_IO_LAYOUTS[1],
        &BufferConfig {
            sample_rate: 48_000.0,
            min_buffer_size: None,
            max_buffer_size: 1536,
            process_mode: ProcessMode::Offline,
        },
        &mut ActivationHost
    ));
    let context = egui::Context::default();
    let mut panel = editor::Panel::new(Arc::clone(&gate.params), Arc::clone(&gate.consent));
    let output = draw_gate_panel(&context, &mut panel, Vec::new());
    assert!(
        !gate.experimental_armed(),
        "opening the editor is not consent"
    );
    let button = output.shapes.iter().find_map(|shape| match &shape.shape {
        egui::Shape::Text(text) if text.galley.job.text == "Arm experimental gate" => {
            Some(text.pos + text.galley.size() / 2.0)
        }
        _ => None,
    });
    let position = button.expect("the actual editor must expose explicit experimental arming");
    for pressed in [true, false] {
        draw_gate_panel(
            &context,
            &mut panel,
            vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    assert!(
        gate.experimental_armed(),
        "the explicit editor click must arm"
    );
    assert!(
        gate.params.bypass.value(),
        "consent must not disable bypass"
    );
    let output = draw_gate_panel(&context, &mut panel, Vec::new());
    assert!(output.shapes.iter().any(|shape| matches!(
        &shape.shape,
        egui::Shape::Text(text) if text.galley.job.text == "Bypassed · original audio"
    )));
    for _ in 0..2 {
        for pressed in [true, false] {
            draw_gate_panel(
                &context,
                &mut panel,
                vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
    }
    for expected in [false, true] {
        for pressed in [true, false] {
            draw_gate_panel(
                &context,
                &mut panel,
                vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
        assert_eq!(
            gate.experimental_armed(),
            expected,
            "focused Enter toggles consent"
        );
    }
    gate.deactivate();
    let output = draw_gate_panel(&context, &mut panel, Vec::new());
    assert!(
        output.shapes.iter().any(|shape| matches!(
            &shape.shape,
            egui::Shape::Text(text) if text.galley.job.text == "Not prepared · original audio"
        )),
        "the editor must report unavailable preparation after deactivation"
    );
}

#[test]
fn editor_lifecycle_interruptions_cancel_an_in_flight_arming_click() {
    for interruption in ["reset", "reactivate"] {
        let mut gate = SmartGate::default();
        let config = BufferConfig {
            sample_rate: 48_000.0,
            min_buffer_size: None,
            max_buffer_size: 1536,
            process_mode: ProcessMode::Offline,
        };
        assert!(gate.activate(
            &SmartGate::AUDIO_IO_LAYOUTS[1],
            &config,
            &mut ActivationHost
        ));
        let context = egui::Context::default();
        let mut panel = editor::Panel::new(Arc::clone(&gate.params), Arc::clone(&gate.consent));
        let output = draw_gate_panel(&context, &mut panel, Vec::new());
        let position = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "Arm experimental gate" => {
                    Some(text.pos + text.galley.size() / 2.0)
                }
                _ => None,
            })
            .expect("explicit arming control");
        let pointer = |pressed| {
            vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        };
        draw_gate_panel(&context, &mut panel, pointer(true));
        if interruption == "reset" {
            gate.reset();
        } else {
            gate.deactivate();
            assert!(gate.activate(
                &SmartGate::AUDIO_IO_LAYOUTS[1],
                &config,
                &mut ActivationHost
            ));
        }
        draw_gate_panel(&context, &mut panel, pointer(false));
        assert!(
            !gate.experimental_armed(),
            "{interruption} must retire a gesture begun before the lifecycle change"
        );
        for pressed in [true, false] {
            draw_gate_panel(&context, &mut panel, pointer(pressed));
        }
        assert!(
            gate.experimental_armed(),
            "a fresh click after {interruption} may arm"
        );
    }
}

#[test]
fn editor_reports_original_fallback_when_an_armed_callback_faults() {
    let mut gate = SmartGate::default();
    Arc::get_mut(&mut gate.params).unwrap().bypass =
        BoolParam::new("Bypass", false).make_bypass();
    assert!(gate.activate(
        &SmartGate::AUDIO_IO_LAYOUTS[1],
        &BufferConfig {
            sample_rate: 48_000.0,
            min_buffer_size: None,
            max_buffer_size: 1536,
            process_mode: ProcessMode::Offline,
        },
        &mut ActivationHost
    ));
    gate.arm_experimental();
    let mut audio = vec![vec![f32::from_bits(0x7fc0_1234)]];
    process_test_block(&mut gate, &mut audio);
    assert_eq!(audio[0][0].to_bits(), 0x7fc0_1234);
    assert!(
        gate.experimental_armed(),
        "consent is separate from processing health"
    );
    let context = egui::Context::default();
    let mut panel = editor::Panel::new(Arc::clone(&gate.params), Arc::clone(&gate.consent));
    let output = draw_gate_panel(&context, &mut panel, Vec::new());
    assert!(
        output.shapes.iter().any(|shape| matches!(
            &shape.shape,
            egui::Shape::Text(text) if text.galley.job.text == "Processing unavailable · original audio"
        )),
        "the editor must report callback fallback, rather than only armed consent"
    );
}

fn draw_gate_panel(
    context: &egui::Context,
    panel: &mut editor::Panel,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 440.0),
            )),
            events,
            ..Default::default()
        },
        |ui| panel.show(ui),
    );
    output.textures_delta.clear();
    output
}

#[test]
fn actual_gate_editor_states_remain_visible_at_declared_size() {
    for (name, expected) in [
        ("unprepared", "Not prepared · original audio"),
        ("bypassed", "Bypassed · original audio"),
        ("armed", "Armed · attenuation requires valid evidence"),
    ] {
        let mut gate = SmartGate::default();
        if name != "unprepared" {
            if name == "armed" {
                Arc::get_mut(&mut gate.params).unwrap().bypass =
                    BoolParam::new("Bypass", false).make_bypass();
            }
            assert!(gate.activate(
                &SmartGate::AUDIO_IO_LAYOUTS[1],
                &BufferConfig {
                    sample_rate: 48_000.0,
                    min_buffer_size: None,
                    max_buffer_size: 1536,
                    process_mode: ProcessMode::Offline,
                },
                &mut ActivationHost
            ));
            if name == "armed" {
                gate.arm_experimental();
            }
        }
        let context = egui::Context::default();
        editor::style(&context);
        let mut panel = editor::Panel::new(Arc::clone(&gate.params), Arc::clone(&gate.consent));
        let mut content = egui::Rect::NOTHING;
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 440.0),
                )),
                ..Default::default()
            },
            |ui| {
                panel.show(ui);
                content = ui.min_rect();
            },
        );
        let status = output.shapes.iter().any(|shape| matches!(
            &shape.shape,
            egui::Shape::Text(text) if text.galley.job.text == expected
                && shape.clip_rect.contains_rect(egui::Rect::from_min_size(text.pos, text.galley.size()))
        ));
        if let Ok(directory) = std::env::var("CLEANSLATE_PLUGIN_CAPTURES") {
            std::fs::create_dir_all(&directory).unwrap();
            crate::editor_capture::write_capture(
                &context,
                &mut output,
                &std::path::Path::new(&directory).join(format!("gate-{name}-640x440.ppm")),
                640,
                440,
            )
            .unwrap();
        } else {
            output.textures_delta.clear();
        }
        assert!(content.right() <= 640.0 && content.bottom() <= 440.0);
        assert!(
            status,
            "{name} status must be drawn within the declared editor size"
        );
    }
}

#[test]
fn restoring_any_saved_gate_state_requires_fresh_explicit_arming() {
    for schema in ["1", "2", "999", "\"invalid\""] {
        let mut saved = PluginState {
            version: SmartGate::VERSION.into(),
            params: BTreeMap::from([(BYPASS_ID.into(), ParamValue::Bool(false))]),
            fields: BTreeMap::from([(STATE_KEY.into(), schema.into())]),
        };
        SmartGate::filter_state(&mut saved);
        assert!(matches!(saved.params[BYPASS_ID], ParamValue::Bool(true)));
    }
}

#[test]
fn experimental_arming_is_explicit_temporary_and_cleared_by_reset() {
    let mut gate = SmartGate::default();
    assert!(gate.activate(
        &SmartGate::AUDIO_IO_LAYOUTS[1],
        &BufferConfig {
            sample_rate: 48_000.0,
            min_buffer_size: None,
            max_buffer_size: 1536,
            process_mode: ProcessMode::Offline,
        },
        &mut ActivationHost
    ));
    assert!(!gate.experimental_armed());
    gate.arm_experimental();
    assert!(gate.experimental_armed());
    assert_eq!(gate.params.serialize_fields().len(), 1);
    gate.reset();
    assert!(!gate.experimental_armed());
}

#[test]
fn fresh_absence_can_attenuate_but_expiring_evidence_returns_to_original() {
    use crate::gain::GainPolicy;
    use cleanslate_gate_detector::Event;

    let mut gain = GainPolicy::new(48_000).unwrap();
    gain.reset(7);
    let mut last_available = 0;
    let mut reduced = 1.0;
    for position in 0..30_720 {
        if position > 0 && position % 1536 == 0 {
            last_available = position;
            gain.observe(Event {
                epoch: 7,
                end_16k: position / 3,
                available_at: position,
                probability: 0.0,
            });
        }
        reduced = gain.next_gain(position, true);
    }
    // -6 dB is the prototype's finite floor, not a speech-preservation claim.
    assert!((reduced - 0.501_187_2).abs() < 0.000_01);
    let expiry = last_available + 4800;
    for position in 30_720..expiry {
        gain.next_gain(position, true);
    }
    assert_eq!(gain.next_gain(expiry, true), 1.0);
    assert_eq!(gain.next_gain(expiry + 1, true), 1.0);
}

#[test]
fn quiet_confidence_opens_a_previously_attenuated_gate_within_five_ms() {
    use crate::gain::GainPolicy;
    use cleanslate_gate_detector::Event;

    let mut gain = GainPolicy::new(48_000).unwrap();
    gain.reset(7);
    let mut value = 1.0;
    for position in 0..30_960 {
        if position > 0 && position % 1536 == 0 {
            gain.observe(Event {
                epoch: 7,
                end_16k: position / 3,
                available_at: position,
                probability: if position < 30_720 { 0.0 } else { 0.14 },
            });
        }
        value = gain.next_gain(position, true);
        if position == 30_719 {
            assert!((value - 0.501_187_2).abs() < 0.000_01);
        }
    }
    assert_eq!(value, 1.0);
}

#[test]
fn offline_worker_reset_discards_old_evidence_and_delivers_the_new_epoch() {
    use crate::worker::WorkerBridge;

    for rate in [44_100, 48_000, 96_000] {
        let frames = (512_u64 * u64::from(rate)).div_ceil(16_000);
        let silence = vec![0.0; frames as usize];
        let mut worker = WorkerBridge::prepare(rate, 1, ProcessMode::Offline).unwrap();
        worker.reset(7);
        assert!(worker.submit(&silence, None));
        assert!(worker.wait_until_processed(frames));
        // Leave epoch 7's real detector event queued when starting a new stream.
        for epoch in [8, 9] {
            worker.reset(epoch);
            assert!(worker.pop_event().is_none(), "old evidence survived reset");
            assert!(worker.submit(&silence, None));
            assert!(worker.wait_until_processed(frames));
            let fresh = worker
                .pop_event()
                .expect("fresh evidence after bounded offline wait");
            assert_eq!(fresh.epoch, epoch, "wrong stream at {rate} Hz");
            assert_eq!(fresh.end_16k, 512);
            assert_eq!(fresh.available_at, frames);
            assert!(fresh.probability.is_finite());
            assert!(worker.pop_event().is_none());
            // Queue another event so each reset rejects an undrained result.
            assert!(worker.submit(&silence, None));
            assert!(worker.wait_until_processed(frames * 2));
        }
    }
}

#[test]
fn mono_worker_matches_direct_detector_across_host_block_sizes() {
    use crate::worker::WorkerBridge;
    use cleanslate_gate_detector::Detector;

    let source: Vec<f32> = (0..4608).map(|i| (i as f32 * 0.07).sin() * 0.1).collect();
    let mut direct = Detector::new(48_000).unwrap();
    direct.reset();
    let mut expected = Vec::new();
    direct
        .push(&source, |mut event| {
            event.epoch = 51;
            expected.push(event);
        })
        .unwrap();
    for block in [127, 512, 1536] {
        let mut worker = WorkerBridge::prepare(48_000, 1, ProcessMode::Offline).unwrap();
        worker.reset(51);
        let mut processed = 0;
        for samples in source.chunks(block) {
            assert!(worker.submit(samples, None));
            processed += samples.len() as u64;
            assert!(worker.wait_until_processed(processed));
        }
        let actual: Vec<_> = std::iter::from_fn(|| worker.pop_event()).collect();
        assert_eq!(actual, expected, "host block size {block}");
    }
}

#[test]
fn worker_faults_hide_queued_evidence_until_reset_even_when_epoch_is_reused() {
    use crate::worker::WorkerBridge;

    let mut worker = WorkerBridge::prepare(48_000, 1, ProcessMode::Offline).unwrap();
    for invalid in [vec![f32::NAN], vec![0.0; 8193]] {
        worker.reset(7);
        assert!(worker.submit(&[0.0; 1536], None));
        assert!(worker.wait_until_processed(1536));
        assert!(!worker.submit(&invalid, None));
        assert!(worker.failed());
        assert!(worker.pop_event().is_none());
        assert!(!worker.submit(&[0.0; 1536], None));
        worker.reset(7);
        assert!(!worker.failed());
        assert!(worker.submit(&[0.0; 1536], None));
        assert!(worker.wait_until_processed(1536));
        let fresh = worker
            .pop_event()
            .expect("reset should recover worker inference");
        assert_eq!(fresh.epoch, 7);
        assert_eq!(fresh.end_16k, 512);
    }
}

#[test]
fn stereo_worker_uses_both_channels_without_phase_cancellation() {
    use crate::worker::WorkerBridge;
    use cleanslate_gate_detector::Detector;

    let right: Vec<f32> = (0..4608).map(|i| (i as f32 * 0.07).sin() * 0.1).collect();
    for left in [
        vec![0.0; right.len()],
        right.iter().map(|sample| -sample).collect(),
    ] {
        let mut left_detector = Detector::new(48_000).unwrap();
        let mut right_detector = Detector::new(48_000).unwrap();
        let mut left_events = Vec::new();
        let mut right_events = Vec::new();
        left_detector
            .push(&left, |event| left_events.push(event))
            .unwrap();
        right_detector
            .push(&right, |event| right_events.push(event))
            .unwrap();
        let mut worker = WorkerBridge::prepare(48_000, 2, ProcessMode::Offline).unwrap();
        worker.reset(4);
        for (i, (l, r)) in left.chunks(1536).zip(right.chunks(1536)).enumerate() {
            assert!(worker.submit(l, Some(r)));
            assert!(worker.wait_until_processed((i as u64 + 1) * 1536));
        }
        for (l, r) in left_events.iter().zip(right_events) {
            let event = worker.pop_event().expect("stereo evidence");
            assert_eq!(event.epoch, 4);
            assert_eq!(event.end_16k, l.end_16k);
            assert_eq!(event.available_at, l.available_at);
            assert_eq!(event.probability, l.probability.max(r.probability));
        }
        assert!(worker.pop_event().is_none());
    }
}

#[test]
fn worker_result_overflow_requires_reset_and_realtime_wait_never_arms_a_fault() {
    use crate::worker::WorkerBridge;

    let mut worker = WorkerBridge::prepare(48_000, 1, ProcessMode::Offline).unwrap();
    worker.reset(4);
    for frame in 1..=32 {
        assert!(worker.submit(&[0.0; 1536], None));
        assert!(worker.wait_until_processed(frame * 1536));
    }
    assert!(worker.submit(&[0.0; 1536], None));
    assert!(!worker.wait_until_processed(33 * 1536));
    assert!(worker.failed());
    assert!(worker.pop_event().is_none());
    worker.reset(5);
    assert!(worker.submit(&[0.0; 1536], None));
    assert!(worker.wait_until_processed(1536));
    assert_eq!(worker.pop_event().unwrap().epoch, 5);
    drop(worker);

    let worker = WorkerBridge::prepare(48_000, 1, ProcessMode::Realtime).unwrap();
    assert!(!worker.wait_until_processed(1536));
    assert!(
        !worker.failed(),
        "realtime must return without entering the offline wait"
    );
}

#[test]
fn gate_has_distinct_stable_format_and_state_identity() {
    assert_eq!(SmartGate::CLAP_ID, "org.cleanslate.smart-gate");
    assert_eq!(SmartGate::VST3_CLASS_ID, *b"CleanSlateGate01");
    assert_ne!(SmartGate::CLAP_ID, "org.cleanslate.fl-probe");
    assert_ne!(SmartGate::VST3_CLASS_ID, *b"CleanSlateProbe1");
    let gate = SmartGate::default();
    assert_eq!(gate.params.serialize_fields()[STATE_KEY], "2");
    assert_eq!(gate.params.param_map().len(), 1);
    assert_eq!(gate.params.param_map()[0].0, BYPASS_ID);
}

#[test]
fn own_foreign_or_future_state_restores_to_safe_bypass() {
    let state = PluginState {
        version: SmartGate::VERSION.into(),
        params: BTreeMap::from([(BYPASS_ID.into(), ParamValue::Bool(false))]),
        fields: GateParams::default().serialize_fields(),
    };
    let wire = serde_json::to_vec(&state).unwrap();
    let mut restored: PluginState = serde_json::from_slice(&wire).unwrap();
    SmartGate::filter_state(&mut restored);
    assert!(matches!(restored.params[BYPASS_ID], ParamValue::Bool(true)));
    assert_eq!(restored.fields, GateParams::default().serialize_fields());
    for fields in [
        BTreeMap::new(),
        BTreeMap::from([(STATE_KEY.into(), "2".into())]),
        BTreeMap::from([(STATE_KEY.into(), "\"invalid\"".into())]),
    ] {
        let mut foreign = PluginState {
            version: "0.0.6".into(),
            params: BTreeMap::from([("probe_group".into(), ParamValue::I32(1))]),
            fields,
        };
        SmartGate::filter_state(&mut foreign);
        assert_eq!(foreign.params.len(), 1);
        assert!(matches!(foreign.params[BYPASS_ID], ParamValue::Bool(true)));
        assert_eq!(foreign.fields, GateParams::default().serialize_fields());
    }
}

#[test]
fn mono_stereo_and_bypass_preserve_every_input_bit_without_host_work() {
    for channels in [1, 2] {
        for frames in [0, 1, 17, 1024] {
            for bypass in [false, true] {
                let pattern = [0.0, -0.0, 0.25, -1.0, f32::from_bits(0x7fc0_1234)];
                let mut audio: Vec<Vec<f32>> = (0..channels)
                    .map(|channel| {
                        (0..frames)
                            .map(|i| pattern[(i + channel) % pattern.len()])
                            .collect()
                    })
                    .collect();
                let expected: Vec<Vec<u32>> = audio
                    .iter()
                    .map(|c| c.iter().map(|x| x.to_bits()).collect())
                    .collect();
                let mut buffer = Buffer::default();
                // The channel storage remains live, equally sized and exclusively borrowed until buffer drops.
                unsafe {
                    buffer.set_slices(frames, |slices| {
                        slices.extend(audio.iter_mut().map(Vec::as_mut_slice))
                    });
                }
                let mut gate = SmartGate::default();
                Arc::get_mut(&mut gate.params).unwrap().bypass =
                    BoolParam::new("Bypass", bypass).make_bypass();
                let mut aux = AuxiliaryBuffers {
                    inputs: &mut [],
                    outputs: &mut [],
                };
                assert!(matches!(
                    gate.process(&mut buffer, &mut aux, &mut UnusedHost),
                    ProcessStatus::Normal
                ));
                let actual: Vec<Vec<u32>> = buffer
                    .as_slice()
                    .iter()
                    .map(|c| c.iter().map(|x| x.to_bits()).collect())
                    .collect();
                assert_eq!(actual, expected);
            }
        }
    }
}

#[test]
fn explicitly_armed_offline_callback_applies_one_finite_gain_and_reset_restores_original() {
    for channels in [1, 2] {
        let mut gate = SmartGate::default();
        Arc::get_mut(&mut gate.params).unwrap().bypass =
            BoolParam::new("Bypass", false).make_bypass();
        let layout = &SmartGate::AUDIO_IO_LAYOUTS[if channels == 1 { 1 } else { 0 }];
        assert!(gate.activate(
            layout,
            &BufferConfig {
                sample_rate: 48_000.0,
                min_buffer_size: None,
                max_buffer_size: 1536,
                process_mode: ProcessMode::Offline,
            },
            &mut ActivationHost
        ));
        gate.reset();
        let original: Vec<Vec<f32>> = (0..channels)
            .map(|channel| vec![if channel == 0 { 0.0001 } else { -0.0001 }; 1536])
            .collect();
        let mut audio = original.clone();
        process_test_block(&mut gate, &mut audio);
        assert_eq!(audio, original, "unarmed audio must stay original");
        gate.arm_experimental();
        for _ in 0..32 {
            audio.clone_from(&original);
            process_test_block(&mut gate, &mut audio);
        }
        let gain = audio[0][1535] / original[0][1535];
        assert!(
            (0.50..0.51).contains(&gain),
            "armed absence did not reach finite reduction: {gain}"
        );
        for channel in 0..channels {
            assert_eq!(audio[channel][1535] / original[channel][1535], gain);
        }
        Arc::get_mut(&mut gate.params).unwrap().bypass =
            BoolParam::new("Bypass", true).make_bypass();
        audio.clone_from(&original);
        process_test_block(&mut gate, &mut audio);
        assert_eq!(audio, original, "bypass overrides explicit arming");
        Arc::get_mut(&mut gate.params).unwrap().bypass =
            BoolParam::new("Bypass", false).make_bypass();
        audio.clone_from(&original);
        audio[0][500] = f32::from_bits(0x7fc0_1234);
        process_test_block(&mut gate, &mut audio);
        for channel in 0..channels {
            for frame in 0..1536 {
                let expected = if channel == 0 && frame == 500 {
                    0x7fc0_1234
                } else {
                    original[channel][frame].to_bits()
                };
                assert_eq!(
                    audio[channel][frame].to_bits(),
                    expected,
                    "invalid input must stay original"
                );
            }
        }
        audio.clone_from(&original);
        process_test_block(&mut gate, &mut audio);
        assert_eq!(
            audio, original,
            "faults preserve subsequent input until reset"
        );
        gate.reset();
        audio.clone_from(&original);
        process_test_block(&mut gate, &mut audio);
        assert_eq!(
            audio, original,
            "reset must disarm and preserve original audio"
        );
        gate.deactivate();
    }
}

#[test]
fn offline_callback_gain_is_independent_of_host_block_size() {
    let source = vec![0.0001_f32; 32_768];
    let mut rendered = Vec::new();
    for block in [127, 1536, 32_768] {
        let mut gate = SmartGate::default();
        Arc::get_mut(&mut gate.params).unwrap().bypass =
            BoolParam::new("Bypass", false).make_bypass();
        assert!(gate.activate(
            &SmartGate::AUDIO_IO_LAYOUTS[1],
            &BufferConfig {
                sample_rate: 48_000.0,
                min_buffer_size: None,
                max_buffer_size: 32_768,
                process_mode: ProcessMode::Offline,
            },
            &mut ActivationHost
        ));
        gate.reset();
        gate.arm_experimental();
        let mut result = Vec::new();
        for samples in source.chunks(block) {
            let mut audio = vec![samples.to_vec()];
            process_test_block(&mut gate, &mut audio);
            result.extend_from_slice(&audio[0]);
        }
        assert!(
            result.last().unwrap() < source.last().unwrap(),
            "all-original output is not gating"
        );
        rendered.push(result);
    }
    assert_eq!(rendered[0], rendered[1]);
    assert_eq!(rendered[0], rendered[2]);
}

fn process_test_block(gate: &mut SmartGate, audio: &mut [Vec<f32>]) {
    let frames = audio[0].len();
    let mut buffer = Buffer::default();
    // SAFETY: all channels have equal lengths and remain exclusively borrowed until buffer drops.
    unsafe {
        buffer.set_slices(frames, |slices| {
            slices.extend(audio.iter_mut().map(Vec::as_mut_slice))
        });
    }
    assert!(matches!(
        gate.process(
            &mut buffer,
            &mut AuxiliaryBuffers {
                inputs: &mut [],
                outputs: &mut [],
            },
            &mut UnusedHost
        ),
        ProcessStatus::Normal
    ));
}

struct ActivationHost;
impl ActivateContext<SmartGate> for ActivationHost {
    fn plugin_api(&self) -> PluginApi {
        panic!("Gate activation must be host independent")
    }
    fn execute(&self, _: ()) {
        panic!("Gate owns its inference worker")
    }
    fn set_latency_samples(&self, samples: u32) {
        assert_eq!(samples, 0);
    }
    fn set_current_voice_capacity(&self, _: u32) {
        panic!("Gate has no voices")
    }
}

struct UnusedHost;
impl ProcessContext<SmartGate> for UnusedHost {
    fn plugin_api(&self) -> PluginApi {
        panic!("pass-through must not query the host")
    }
    fn execute_background(&self, _: ()) {
        panic!("pass-through must not schedule work")
    }
    fn execute_gui(&self, _: ()) {
        panic!("pass-through must not schedule work")
    }
    fn transport(&self) -> &Transport {
        panic!("pass-through must not depend on transport")
    }
    fn next_event(&mut self) -> Option<PluginNoteEvent<SmartGate>> {
        panic!("pass-through must not consume events")
    }
    fn try_send_event(
        &mut self,
        _: PluginNoteEvent<SmartGate>,
    ) -> Result<(), (PluginNoteEvent<SmartGate>, SendEventError)> {
        panic!("pass-through must not emit events")
    }
    fn set_latency_samples(&self, _: u32) {
        panic!("pass-through has no runtime latency change")
    }
    fn request_restart(&self) {
        panic!("pass-through must not restart the host")
    }
    fn set_current_voice_capacity(&self, _: u32) {
        panic!("pass-through has no voices")
    }
}
