# Sound presets and named project saves

`src/sound.rs` owns `SoundSettings`, factory values and the versioned custom `Presets` collection. `Project::apply_sound` in `src/project.rs` applies these values through `set_parameters`, the same transaction used by individual controls. That transaction owns validation, local persistence, sound undo/redo and render staleness. Mute stays outside `SoundSettings`; individual attenuation gestures explicitly clear it, while presets preserve it.

`src/app/presets.rs` presents the dialog and dispatches registered commands. The name and selection are transient dialog drafts. A stored collection changes in memory only after its candidate has been saved successfully. `sound-presets.json` lives beside `preferences.json`; malformed or unsupported stored data is preserved and disables custom writes until repaired. Factory presets remain usable.

## Add an eligible control

1. Decide whether the new control is reusable sound configuration. Source assignments, reference labels, training, protections and manual edits remain project data.
2. Add an eligible field to `SoundSettings` and extend `From<AutomixParameters>`, `validate`, `apply_to` and the factory definitions together. Domain validation also serves `prepared_analysis::validate_parameters` and typed UI sound entry. Add the control's existing UI metadata and rendering separately; preset CRUD, selection and file orchestration need no field-specific branch.
3. Introduce a new stored version when extending the preset schema. Decode the previous version at `Presets::load`, then normalize its available values against current project sound at application. Older presets leave controls absent from their version unchanged. Keep historical decoding out of GUI callers, reject newer unsupported versions, and retain a previous-version fixture. Version 1 currently has exactly four required fields; do not add a fake future field or migration until the schema changes.
4. Define project-load compatibility independently. Projects store applied values, not references to preset definitions, so changing factory values or deleting presets must not change saved sound.
5. Extend interface-level tests in `src/sound.rs`, `tests/project.rs` and the focused preset action test in `src/app.rs`. Cover invalid input without partial mutation, undo, preserved mute, saved project values and the real older-version fixture when introduced.

## Save and recovery ownership

Trust a saved recovery/destination association only when its explicit recovery marker matches the opened path and that path is within local application project storage. External project contents cannot redirect Save: opening an external file always selects its actual opened path, even if its JSON contains forged recovery metadata.

`src/app/project_files.rs` owns native chooser presentation. `Project::save` remains the automatic working-file save; `save_as` and `save_named` explicitly publish named snapshots. An explicit recovery-path marker and named destination/snapshot association live only in local recovery files. Named snapshots omit this association and reopen using their actual opened path, including inside the application's storage tree. `has_unsaved_changes` is cached during transactions rather than serializing projects on every redraw.

The named snapshot is the save commit. A failed named publication leaves its previous file and the in-memory destination intact. If named publication succeeds but writing the local association fails, the save succeeds and `save_warning` reports the recovery limitation. Keep the named file and retry Save after local storage is writable; the app cannot atomically publish two files on independent destinations. This warning does not claim the association is durable. Close/restart uses the same named-save flow and retains the session on cancellation or failed named publication.
