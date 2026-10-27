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
    assert!(!gate.experimental_armed());
    gate.arm_experimental();
    assert!(gate.experimental_armed());
    assert_eq!(gate.params.serialize_fields().len(), 1);
    gate.reset();
    assert!(!gate.experimental_armed());
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
