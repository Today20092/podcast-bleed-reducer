# GUI development

Multichannel WAV import is implemented in the development build. Import tracks or drop a WAV/BWF, choose the speaker channels and names, then import. The owner's 261005-T001.WAV passed a complete three-channel waveform/validation/reopen check on 2026-10-09 with Tr5/Tr6/Tr7 and an unchanged source hash. That check took 152.23 seconds while compilation ran in parallel; it is not an isolated performance benchmark or listening acceptance. The scoped suite passed 124 checks, strict all-target/all-feature Clippy passed, and the final GUI build took 14.03 seconds. Saved projects reference the original WAV.

From the repository root, run:

```powershell
pwsh -NoProfile -File scripts/gui-dev.ps1
```

Save and close the development window before running it again. The script refuses to rebuild over a running development executable. It builds only `rust-app`, uses one Cargo job, locked/offline dependencies and incremental local compilation, then opens a separate saved-project copy. The window title is `CleanSlate - GUI development`.

The default input is `test-audio/quick-test-10min/261005-quick-test.cleanslate`. On first launch, the script copies the small project into `%LOCALAPPDATA%/CleanSlate/Projects/GUI-development/<source-project-path-hash>/261005-quick-test.gui-dev.cleanslate`. Later launches keep that development copy and its edits. Original audio is referenced in place; the original saved project and its directory are untouched. Analysis/intermediate files use local application working storage, and waveform caches remain shared. Ordinary source validation, cache compatibility checks and playback preparation still run; automatic opening does not bypass them. Analysis is not forced. If the saved result is stale, use Analyze explicitly.

```powershell
# Build without launching or copying a project.
pwsh -NoProfile -File scripts/gui-dev.ps1 -BuildOnly

# Use another saved project; a separate local development copy is created.
pwsh -NoProfile -File scripts/gui-dev.ps1 -Project 'C:/path/session.cleanslate'
```

The script and `scripts/check.ps1 -Local` share `%LOCALAPPDATA%/CleanSlate/BuildCache` on Windows, outside all worktrees. `scripts/local-build-settings.ps1` owns the target and locked/offline, single-job Cargo flags; GUI `-TargetDir` remains an explicit override. Keep `rust-toolchain.toml`, Cargo.lock and `scripts/gui-dev.toml` consistent between builds. Incremental compilation stays enabled; dependency debug information is disabled to reduce disk use, so detailed dependency debugging needs a deliberate separate configuration. Cargo locks the shared target while compiling; coordinate builds and close the development executable before rebuilding. A fresh cache requires the dependencies to be available locally. Nemotron has been removed.

## Measurements on 2026-10-09

Speaker examples replace the exact-time input panel. The saved list uses minutes:seconds and short durations, with Play, Show on timeline and Delete. The waveform's example strip selects existing examples; its selected left/right handles resize the range. Right-click an example to change its speakers or delete it. Dragging previews the change, release saves one undoable edit, and Escape or leaving the waveform cancels. Conflicting ranges are rejected; speaker changes retain explicit conflict replacement. Analyze is still required after saved changes.

Track-header Solo buttons select any combination for monitoring. Turning every Solo off restores all tracks. Selection applies immediately in Original, Processed and Removed, survives stopping/starting playback, and resets when a session loads. It does not change project gain edits, analysis or WAV exports.

The development profile now optimizes the existing Lyon and Tiny Skia path-building/rendering packages as well as local drawing code. Continuous Lock to center scrolling rebuilds geometry every frame, unlike Page at edge. This profile change incurs a one-time dependency rebuild; keep the retained cache afterward. No dependency versions or release profile settings changed. Cargo's per-package profile behavior is documented in [profile overrides](https://doc.rust-lang.org/cargo/reference/profiles.html).

Playback buttons sit above the timeline. `Lock playhead` captures its current horizontal position and scrolls the waveform underneath it at the current zoom. Seeking keeps the lock; wheel/Ctrl-wheel zoom and middle-drag pan stop following without stopping playback. `F` fits the recording; `Shift+F` fits selection with context; `Alt+drag` fits a temporary range without changing selection, labels or gain. `Help → Keyboard shortcuts` or `Ctrl+/` opens the full reference; `Alt+H` reaches Help by keyboard. `Unlock playhead` leaves the viewport under manual control. `Center playhead` centers it without changing whether it is locked. The captured fraction survives window resizing; the viewport stops at the recording boundaries. Ownership percentage meters follow the playhead in a strip within each track header, retaining explicit stale status. `Meters: Playhead` toggles to `Meters: Right` to restore the previous placement during this trial. These controls are transient and do not change the saved project. The inspector groups references/analysis, sound and export separately, with model/source setup expandable. Whole-track-protection creation buttons are removed; saved protections still apply.

The read-only `scripts/probe-pan.ps1` compiles the current production waveform and gain canvases against retained dependencies and exercises 40 viewport moves on a saved project's source grids, with synthetic waveforms and envelopes. Combined software-geometry preparation measured median 15.08 ms/max 15.55 ms before renderer optimization, 13.64/16.56 ms after the first partial optimization, and 6.54/7.75 ms with path conversion optimized as well. A subsequent repeat during compilation measured 9.57/9.84 ms. The final idle comparison, using unchanged S-curve drawing/envelope code with retained old versus optimized renderer dependencies, measured 15.63/20.83 ms versus 7.30/9.58 ms. This measures geometry preparation, not full GPU rendering or display FPS. The probe writes only diagnostic compiler outputs in `.scratch/offline-supervised-editor/`, never project/audio/cache files; unused production loading/saving entry points are guarded by unreachable diagnostic stubs.

The earlier gain-only probe measured median 24.16 ms/max 27.54 ms before stroke batching and 9.53/10.04 ms after it. Each column retains its exact bounds and position. These earlier figures are not comparable to the expanded waveform-plus-gain workload.

The GUI configuration now uses optimization level 1 for local app/test code, retaining incremental compilation and the existing dependency settings. The unoptimized candidate spent a median 31.67 ms preparing drawing data for a synthetic three-track, ten-minute, 1900-column zoom workload. The identical workload took 9.22 ms with light optimization, with a 15.75 ms maximum across 24 zoom changes. These are production gain-query/curve kernels, not full GPU frames or a measured display FPS. The first test-profile transition compiled local code in 98 seconds; dependency compilation was not repeated. Subsequent GUI iteration timings below describe the earlier level-0 configuration and should not be assumed for level 1.

| Build | Measured time |
|---|---:|
| Initial main-checkout build into retained cache, nonincremental, including several dependency rebuilds | 72 seconds reported by Cargo |
| First incremental local build | 31.59 seconds |
| Subsequent small GUI title change | 9.82 seconds |
| No-change GUI build | 0.80 seconds |

These are observed build times on this machine, not a controlled clean-build speed comparison or a guarantee for every change. They exclude project opening and audio preparation. Cargo timing reports are saved under the selected target's `cargo-timings` directory.

Validation: 23 applicable app tests passed, one opt-in ignored; strict all-target/all-feature Clippy, formatting, diff and PowerShell parser checks passed. Launch produced a responding native development window. The saved owner project remained byte-identical, the separate development project initially matched it, and rebuilding while that window was open was correctly rejected. Physical playback/opening latency and listening were not measured.

## App menu

The top-bar Menu offers Settings, Keyboard shortcuts, Help, GitHub, Report a bug and About. `Alt+M` opens it; Tab or Up/Down selects an entry, Enter activates it, and Escape closes it. `Ctrl+/` still opens the reference, and `Alt+H` retains Help → Keyboard shortcuts. About shows the compiled version and application license; its Copy version info action includes only app/build/platform information. Tab switches between Copy and Close, while Page Up/Down and Home/End scroll the credits. Settings and About retain the workspace behind centered dialogs. Browser links open only after an explicit menu action; Report a bug opens the issue chooser for the user to complete.

Settings offers Dark and Light, timeline preferences and Keyboard shortcuts. Tab or Up/Down selects a control; Enter activates it. Dark is the default. Preferences are stored independently of projects in the user's local configuration directory. Invalid appearance falls back to Dark with visible feedback; a failed save keeps the selected appearance usable and reports the failure. Changing appearance redraws the interface without rebuilding audio or changing the project.

## Adjustable shortcuts

In the Who is talking? popup, the default `S` saves a new example or changes to an existing example. `Enter` remains a confirmation alternative. Speaker toggles default to `1`–`8`, Clear to `R` and loop pause/resume to `Space`. Workspace commands do not run through the popup; `Ctrl+S` (Command+S on macOS) retains project saving. Text entry, dialogs and capture own their input first. Holding an application command key does not repeat saves or toggle lock repeatedly.

Settings → Keyboard shortcuts lists command names, effective bindings and activation contexts. Choose Change, press a key or chord, review the proposed binding, then choose Apply or press Enter. Escape cancels capture. Conflicting commands and their contexts are named; commands in mutually exclusive contexts can reuse a binding. Text-editing, confirmation, focus-navigation and operating-system bindings are reserved. Restore default affects one command; Restore all defaults restores keyboard bindings. Neither silently displaces another command. Remove extra for project saving retains the fixed Ctrl/Command+S compatibility binding.

Center playhead defaults to `C`; Toggle playhead lock defaults to `L`. Both reuse the mouse actions. Center changes the viewport without seeking or changing selection, playback or lock state. Command tooltips appear after approximately 600 ms and use the current binding and applicable Lock/Unlock or Save example/Save changes name. The shortcut reference and popup hints use the same definitions as dispatch. Existing sound-value instruction tooltips remain absent.

User wheel zoom and all fit routes now preserve lock by default, superseding the historical zoom-unlocks behavior described above. A locked fit changes the scale while retaining the captured playhead anchor, subject to recording boundaries. Settings → Unlock playhead when zooming is off by default; enabling it releases an active lock when zooming or fitting, without enabling lock when it was off. This preference and shortcut overrides persist locally across restart and New project. Manual panning and background viewport changes retain their existing behavior; ticket 42 owns its separate planned middle-pan changes.

Missing entries use defaults. Invalid, conflicting or obsolete overrides produce actionable feedback while valid unrelated preferences remain usable. Reads never rewrite the file. Preference changes merge into the existing JSON object and save atomically; malformed JSON is preserved and must be repaired or backed up and removed before saving changes. A failed preference save leaves the current in-memory choices usable and reports the failure.

## Inspector B development pass

The selected B layout is implemented on `codex/gui-inspector`. The desktop inspector separates a compact status header, a scrolling controls body and a persistent Export WAV files footer. Compact windows place status and Export beside each other above the scrolling controls below the tracks, keeping Export reachable without increasing the panel height. Speaker examples retain Undo/Redo/Analyze and the expandable list; source blockers still expand Sources automatically. Analysis stage, elapsed time and Cancel remain in the status header. General job messages remain in the scrolling body; export errors stay beside Export in a bounded scrolling area.

Attenuation uses a vector dial beside the existing numeric editor, explicit Mute state and Reset. Drag vertically (4 pixels per dB), or use Up/Down while hovering, for the former slider's −30 to −3 dB quick-adjustment range. Clicking without dragging leaves the value and Mute unchanged. Numeric editing retains the full −120 to 0 dB range, validation, Enter/blur commit, Escape cancel, Ctrl-click reset and right-click reset menu. The timing sliders, typed ranges and audio update paths are unchanged. Arrow adjustment does not intercept active sound or track-name typing. No FabFilter code/artwork, dependency or audio algorithm was added.

The browser sketches remain separately preserved on `codex/gui-inspector-sketches` (`b28c841`); they are not part of this native implementation. Native visual and physical input acceptance must be recorded in ticket 23 before treating this as accepted appearance.

### Full FabFilter-inspired review pass

[Ticket 56](../.scratch/offline-supervised-editor/issues/56-fabfilter-inspired-gui.md) extends B to the owner's complete fourteen-item proposal. Commands use filled action styling, persistent modes use selected outlines/tints and a brief accent, values use inset monospace fields, and disclosures use quiet arrow labels. Analyze gains emphasis when valid examples need analysis; Export gains emphasis when the current result can be exported. Source review is explicit in the pinned status header. The expanded examples list has its own bounded scroll area.

Original / Processed / Removed form one segmented listening selector. Lock, Solo and Mute retain visible state independently of command emphasis. Selected tracks have an editing badge/border and waveform rail; real selection boundaries align across waveform/gain displays, and existing gain points and example handles have clearer hover/selection cues. Project actions remain visible with quieter styling after import. The waveform/example/gain gestures and all audio processing remain unchanged.

Tab / Shift+Tab navigate enabled workspace/dialog buttons and value fields; Enter or Space activates a button reached by keyboard. Pointer clicks clear keyboard button focus, preserving the existing playback Space command after clicking controls. Settings, About, Menu, shortcut capture and labeling retain their existing keyboard owners; inactive background workspaces do not receive focus or keyboard actions. Numeric editing retains its existing Enter/blur/Escape/Tab ownership.

Hover and selected-state feedback run for 90 ms; disclosures change height over 120 ms. Actions and audio changes apply immediately. These are widget-local redraws, with finite transitions and no new application Tick timer. Hidden/collapsing disclosure content cannot capture input/focus; offscreen disclosure animation does not request continuous frames. Window resizing, native UI scaling and timeline zoom remain independent.

The rendering audit found existing geometry caches keyed by revision/theme/size, uncached moving overlays, display-frame playback updates and conditional job polling. No measured defect justified changing the renderer. This pass adds original vector controls/shared styles, with no copied FabFilter code or artwork. See the investigation report recorded in ticket 56 and the primary [Pro-Q display](https://www.fabfilter.com/help/pro-q/using/eqdisplay) and [Pro-R active display](https://www.fabfilter.com/help/pro-r/using/decayrateEQ) references.

## Analysis feedback

Analyze reports source integrity checks, the current track during audio preparation, decoded recording position while making a new analysis copy, and track/recording position during speech detection. Cached copies are checked without displaying invented decoding progress. Later stages identify example checking, acoustic measurement, learning, timeline processing and preview construction. Elapsed wall time and Cancel analysis remain available throughout the job.

After successful result installation, the message area reports total elapsed time and measured source-check, audio-preparation, example-check, speaker-analysis and timeline times. Recording position describes audio already processed, not estimated time remaining. This feedback does not establish a processing speedup or long-session qualification.

## Hot reload assessment

Iced 0.14 supports experimental hot reload through `iced/hot` and `cargo-hot`. Its documented limitation is patching the root crate. Our executable delegates to `rust_app::app::run()` and the GUI lives in the library target. The current cargo-hot source selects the binary/example target and reruns its rustc invocation for patches. That makes ordinary library GUI edits a poor fit for the documented path. No runtime hot-reload trial or new dependency was added; a separate executable UI boundary would need investigation before adoption.

Sources: [Iced hot-reload documentation](https://github.com/iced-rs/iced/pull/3000), [cargo-hot implementation](https://github.com/hecrj/cargo-hot/blob/master/src/main.rs), [Cargo build performance](https://doc.rust-lang.org/cargo/guide/build-performance.html).
