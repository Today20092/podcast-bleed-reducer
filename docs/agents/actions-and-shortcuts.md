# Actions and shortcuts

The standalone FL probe uses a separate egui editor and cannot dispatch desktop Iced actions. Its waveform navigation commands are declared once in `plugins/fl-probe/src/session_editor.rs` (`Navigation::control`/`navigate`), shared by buttons, focused keyboard handling and the context menu. Its source/range canvas gestures, navigator and Follow mode are local viewport editing controls; they do not seek the host, modify source audio or register global desktop shortcuts. Host transport remains in the DAW. Customizable plugin shortcuts are not currently provided.

New user commands must be registered before they can be attached to a button.

1. Add one entry to `commands!` in [shortcuts.rs](../../src/shortcuts.rs), supplying its stable identifier, name, activation context and default bindings. Use `[]` when no default is appropriate. Never rename an existing identifier: it is a saved-preferences key.
2. Handle the identifier in `CleanSlateApp::dispatch_shortcut`. Its exhaustive match rejects a missing handler. Check current availability there through `action_available`, before any state change. Preserve validation in the operation that owns the data.
3. Build its button with `self.action_control(command, button(content).style(...))`. This supplies the activation message, current availability and effective-binding hint together. Do not attach a raw application message or separately write shortcut text. Settings, the shortcut reference, conflict checking and persistence already enumerate the same declaration.
4. Verify the action through application input and preferences: initially bound or Unbound, change, conflict, restart, clear, restore and unavailable contexts. An unbound action is still discoverable. Do not invent an arbitrary default just to make it appear in Settings.

`Center`, `Lock`, `CatchUp` and `Meters` demonstrate this path. Meters has no default. Both pointer activation and keyboard resolution reach the same handler. These workspace actions cannot run through text entry, shortcut capture, import/export dialogs or labeling/gain/application popups. Existing input handling still suppresses repeated and captured key presses.

The shared button's activation methods accept the `ButtonAction` trait, not arbitrary `Message` values. Only the private registered-button adapter and the closed `LegacyButton` exception enum implement that contract for the application. Adding a new message does not make it a button action. Existing exceptions cannot be used as generic wrappers for new messages. Do not add a blanket implementation or construct native Iced buttons to bypass the contract.

`scripts/test-action-contract.ps1`, invoked by `scripts/check.ps1`, compiles a deliberately unregistered action and requires E0277, then adds its explicit registration and requires successful compilation. The normal Rust build checks the actual button call sites, exhaustive command handler and declaration-generated metadata. A supplementary check rejects direct native-button construction; source matching is not the activation guarantee.

## Existing exceptions

The `legacy_buttons!` declaration in [app.rs](../../src/app.rs) is the individually named inventory: each entry supplies its argument types, reason and exact application-message mapping. It is an explicit migration boundary, not permission to exempt future commands. Each variant preserves the existing handler and its validation; no global shortcut is allocated to a transient item. The remaining migration scope is these named entries, reviewed one action/context at a time:

| Existing controls | Reason / remaining scope |
| --- | --- |
| `AnalysisDetails`, `SetupDetails` | Disclosure navigation. |
| `Appearance`, `CalibrationTiming`, `Mute`, `WaveformDisplay` | Existing value/view selections; consider explicit registered value actions individually. |
| `EditSound`, `ResetSound`, `EditTrackName`, `SaveTrackName`, `CancelTrackName` | Numeric or text editing owns the input. |
| `ApplyShortcut`, `CaptureShortcut`, `RestoreAllShortcuts`, `RestoreShortcut`, `UnbindShortcut` | Current Settings/capture operation with an explicit registered-command argument. |
| `CancelChannels`, `ConfirmChannels`, `CancelExportDialog`, `ExportDestination`, `SubmitExport` | Current import/export dialog navigation and confirmation. |
| `CloseHelp`, `DismissReferenceMenu`, `KeyboardShortcuts`, `Menu`, `Settings`, `About`, `OpenLink`, `CopyVersion` | Current menu/dialog navigation or its explicit built-in link/diagnostic operation. |
| `ContextReference`, `RemoveReference`, `PlayExample`, `ShowExample` | A current label identity/range/speaker set is required; preserve stale-item checks. |
| `DeleteRegion`, `GainAction`, `InsertGainPoint`, `ProtectSelection` | A current validated track/range/gain selection is required. |
| `RebindSource`, `Relink`, `SoloTrack` | Explicit current source/track argument and bounds/identity checks. |
| `RemoveImport`, `RetryImport` | Explicit pending-import identity. |
| `AnalyzeReferences`, `CancelAnalysis`, `Cancel`, `CancelPendingImports`, `Validate` | Legacy worker/source commands; migrate with the operation's availability policy. |
| `ImportStems`, `NewProject`, `OpenProject`, `SaveProject`, `CopyProjectPath`, `Export` | Legacy project/file operations; preserve native chooser, dirty-state and export safeguards. |
| `Audition`, `Stop`, `Previous`, `Original`, `Processed`, `Removed` | Legacy transport/listening routes, including reference-loop differences. |
| `UndoGain`, `RedoGain`, `DeleteGain`, `UndoReferences`, `RedoReferences` | Existing registered keyboard actions with legacy button routes; preserve distinct editing contexts. |

Non-button editing controls also retain explicit exemptions: `UnlockOnZoom` is a Settings preference; `ImportChannel` and `ImportChannelName` edit the current import choice; `TrackName` and `SoundText` edit text; `GainTool` and `ReplaceGainConstraints` change the editing mode/policy; `SuppliedAlignment` confirms source setup; `ReferenceSpeaker` edits the current labeling popup. `ExportLayout` and `ExportEncoding` select current export values. `Attenuation`, `OpenTime`, `CloseTime`, `MinimumGap` and `MonitorGain` are continuous/precise values with their existing input ownership. Timeline/example/gain canvas gestures carry the current source, range or point identity and retain their existing validation rather than becoming arbitrary global commands.

Any new non-button actionable control must likewise choose a registered action or add an individually justified editing/navigation exemption here. Parameterized commands must resolve and validate the current item at activation time; never persist a transient item index as a global shortcut.
