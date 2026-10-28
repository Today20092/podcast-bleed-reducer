mod acoustic_transfer;
mod acoustics;
mod analysis_audio;
#[cfg(feature = "desktop-ui")]
pub mod app;
// Core builds retain mixed cache/display modules; their desktop-only helpers have no callers.
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod appearance;
mod audio_io;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod audition;
#[cfg(feature = "desktop-ui")]
mod button_action;
mod calibration;
pub mod detector_quality;
mod editor_benchmark;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod editor_drawing;
mod editor_workflow;
mod effective_envelope;
mod engine;
mod export;
mod export_settings;
#[cfg(feature = "desktop-ui")]
mod gain_editor;
#[cfg(feature = "desktop-ui")]
mod gui_controls;
#[cfg(feature = "desktop-ui")]
mod gui_disclosure;
#[cfg(feature = "desktop-ui")]
mod gui_icons;
#[cfg(feature = "desktop-ui")]
mod import;
mod joint_detector;
mod media;
mod ownership_model;
mod package_check;
#[cfg(feature = "desktop-ui")]
mod parameter_dial;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
pub mod performance;
mod prepared_analysis;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod project;
mod references;
pub mod release_gate;
mod review;
mod session_ownership;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod shortcuts;
mod sound;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod storage;
mod storage_io;
pub use sound::SoundSettings;
mod supervised_render;
mod temporal_ownership;
mod timebase;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
pub mod timeline;
pub mod updater;
mod vad;
mod wav_delivery;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod waveform;
#[cfg_attr(not(feature = "desktop-ui"), allow(dead_code))]
mod worker_lifetime;

pub use acoustic_transfer::AcousticEstimatorComparison;
pub use acoustics::*;
pub use analysis_audio::{
    ANALYSIS_RESAMPLER_RECIPE, ANALYSIS_SAMPLE_RATE, AUDIO_DECODER_RECIPE, AnalysisAudioIdentity,
    PreparedAnalysisAudio, prepare_analysis_audio, prepare_analysis_audio_in,
    prepare_source_analysis_audio_in,
};
pub use audio_io::{AudioSource, SourceMetadata, wav_channel_sources};
pub use audition::{AuditionMode, AuditionPlayback, PreparedAudio};
pub use calibration::{CalibrationExample, CalibrationKind, OwnershipSettings};
pub use editor_benchmark::*;
pub use editor_drawing::{
    GainDrawingBucket, OverviewBand, OverviewState, OwnershipOverview, visible_automatic_buckets,
    visible_gain_buckets, visible_gain_curve,
};
pub use editor_workflow::*;
pub use effective_envelope::*;
pub use engine::*;
pub use export::*;
pub use export_settings::*;
#[cfg(feature = "desktop-ui")]
pub use gain_editor::{GainEditorEvent, GainPointer, hit_gain_point};
pub use ownership_model::*;
pub use package_check::package_self_check;
pub use prepared_analysis::{JointDiagnostics, PreviewCache, PreviewCacheStats, preview_stage};
pub use project::*;
pub use references::{
    IdentityReview, REFERENCE_MAPPING_RECIPE, ReferenceReadiness, SpeakerMapping,
    reference_readiness,
};
pub use review::*;
pub use session_ownership::*;
pub use supervised_render::*;
pub use temporal_ownership::*;
pub use timebase::{
    Rounding, SourceTimebase, TICKS_PER_SECOND, TimePosition, TimeRange, time_from_seconds,
};
pub use waveform::SourceStamp;
