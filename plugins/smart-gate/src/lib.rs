mod editor;
mod engine;
pub mod gain;
pub mod worker;

use nice_plug::editor::dpi::LogicalSize;
use nice_plug::plugin::ParamValue;
use nice_plug::prelude::*;
use nice_plug_egui::{
    EguiEditor, EguiEditorState, EguiNiceSettings, RepaintNotifier, create_egui_editor,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub const BYPASS_ID: &str = "smart_gate_bypass";
pub const STATE_KEY: &str = "org.cleanslate.smart-gate.state-version";
pub const STATE_VERSION: u32 = 2;

const CONSENT_MASK: u64 = 3;

#[repr(u64)]
enum Consent {
    Unavailable,
    Ready,
    Armed,
}

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
    consent: Arc<AtomicU64>,
    engine: Option<engine::Engine>,
}

impl Default for SmartGate {
    fn default() -> Self {
        Self {
            params: Arc::new(GateParams::default()),
            editor_state: EguiEditorState::from_size(LogicalSize::new(640.0, 440.0), 1.0),
            consent: Arc::new(AtomicU64::new(Consent::Unavailable as u64)),
            engine: None,
        }
    }
}

impl SmartGate {
    /// Ignores requests unless prepared and active; consent is never saved.
    pub fn arm_experimental(&self) {
        let consent = self.consent.load(Ordering::Acquire);
        if consent & CONSENT_MASK == Consent::Ready as u64 {
            let _ = self.consent.compare_exchange(
                consent,
                (consent & !CONSENT_MASK) | Consent::Armed as u64,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
        }
    }

    pub fn experimental_armed(&self) -> bool {
        self.consent.load(Ordering::Acquire) & CONSENT_MASK == Consent::Armed as u64
    }

    fn invalidate_consent(&self) {
        // Changing generation retires a press/focus that predates this lifecycle boundary.
        let _ = self
            .consent
            .try_update(Ordering::AcqRel, Ordering::Acquire, |consent| {
                Some(consent.wrapping_add(CONSENT_MASK + 1) & !CONSENT_MASK)
            });
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
            editor::Panel::new(Arc::clone(&self.params), Arc::clone(&self.consent)),
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

    fn activate(
        &mut self,
        layout: &AudioIOLayout,
        config: &BufferConfig,
        context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.invalidate_consent();
        self.engine = None;
        if ![44_100.0, 48_000.0, 96_000.0].contains(&config.sample_rate) {
            return false;
        }
        let channels = layout.main_input_channels.map_or(0, NonZeroU32::get);
        if layout.main_input_channels != layout.main_output_channels {
            return false;
        }
        let Ok(engine) =
            engine::Engine::prepare(config.sample_rate as u32, channels, config.process_mode)
        else {
            return false;
        };
        self.engine = Some(engine);
        self.consent
            .fetch_or(Consent::Ready as u64, Ordering::Release);
        context.set_latency_samples(0);
        true
    }

    fn reset(&mut self) {
        self.invalidate_consent();
        if let Some(engine) = &mut self.engine {
            engine.reset();
            self.consent
                .fetch_or(Consent::Ready as u64, Ordering::Release);
        }
    }

    fn deactivate(&mut self) {
        self.invalidate_consent();
        self.engine = None;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let enabled = self.experimental_armed() && !self.params.bypass.value();
        if let Some(engine) = &mut self.engine {
            engine.process(buffer, enabled);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for SmartGate {
    const CLAP_ID: &'static str = "org.cleanslate.smart-gate";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Experimental single-input speech gate; bypassed until explicitly armed.");
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
