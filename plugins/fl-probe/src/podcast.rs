//! Podcast-only product boundary around the maintained Session adapter.

use nice_plug::plugin::{ParamValue, PluginState};
use nice_plug::prelude::Params;

pub const STATE_VERSION: &str = "org.cleanslate.podcast.state-version";
pub const STATE_REJECTED: &str = "org.cleanslate.podcast.state-rejected";

pub fn editor(
    params: std::sync::Arc<crate::ProbeParams>,
    executor: nice_plug::prelude::AsyncExecutor<crate::FlProbe>,
    gui: std::sync::Arc<crate::runtime::GuiState>,
    state: std::sync::Arc<nice_plug_egui::EguiEditorState>,
) -> Option<nice_plug_egui::EguiEditor<crate::editor::Panel>> {
    nice_plug_egui::create_egui_editor(
        state,
        nice_plug_egui::RepaintNotifier::new(),
        nice_plug_egui::EguiNiceSettings::new(),
        crate::editor::Panel::new(params, executor, gui),
    )
}

pub fn filter_state(state: &mut PluginState) {
    if state.fields.get(STATE_VERSION).map(String::as_str) != Some("1") {
        // The adapter hook cannot reject a load: disconnect and report the rejected state.
        state.fields = crate::FlProbe::default().params.serialize_fields();
        state.fields.insert(STATE_REJECTED.into(), "true".into());
        state.params = [
            ("probe_group".into(), ParamValue::I32(0)),
            ("probe_member".into(), ParamValue::I32(1)),
            ("probe_export_run".into(), ParamValue::Bool(false)),
            ("enable_processing".into(), ParamValue::Bool(false)),
            ("reduction_db".into(), ParamValue::F32(-12.0)),
            ("fade_in_ms".into(), ParamValue::F32(15.0)),
            ("fade_out_ms".into(), ParamValue::F32(100.0)),
            ("listen_mode".into(), ParamValue::I32(1)),
        ]
        .into();
    }
    state
        .params
        .insert("session_mode".into(), ParamValue::Bool(true));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlProbe;
    use nice_plug::plugin::{ParamValue, PluginState};
    use nice_plug::prelude::*;

    #[test]
    fn podcast_identity_cannot_replace_probe_or_gate() {
        assert_eq!(FlProbe::NAME, "CleanSlate Podcast");
        assert_eq!(FlProbe::CLAP_ID, "org.cleanslate.podcast");
        assert_eq!(FlProbe::VST3_CLASS_ID, *b"CleanSlatePodc01");
        assert_ne!(FlProbe::VST3_CLASS_ID, *b"CleanSlateProbe1");
        assert_ne!(FlProbe::VST3_CLASS_ID, *b"CleanSlateGate01");
        assert!(
            FlProbe::default()
                .params()
                .serialize_fields()
                .contains_key(STATE_VERSION)
        );
    }

    #[test]
    fn foreign_state_is_disconnected_and_visibly_rejected() {
        let mut state = PluginState {
            version: "0.0.6".into(),
            params: [
                ("probe_group".into(), ParamValue::I32(7)),
                ("enable_processing".into(), ParamValue::Bool(true)),
            ]
            .into(),
            fields: [("session_cache".into(), "\"old-source-key\"".into())].into(),
        };
        FlProbe::filter_state(&mut state);
        assert!(matches!(state.params["probe_group"], ParamValue::I32(0)));
        assert!(matches!(
            state.params["enable_processing"],
            ParamValue::Bool(false)
        ));
        assert!(matches!(
            state.params["session_mode"],
            ParamValue::Bool(true)
        ));
        assert_eq!(state.fields[STATE_REJECTED], "true");
        assert_eq!(state.fields["session_cache"], "\"\"");
        assert!(
            !serde_json::to_string(&state)
                .unwrap()
                .contains("old-source-key")
        );
    }

    #[test]
    fn podcast_state_keeps_source_binding_labels_and_session_only_mode() {
        let mut state = PluginState {
            version: "0.0.1".into(),
            params: [("session_mode".into(), ParamValue::Bool(false))].into(),
            fields: FlProbe::default().params().serialize_fields(),
        };
        state.fields.insert(
            "session_cache".into(),
            "\"0123456789abcdef0123456789abcdef\"".into(),
        );
        state.fields.insert("session_labels".into(), "[]".into());
        let fields = state.fields.clone();
        FlProbe::filter_state(&mut state);
        assert_eq!(state.fields, fields);
        assert!(matches!(
            state.params["session_mode"],
            ParamValue::Bool(true)
        ));
        state.fields.insert(STATE_VERSION.into(), "2".into());
        FlProbe::filter_state(&mut state);
        assert_eq!(state.fields[STATE_REJECTED], "true");
    }
}
