mod editor;
pub mod gain;

use nice_plug::editor::dpi::LogicalSize;
use nice_plug::plugin::ParamValue;
use nice_plug::prelude::*;
use nice_plug_egui::{
    EguiEditor, EguiEditorState, EguiNiceSettings, RepaintNotifier, create_egui_editor,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const BYPASS_ID: &str = "smart_gate_bypass";
pub const STATE_KEY: &str = "org.cleanslate.smart-gate.state-version";
pub const STATE_VERSION: u32 = 2;

#[derive(Params)]
pub struct GateParams {
    #[id = "smart_gate_bypass"]
    bypass: BoolParam,
    #[persist = "org.cleanslate.smart-gate.state-version"]
    state_version: Mutex<u32>,
}

impl Default for GateParams {
    fn default() -> Self {
        Self {
            bypass: BoolParam::new("Bypass", true).make_bypass(),
            state_version: Mutex::new(STATE_VERSION),
        }
    }
}

pub struct SmartGate {
    params: Arc<GateParams>,
    editor_state: Arc<EguiEditorState>,
    armed: Arc<AtomicBool>,
}

impl Default for SmartGate {
    fn default() -> Self {
        Self {
            params: Arc::new(GateParams::default()),
            editor_state: EguiEditorState::from_size(LogicalSize::new(640.0, 440.0), 1.0),
            armed: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl SmartGate {
    /// This consent is deliberately absent from parameters and saved state.
    pub fn arm_experimental(&self) {
        self.armed.store(true, Ordering::Release);
    }

    pub fn experimental_armed(&self) -> bool {
        self.armed.load(Ordering::Acquire)
    }
}

impl Plugin for SmartGate {
    const NAME: &'static str = "CleanSlate Smart Gate";
    const VENDOR: &'static str = "CleanSlate";
    const URL: &'static str = "https://github.com/Today20092/podcast-bleed-reducer";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];
    type Editor = EguiEditor<editor::Panel>;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Self::Editor> {
        create_egui_editor(
            self.editor_state.clone(),
            RepaintNotifier::new(),
            EguiNiceSettings::new(),
            editor::Panel::new(self.params.clone()),
        )
    }

    fn filter_state(state: &mut PluginState) {
        // Restoring a project is never consent to experimental attenuation.
        state.params.clear();
        state
            .params
            .insert(BYPASS_ID.into(), ParamValue::Bool(true));
        state.fields.clear();
        state
            .fields
            .insert(STATE_KEY.into(), STATE_VERSION.to_string());
        state.version = Self::VERSION.into();
    }

    fn reset(&mut self) {
        self.armed.store(false, Ordering::Release);
    }

    fn process(
        &mut self,
        _: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Ticket 93 owns qualified gating. Leaving the framework's input-filled buffer untouched is exact unity.
        ProcessStatus::Normal
    }
}

impl ClapPlugin for SmartGate {
    const CLAP_ID: &'static str = "org.cleanslate.smart-gate";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Single-input pass-through prototype; speech gating is not enabled.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
        ClapFeature::Mono,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for SmartGate {
    const VST3_CLASS_ID: [u8; 16] = *b"CleanSlateGate01";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nice_export_clap!(SmartGate);
nice_export_vst3!(SmartGate);

#[cfg(test)]
mod tests;
