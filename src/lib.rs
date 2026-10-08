pub mod app;
mod audio_io;
mod audition;
mod calibration;
pub mod detector_quality;
mod engine;
mod export;
mod joint_detector;
pub mod model;
mod prepared_analysis;
mod project;
pub mod release_gate;
mod review;
mod timebase;
pub mod timeline;
mod vad;
mod waveform;

pub use audio_io::SourceMetadata;
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
