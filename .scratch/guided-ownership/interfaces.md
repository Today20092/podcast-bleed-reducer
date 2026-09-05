# Shared implementation contract

The parent seeds public serde types in calibration.rs: CalibrationKind::{RoomTone,SoloSpeech}, CalibrationExample { kind, track, start, end }, OwnershipSettings { enabled, examples, sources }. SourceStamp reuses the existing ordered path/size/mtime representation. Default OwnershipSettings is disabled with empty examples/sources for old project compatibility. Project::create opts into joint mode.

Ticket01 owns project/editor lifecycle and calibration.rs methods, and tests. Ticket02 owns joint evidence/classification, prepared_analysis, engine/report and its tests. Avoid independently changing these names. Parent resolves integration and adds the final editor-to-engine calls after both branches exist, so each task may stay green without a speculative stub.

Ticket02 supplies:
- PreviewCache::analyze_with_ownership(paths, protections, parameters, ownership: &OwnershipSettings, progress, cancelled), retaining analyze as legacy wrapper.
- run_with_ownership(request, progress, parameters, ownership: &OwnershipSettings), retaining run_with_parameters as legacy wrapper.
- If preview diagnostics are added, use a separate optional accessor rather than adding mandatory fields to every AnalysisPreview literal unless needed.

Ticket01 supplies:
- Project::ownership_settings() -> &OwnershipSettings.
- Project::set_ownership_settings(settings) -> io::Result<()>, with validation and stale render handling.
- UI controls/selection/examples write project settings and call existing start_preview to queue a refresh; parent swaps worker/render calls to the new entry points.

Examples use half-open sample ranges. Non-empty examples require matching ordered source stamps. Empty examples permit automatic estimates. Invalid track/range, too-short unusable examples, nonfinite values or changed source stamps must fail safely. SourceStamp fields remain opaque; SourceStamp::read is the constructor. Full render still uses current validation.

Warm settings changes may derive from prepared facts but must not reread PCM/reinfer. Parent may add settings-aware generation comparisons to the editor during integration. Every task records exactly what it implements versus parent wiring still pending.
