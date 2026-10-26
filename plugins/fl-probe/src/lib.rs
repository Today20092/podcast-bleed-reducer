mod calibration;
mod editor;
#[cfg(test)]
mod editor_capture;
mod history;
#[cfg(cleanslate_podcast)]
mod podcast;
mod probe;
mod report;
mod runtime;
mod session_analysis;
mod session_editor;
mod session_mode;
mod shared_audio;

use calibration::{SavedCalibration, Session, TrainingTask};
use nice_plug::editor::dpi::LogicalSize;
#[cfg(not(cleanslate_podcast))]
use nice_plug::plugin::ParamValue;
use nice_plug::prelude::*;
use nice_plug_egui::{EguiEditor, EguiEditorState};
#[cfg(not(cleanslate_podcast))]
use nice_plug_egui::{EguiNiceSettings, RepaintNotifier, create_egui_editor};
use probe::{GROUPS_STATE, Probe};
use report::{Event, Reporter};
use runtime::{GuiState, RenderSettings, Runtime, RuntimeStatus};
use shared_audio::{Bus, SharedAudio};
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;

#[derive(Enum, Clone, Copy, Debug, PartialEq)]
pub enum ListenMode {
    Original,
    Processed,
    Removed,
}

#[derive(Params)]
pub struct ProbeParams {
    #[cfg(cleanslate_podcast)]
    #[persist = "org.cleanslate.podcast.state-version"]
    product_state_version: parking_lot::Mutex<u32>,
    #[cfg(cleanslate_podcast)]
    #[persist = "org.cleanslate.podcast.state-rejected"]
    product_state_rejected: parking_lot::Mutex<bool>,
    #[id = "probe_group"]
    group: IntParam,
    #[id = "probe_member"]
    member: IntParam,
    #[id = "probe_export_run"]
    export_run: BoolParam,
    #[id = "enable_processing"]
    enabled: BoolParam,
    #[id = "reduction_db"]
    reduction: FloatParam,
    #[id = "fade_in_ms"]
    attack: FloatParam,
    #[id = "fade_out_ms"]
    release: FloatParam,
    #[id = "listen_mode"]
    listen: EnumParam<ListenMode>,
    #[persist = "calibration"]
    saved: SavedCalibration,
    #[persist = "examples"]
    saved_examples: parking_lot::Mutex<Option<Vec<calibration::ReferenceFrame>>>,
    #[id = "session_mode"]
    session_mode: BoolParam,
    #[persist = "session_cache"]
    session_key: parking_lot::Mutex<String>,
    #[persist = "session_labels"]
    session_labels: parking_lot::Mutex<Vec<session_analysis::LabelRange>>,
    #[persist = "session_sound"]
    session_settings: parking_lot::Mutex<session_analysis::Settings>,
    #[persist = "session_label_evidence"]
    session_label_evidence: parking_lot::Mutex<Vec<session_mode::LabelEvidence>>,
}

pub struct FlProbe {
    params: Arc<ProbeParams>,
    probe: Probe,
    reporter: Option<Reporter>,
    callbacks: u64,
    offline: bool,
    mode: &'static str,
    bus: &'static Bus,
    audio: SharedAudio,
    session: Option<Arc<Session>>,
    bound_group: i32,
    bound_member: usize,
    gui: Arc<GuiState>,
    editor_state: Arc<EguiEditorState>,
    runtime: Runtime,
    restart_pending: bool,
    saved_revision: u64,
    capture: Option<session_mode::CaptureHandle>,
    session_mode: bool,
}

impl Default for FlProbe {
    fn default() -> Self {
        Self {
            params: Arc::new(ProbeParams {
                #[cfg(cleanslate_podcast)]
                product_state_version: parking_lot::Mutex::new(1),
                #[cfg(cleanslate_podcast)]
                product_state_rejected: parking_lot::Mutex::new(false),
                group: IntParam::new(
                    "Session group (0 = off)",
                    0,
                    IntRange::Linear { min: 0, max: 8 },
                ),
                member: IntParam::new("Microphone member", 1, IntRange::Linear { min: 1, max: 8 }),
                export_run: BoolParam::new("Label this test as Export (manual)", false),
                enabled: BoolParam::new("Enable processing", false).non_automatable(),
                reduction: FloatParam::new(
                    "Legacy probe reduction",
                    -12.0,
                    FloatRange::Linear {
                        min: -48.0,
                        max: 0.0,
                    },
                )
                .with_unit(" dB"),
                attack: FloatParam::new(
                    "Legacy probe fade in",
                    15.0,
                    FloatRange::Linear {
                        min: 5.0,
                        max: 80.0,
                    },
                )
                .with_unit(" ms"),
                release: FloatParam::new(
                    "Legacy probe fade out",
                    100.0,
                    FloatRange::Linear {
                        min: 10.0,
                        max: 150.0,
                    },
                )
                .with_unit(" ms"),
                listen: EnumParam::new("Listen", ListenMode::Processed),
                saved: SavedCalibration::default(),
                saved_examples: parking_lot::Mutex::new(None),
                session_mode: BoolParam::new("Session workspace", true).non_automatable(),
                session_key: parking_lot::Mutex::new(String::new()),
                session_labels: parking_lot::Mutex::new(Vec::new()),
                session_settings: parking_lot::Mutex::new(session_analysis::Settings::default()),
                session_label_evidence: parking_lot::Mutex::new(Vec::new()),
            }),
            probe: Probe::new(),
            reporter: None,
            callbacks: 0,
            offline: false,
            mode: "Realtime",
            bus: shared_audio::bus(),
            audio: SharedAudio::for_processing(),
            session: None,
            bound_group: 0,
            bound_member: 0,
            gui: Arc::new(GuiState::new()),
            editor_state: EguiEditorState::from_size(LogicalSize::new(1000.0, 680.0), 1.0),
            runtime: Runtime::new(),
            restart_pending: false,
            saved_revision: 0,
            capture: None,
            session_mode: true,
        }
    }
}

impl Plugin for FlProbe {
    const NAME: &'static str = if cfg!(cleanslate_podcast) {
        "CleanSlate Podcast"
    } else {
        "CleanSlate FL Probe"
    };
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
    type BackgroundTask = TrainingTask;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    #[cfg(cleanslate_podcast)]
    fn filter_state(state: &mut PluginState) {
        podcast::filter_state(state);
    }

    #[cfg(not(cleanslate_podcast))]
    fn filter_state(state: &mut PluginState) {
        state
            .fields
            .entry("session_label_evidence".into())
            .or_insert_with(|| "[]".into());
        state
            .params
            .entry("session_mode".into())
            .or_insert(ParamValue::Bool(false));
        state
            .fields
            .entry("session_cache".into())
            .or_insert_with(|| "\"\"".into());
        state
            .fields
            .entry("session_labels".into())
            .or_insert_with(|| "[]".into());
        state
            .fields
            .entry("session_sound".into())
            .or_insert_with(|| {
                serde_json::to_string(&session_analysis::Settings::default()).unwrap()
            });
        // Older probe states contain neither processing nor a persisted calibration field.
        state
            .fields
            .entry("calibration".into())
            .or_insert_with(|| "null".into());
        state
            .fields
            .entry("examples".into())
            .or_insert_with(|| "null".into());
        state
            .params
            .entry("enable_processing".into())
            .or_insert(ParamValue::Bool(false));
    }

    #[cfg(cleanslate_podcast)]
    fn editor(&mut self, executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        podcast::editor(
            self.params.clone(),
            executor,
            self.gui.clone(),
            self.editor_state.clone(),
        )
    }

    #[cfg(not(cleanslate_podcast))]
    fn editor(&mut self, executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        create_egui_editor(
            self.editor_state.clone(),
            RepaintNotifier::new(),
            EguiNiceSettings::new(),
            editor::Panel::new(self.params.clone(), executor, self.gui.clone()),
        )
    }

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        Box::new(TrainingTask::run)
    }

    fn activate(
        &mut self,
        layout: &AudioIOLayout,
        config: &BufferConfig,
        context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.probe.reset(&GROUPS_STATE);
        self.reporter = None;
        self.callbacks = 0;
        self.offline = matches!(config.process_mode, ProcessMode::Offline);
        self.mode = match config.process_mode {
            ProcessMode::Realtime => "Realtime",
            ProcessMode::Buffered => "Buffered",
            ProcessMode::Offline => "Offline",
        };
        self.audio.reset();
        let channels = layout.main_input_channels.map_or(0, |n| n.get() as usize);
        if !self.runtime.activate(
            config.sample_rate,
            cfg!(cleanslate_podcast)
                || self.params.enabled.value()
                || self.params.session_mode.value(),
            channels,
            config.max_buffer_size as usize,
        ) {
            return false;
        }
        let group = self.params.group.value();
        let member = (self.params.member.value() - 1) as usize;
        let requested_session = cfg!(cleanslate_podcast) || self.params.session_mode.value();
        let mode_changed = self.session_mode != requested_session;
        self.session_mode = requested_session;
        if (group != self.bound_group
            || member != self.bound_member
            || !self.session_mode
            || self
                .capture
                .as_ref()
                .is_some_and(|c| c.engine.sample_rate() != config.sample_rate as u32))
            && let Some(capture) = &self.capture
        {
            capture.unbind();
        }
        let previous_capture = self.capture.take();
        *self.gui.capture.lock() = None;
        match session_mode::SessionCapture::bind(
            if self.session_mode { group } else { 0 },
            member,
            self.probe.id,
            &self.params,
            config.sample_rate as u32,
        ) {
            Ok(capture) => self.capture = capture,
            Err(error) => {
                if let Some(previous) = &previous_capture {
                    previous.unbind();
                }
                *self.gui.error.lock() = Some(error);
            }
        }
        *self.gui.capture.lock() = self.capture.as_ref().map(|handle| handle.engine.clone());
        drop(previous_capture);
        if (group != self.bound_group || member != self.bound_member || mode_changed)
            && let Some(old) = &self.session
        {
            old.unbind(self.bound_member, &self.params);
        }
        self.session = if member < 2 && !self.session_mode {
            calibration::session_for(group)
        } else {
            None
        };
        self.bound_group = group;
        self.bound_member = member;
        self.restart_pending = false;
        self.gui.status.store(RuntimeStatus::Disabled as u8, SeqCst);
        self.saved_revision = self.params.saved.revision.load(SeqCst);
        if self.capture.is_some() || group == 0 {
            *self.gui.error.lock() = None;
        }
        if let Some(session) = &self.session
            && let Err(error) =
                session.bind(member, &self.params, config.sample_rate.to_bits(), channels)
        {
            session.confirmed.store(false, SeqCst);
            session.unbind(member, &self.params);
            *self.gui.error.lock() = Some(error);
            self.session = None;
        }
        *self.gui.session.lock() = self.session.clone();
        self.gui.latency_ms.store(
            (self.runtime.delay_frames as f32 * 1000.0 / config.sample_rate).to_bits(),
            SeqCst,
        );
        context.set_latency_samples(self.runtime.delay_frames as u32);
        match Reporter::start(self.probe.id) {
            Ok(reporter) => {
                self.reporter = Some(reporter);
                true
            }
            Err(_) => false,
        }
    }

    fn reset(&mut self) {
        if let Some(capture) = &mut self.capture {
            capture.reset_stream();
        }
        self.probe.reset(&GROUPS_STATE);
        self.audio.reset();
        self.runtime.reset_transport();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        self.gui
            .position
            .store(transport.pos_samples().unwrap_or(-1), SeqCst);
        self.gui.playing.store(transport.playing, SeqCst);
        let export_run = self.params.export_run.value();
        // The manual run label is diagnostic only; it never enables processing.
        let active = transport.playing || self.offline;
        let assignment_changed = self.params.group.value() != self.bound_group
            || (self.params.member.value() - 1) as usize != self.bound_member
            || (!cfg!(cleanslate_podcast) && self.params.session_mode.value() != self.session_mode);
        let enabled_changed =
            !self.session_mode && self.params.enabled.value() != self.runtime.enabled;
        let state_changed = self.params.saved.revision.load(SeqCst) != self.saved_revision;
        if (assignment_changed || enabled_changed || state_changed) && !self.restart_pending {
            if state_changed && let Some(session) = &self.session {
                session.confirmed.store(false, SeqCst);
                session.generation.fetch_add(1, SeqCst);
            }
            context.request_restart();
            self.restart_pending = true;
        }
        self.runtime.prepare(transport.pos_samples(), active);
        if !assignment_changed
            && !state_changed
            && let Some(capture) = &mut self.capture
        {
            capture.capture(transport.pos_samples(), buffer.as_slice_immutable(), active);
        }
        let mut observation = self.probe.observe(
            &GROUPS_STATE,
            if (self.session.is_some() || self.capture.is_some())
                && !assignment_changed
                && !state_changed
            {
                self.bound_group
            } else {
                0
            },
            self.params.member.value(),
            transport.pos_samples(),
            buffer.samples() as u64,
            transport.sample_rate.to_bits(),
            active,
        );
        // Epoch tracking follows processing intent; the report keeps the actual host play flag.
        observation.playing = transport.playing;
        if self.bound_member == 0
            && let Some(session) = &self.session
        {
            session.transport(
                transport.pos_samples().unwrap_or(0),
                transport.playing,
                observation.discontinuity,
            );
        }
        let share = self.audio.process(
            self.bus,
            &GROUPS_STATE,
            self.probe.binding(&GROUPS_STATE),
            transport.pos_samples(),
            transport.sample_rate.to_bits(),
            buffer.as_slice_immutable(),
            active,
        );
        let binding = self.probe.binding(&GROUPS_STATE);
        let session = if assignment_changed || state_changed {
            None
        } else {
            self.session.as_deref()
        };
        self.runtime.analyze(
            &self.audio,
            &GROUPS_STATE,
            binding,
            session,
            self.bound_member,
            transport.pos_samples(),
            buffer.samples(),
            active,
            &self.gui,
        );
        let allow_attenuation = binding.is_some_and(|binding| GROUPS_STATE.is_current(binding))
            && !assignment_changed
            && !enabled_changed
            && !state_changed
            // shortcut: CLAP render transitions are not reliably identified in the native export
            // evidence; qualify that contract before allowing attenuation through this adapter.
            && !matches!(context.plugin_api(), PluginApi::Clap);
        self.runtime.render(
            buffer.as_slice(),
            transport.pos_samples(),
            active,
            session,
            self.bound_member,
            RenderSettings {
                attenuation_db: self.params.reduction.value(),
                attack_ms: self.params.attack.value(),
                release_ms: self.params.release.value(),
                listen: if self.session_mode {
                    ListenMode::Original
                } else {
                    self.params.listen.value()
                },
            },
            allow_attenuation && !self.session_mode,
            &self.gui,
        );
        if self.session_mode {
            if let Some(capture) = &self.capture {
                capture.render(
                    buffer.as_slice(),
                    transport.pos_samples(),
                    self.runtime.delay_frames,
                    active
                        && self.params.enabled.value()
                        && !assignment_changed
                        && !enabled_changed
                        && !state_changed,
                    self.params.listen.value(),
                );
            } else if self.params.listen.value() == ListenMode::Removed {
                for channel in buffer.as_slice() {
                    channel.fill(0.0);
                }
            }
        }
        if assignment_changed || enabled_changed || state_changed {
            self.gui
                .status
                .store(RuntimeStatus::Restarting as u8, SeqCst);
        } else if binding.is_none() && self.runtime.enabled {
            self.gui
                .status
                .store(RuntimeStatus::NeedGroup as u8, SeqCst);
        }
        self.callbacks += 1;
        if let Some(reporter) = &self.reporter {
            reporter.push(Event {
                observation,
                callback: self.callbacks,
                elapsed_us: reporter
                    .origin
                    .elapsed()
                    .as_micros()
                    .min(u128::from(u64::MAX)) as u64,
                offline: self.offline,
                activation_mode: self.mode,
                host_api: match context.plugin_api() {
                    PluginApi::Clap => "CLAP",
                    PluginApi::Vst3 => "VST3",
                    PluginApi::Standalone => "Standalone",
                },
                manual_export: export_run,
                processing_active: active,
                share,
                timing_source: if transport.pos_samples.is_some() {
                    "samples"
                } else if transport.pos_seconds.is_some() {
                    "seconds"
                } else if transport.pos_beats.is_some() && transport.tempo.is_some() {
                    "beats"
                } else {
                    "missing"
                },
            });
        }
        if self.runtime.delay_frames > 0 {
            ProcessStatus::Tail(self.runtime.delay_frames as u32)
        } else {
            ProcessStatus::Normal
        }
    }

    fn deactivate(&mut self) {
        self.probe.reset(&GROUPS_STATE);
        self.audio.reset();
        self.reporter = None;
    }
}

impl Drop for FlProbe {
    fn drop(&mut self) {
        if let Some(capture) = &self.capture {
            capture.unbind();
        }
        self.probe.reset(&GROUPS_STATE);
        if let Some(session) = &self.session {
            session.unbind(self.bound_member, &self.params);
        }
    }
}

impl ClapPlugin for FlProbe {
    const CLAP_ID: &'static str = if cfg!(cleanslate_podcast) {
        "org.cleanslate.podcast"
    } else {
        "org.cleanslate.fl-probe"
    };
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Captured Session analysis for synchronized dedicated microphones.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for FlProbe {
    const VST3_CLASS_ID: [u8; 16] = if cfg!(cleanslate_podcast) {
        *b"CleanSlatePodc01"
    } else {
        *b"CleanSlateProbe1"
    };
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nice_export_clap!(FlProbe);
nice_export_vst3!(FlProbe);

#[cfg(all(test, not(cleanslate_podcast)))]
mod identity_tests {
    use super::*;

    #[test]
    fn probe_identity_and_legacy_state_remain_compatible() {
        assert_eq!(FlProbe::NAME, "CleanSlate FL Probe");
        assert_eq!(FlProbe::CLAP_ID, "org.cleanslate.fl-probe");
        assert_eq!(FlProbe::VST3_CLASS_ID, *b"CleanSlateProbe1");
        assert!(
            !FlProbe::default()
                .params()
                .serialize_fields()
                .contains_key("org.cleanslate.podcast.state-version")
        );
        let mut state: PluginState = serde_json::from_str(
            r#"{"version":"0.0.1","params":{"probe_group":{"i32":1}},"fields":{}}"#,
        )
        .unwrap();
        FlProbe::filter_state(&mut state);
        assert!(matches!(state.params["probe_group"], ParamValue::I32(1)));
        assert!(matches!(
            state.params["session_mode"],
            ParamValue::Bool(false)
        ));
        assert!(matches!(
            state.params["enable_processing"],
            ParamValue::Bool(false)
        ));
    }
}
