mod analysis_audio;
pub mod app;
mod audio_io;
mod audition;
mod calibration;
pub mod detector_quality;
mod engine;
mod export;
mod joint_detector;
mod media;
pub mod model;
mod prepared_analysis;
mod project;
pub mod release_gate;
mod review;
mod timebase;
pub mod timeline;
mod vad;
mod waveform;

pub use analysis_audio::{
    ANALYSIS_RESAMPLER_RECIPE, ANALYSIS_SAMPLE_RATE, AUDIO_DECODER_RECIPE, AnalysisAudioIdentity,
    PreparedAnalysisAudio, prepare_analysis_audio, prepare_analysis_audio_in,
};
pub use audio_io::SourceMetadata;
pub use audition::{AuditionMode, AuditionPlayback, PreparedAudio};
pub use calibration::{CalibrationExample, CalibrationKind, OwnershipSettings};
pub use engine::*;
pub use export::*;
pub use prepared_analysis::{JointDiagnostics, PreviewCache, PreviewCacheStats, preview_stage};
pub use project::*;
pub use review::*;
pub use timebase::{
    Rounding, SourceTimebase, TICKS_PER_SECOND, TimePosition, TimeRange, time_from_seconds,
};
pub use waveform::SourceStamp;
