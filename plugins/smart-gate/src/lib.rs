mod editor;
#[cfg(test)]
#[path = "../../fl-probe/src/editor_capture.rs"]
mod editor_capture;
mod engine;
pub mod gain;
mod history;
pub mod worker;

use nice_plug::editor::dpi::LogicalSize;
use nice_plug::plugin::ParamValue;
use nice_plug::prelude::*;
use nice_plug_egui::{
    EguiEditor, EguiEditorState, EguiNiceSettings, RepaintNotifier, create_egui_editor,
};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
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

#[must_use]
#[repr(u8)]
pub(crate) enum DetectionState {
    Waiting,
    Current,
    Failed,
}

#[derive(Params)]
pub struct GateParams {
    #[id = "smart_gate_bypass"]
    bypass: BoolParam,
    #[persist = "org.cleanslate.smart-gate.state-version"]
    state_version: Mutex<u32>,
    // One coherent last-block gain and generation; no GUI work on the callback.
    meter: AtomicU64,
    history: history::History,
    #[id = "smart_gate_range"]
    range: FloatParam,
}

impl Default for GateParams {
    fn default() -> Self {
        Self {
            bypass: BoolParam::new("Bypass", true).make_bypass(),
            state_version: Mutex::new(STATE_VERSION),
            meter: AtomicU64::new(0),
            history: history::History::default(),
            range: FloatParam::new("Range", 6.0, FloatRange::Linear { min: 0.0, max: 6.0 })
                .with_unit(" dB"),
        }
    }
}

pub struct SmartGate {
    params: Arc<GateParams>,
    editor_state: Arc<EguiEditorState>,
    consent: Arc<AtomicU64>,
    detection_state: Arc<AtomicU8>,
    repaint: RepaintNotifier,
    engine: Option<engine::Engine>,
}

impl Default for SmartGate {
    fn default() -> Self {
        Self {
            params: Arc::new(GateParams::default()),
            editor_state: EguiEditorState::from_size(LogicalSize::new(1000.0, 620.0), 1.0),
            consent: Arc::new(AtomicU64::new(Consent::Unavailable as u64)),
            detection_state: Arc::new(AtomicU8::new(DetectionState::Waiting as u8)),
            repaint: RepaintNotifier::new(),
            engine: None,
        }
    }
}

impl SmartGate {
    /// Ignores requests unless prepared and active; consent is never saved.
    pub fn arm_experimental(&self) {
        let consent = self.consent.load(Ordering::Acquire);
        if consent & CONSENT_MASK == Consent::Ready as u64
            && self
                .consent
                .compare_exchange(
                    consent,
                    (consent & !CONSENT_MASK) | Consent::Armed as u64,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
        {
            self.repaint.request_repaint();
        }
    }

    pub fn experimental_armed(&self) -> bool {
        self.consent.load(Ordering::Acquire) & CONSENT_MASK == Consent::Armed as u64
    }

    fn invalidate_consent(&self) {
        self.params.history.clear();
        self.params.meter.store(0, Ordering::Relaxed);
        // Changing generation retires a press/focus that predates this lifecycle boundary.
        let _ = self
            .consent
            .try_update(Ordering::AcqRel, Ordering::Acquire, |consent| {
                Some(consent.wrapping_add(CONSENT_MASK + 1) & !CONSENT_MASK)
            });
        self.repaint.request_repaint();
    }

    fn publish_detection_state(&self, state: DetectionState) {
        let state = state as u8;
        if self.detection_state.swap(state, Ordering::AcqRel) != state {
            self.repaint.request_repaint();
        }
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
            self.repaint.clone(),
            EguiNiceSettings::new(),
            editor::Panel::new(
                Arc::clone(&self.params),
                Arc::clone(&self.consent),
                Arc::clone(&self.detection_state),
            ),
        )
    }

    fn filter_state(state: &mut PluginState) {
        // Restoring a project is never consent to experimental attenuation.
        let range = state.params.get("smart_gate_range").cloned();
        state.params.clear();
        if let Some(ParamValue::F32(value)) = range
            && value.is_finite()
            && (0.0..=6.0).contains(&value)
        {
            state
                .params
                .insert("smart_gate_range".into(), ParamValue::F32(value));
        }
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
        self.publish_detection_state(DetectionState::Waiting);
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
        self.repaint.request_repaint();
        context.set_latency_samples(0);
        true
    }

    fn reset(&mut self) {
        self.invalidate_consent();
        if let Some(engine) = &mut self.engine {
            engine.reset();
            let state = engine.detection_state();
            self.publish_detection_state(state);
            self.consent
                .fetch_or(Consent::Ready as u64, Ordering::Release);
            self.repaint.request_repaint();
        } else {
            self.publish_detection_state(DetectionState::Waiting);
        }
    }

    fn deactivate(&mut self) {
        self.invalidate_consent();
        self.publish_detection_state(DetectionState::Waiting);
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
            engine.process(
                buffer,
                enabled,
                self.params.range.value(),
                &self.params.history,
            );
            let sequence = (self.params.meter.load(Ordering::Relaxed) >> 32) as u32;
            self.params.meter.store(
                (u64::from(sequence.wrapping_add(1)) << 32)
                    | u64::from(engine.minimum_gain.to_bits()),
                Ordering::Relaxed,
            );
            let state = engine.detection_state();
            self.publish_detection_state(state);
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
