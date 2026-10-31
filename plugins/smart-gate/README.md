# CleanSlate Smart Gate 0.0.1

## Engineering waveform editor (2026-10-31)

The editor shows 5.12 seconds of original (gray) and processed (teal) peak
envelopes from real host audio. Purple is the maximum applied reduction in
each matching 10 ms interval. IN/OUT meters are in dBFS; the header shows
current reduction in dB. Stopped callbacks clear current meters after 250 ms;
the retained history stays available to inspect. No input produces an empty
display, not a demonstration waveform.

Bypass preserves original audio. Arm is temporary and does not clear Bypass.
Range sets the maximum reduction between 0 and 6 dB and is host automatable;
restoring a saved project restores a valid Range while forcing safe Bypass.
Freeze pauses only the history display; live meters and processing continue.
Original/Processed toggle drawing independently. Fixed detector thresholds,
hold/ramp timing and evidence expiry are described below the controls.

The historical checkpoints below retain their original verification scope.

The last checked native candidate is a pass-through prototype for one mono/stereo
insert. Current ticket 93 source connects the embedded speech detector, bounded
worker and finite gain policy. Current engineering source passes 25 library/editor
tests, including actual callback-fault feedback and CPU captures of the real
editor. No new native
candidate has been built or qualified from this source. Automatic volume leveling
is outside scope; there is no group/member, Session capture or desktop prerequisite.

New/restored instances start bypassed. Processing additionally requires temporary
explicit arming through the current editor or Rust engineering API. The editor's
Arm / Disarm action is temporary and never changes saved Bypass. Activation/reset
boundaries retire an in-flight arm click; deactivation makes arming unavailable.
Headless pointer and keyboard checks pass; no new native editor is qualified.
The -6 dB floor, confidence thresholds and 500 ms offline wait are engineering
limits, not sound acceptance. Missing/invalid/stale evidence preserves original
audio. Stereo uses two independent detectors and the greater confidence, applying
one envelope to both outputs. No audio delay is introduced; detector/delivery
delay and speech-onset preservation still need native measurement/listening.

The compact editor identifies experimental processing, saved Bypass and temporary
consent. It reports unavailable preparation and original audio while bypassed;
armed consent does not claim current inference or gain reduction.
Callback faults report original-audio fallback independently of consent. Fault
changes and lifecycle updates use the existing atomic editor repaint notifier;
native wake behavior remains unqualified.
The editor distinguishes waiting for data from current detection data for the
last processed block. Valid absence counts as current data; this is not a claim
of speech presence, continuous host playback or sound quality. Reset retires old
data, and source-age expiry uses the same predicate as gain application.
The full accepted [Gate design](../../docs/design/plugin-ui-contract.md) belongs
to ticket 94 after detector/DSP qualification. A successful build is neither
native UI acceptance nor evidence of a working speech gate.

## Independent identities

| Identity | Smart Gate | Preserved probe |
| --- | --- | --- |
| Cargo package/library | `cleanslate-smart-gate` / `cleanslate_smart_gate` | `cleanslate-fl-probe` / `cleanslate_fl_probe` |
| Product name | CleanSlate Smart Gate | CleanSlate FL Probe |
| CLAP ID | `org.cleanslate.smart-gate` | `org.cleanslate.fl-probe` |
| VST3 raw class bytes | ASCII `CleanSlateGate01` | ASCII `CleanSlateProbe1` |
| State | `org.cleanslate.smart-gate.state-version` = `2`; parameter `smart_gate_bypass` | Existing probe schema and parameters unchanged |

VST3 applies the existing SDK's Windows GUID byte-order conversion to these fixed
16 bytes. IDs are literal constants, independent of branch names, build paths or
random installation identifiers. Keep them stable when the implementation grows.
All restored Gate states force bypass; no saved state grants arming consent.
No probe settings or calibration are imported.

Prototype 0.0.1 has its own maturity under [the version policy](../../docs/releasing.md#versioning-and-stability).
It does not inherit desktop stable status. No signed/published package, VST2,
AAX/AU, native host or other-platform compatibility is claimed.

## Windows candidate workflow

Use `scripts/build-smart-gate.ps1 -RuntimeDirectory <verified-flat-runtime-directory> -RuntimeReceipt docs/licenses/directml-1.15.4-runtime.json -MsvcRuntimeDirectory <verified-x64-CRT-directory>`
from this product checkout. Runtime inputs are explicit; this entrypoint and
`scripts/package-smart-gate.ps1` never download dependencies. The package helper
checks the runtime and all three official notices against the pinned observed
NuGet inventory, copies the DLL and notices next to both format binaries, and publishes a
new candidate with an atomic directory rename after copy/hash verification.
It also retains the repository's existing embedded-model, dependency and MSVC
notices. The staging fixture is
`scripts/test-package-smart-gate.ps1 -VerifiedRuntimeDirectory <verified-flat-runtime-directory> -VerifiedMsvcRuntimeDirectory <verified-x64-CRT-directory>`;
it loads no native code and uses a synthetic plugin solely to test copying.
See [the provenance record](../../docs/research/gate93-directml-provenance-2026-10-30.md).

The build entrypoint keeps locked,
offline dependencies, the shared local Cargo target, one build job, incremental
compilation, global single-compiler coordination, timeout, cancellation and owned
cleanup. The owner's 2026-10-30 override removes RAM/commit launch thresholds,
running reserves and job memory caps; memory metrics are informational. The shared
execution wrapper and product host caller use this revised policy.
The entrypoint formats/checks/tests this
separate manifest and builds uniquely named actual-format check executables.
The graph contains no `rust-app`, ownership trainer or probe dependency.

Every successful build creates a new `dist/smart-gate-candidates/<unique-id>/`
with `CleanSlate Smart Gate.clap`,
`CleanSlate Smart Gate.vst3/Contents/x86_64-win/CleanSlate Smart Gate.vst3`
and a hash/source receipt. Earlier candidates and installed paths are untouched.
The entrypoint rejects linked or non-directory output parents before build setup
and rechecks them before creating candidate outputs.
These are development candidates, not a distributable signed installer.
The pinned inventories cover DirectML, its notices and the four imported MSVC
runtime DLLs observed in the actual Gate artifact, using the installed official
VS Build Tools redist bytes. The VST3 bundle also retains the project license and
existing model/dependency/MSVC notices under `Contents/Resources`.
Inherited development search paths and preloaded host runtimes can satisfy existing ABI
checks, so these checks do not prove candidate-local runtime completeness.

`scripts/test-smart-gate-hosts.ps1 -PluginDirectory <candidate>` reuses the
maintained guarded host runner's timeout, PASS contract and per-child isolated
application-data cache. It requires a preserved probe binary, defaulting to the
shared target's `debug/cleanslate_fl_probe.dll`, or explicit `-ProbeBinary`.
The new `smart_gate_clap_check` / `smart_gate_vst3_check` executables load both
products, check distinct factory identities and state round-trips, and exercise
Gate mono/stereo, bypass, exact input/output bits, latency and block layouts.
Their isolated caches remain available for review. No DAW, audio device or
native editor window is opened by these tests.

`scripts/test-smart-gate-loader.ps1 -PluginDirectory <candidate> -HostDirectory <preserved hosts>`
prepares fresh copies of each candidate and host outside the Cargo cache. Each
dedicated host process uses an empty working directory, isolated application
data and System32-only PATH. Its `--candidate-loader <binary>` mode excludes
application-directory, CWD and PATH DLL search, adds only the candidate binary's
directory, and never loads the probe. Before loading Gate it rejects preloaded
DirectML; after activation it checks the actual loaded DirectML path. The script
validates both candidate sidecars against the reviewed DirectML inventory before
any host execution, checks the loaded SHA256 against that same inventory, and requires a second fresh process
without the staged DirectML sidecars to fail. Fixtures and child logs are retained.
The host reports the four imported MSVC CRT modules already loaded at startup,
including their actual paths; these may mask candidate CRT sidecars.
`scripts/test-smart-gate-loader-preflight.ps1` verifies a same-length tampered
runtime is rejected before any host process starts, using only a disposable file.

This explicit `AddDllDirectory` strategy verifies an engineering loader only.
It does not establish that FL Studio, OBS or another ordinary host discovers the
sidecars, nor does it prove candidate-local CRT loading, arming, inference or
realtime behavior. Loader source `cdf5ce0` passed both format positives and
classified missing-sidecar controls: both System32 DirectML fallbacks were
explicitly rejected with exit 42. The runner accepts only a defined runtime-path
rejection (exit 42) or recognized missing-DLL dependency (Windows error 126,
exit 43), with exact diagnostics; generic panics cannot pass. Earlier `80eac5c`
negatives remain unqualified because its runner accepted any nonzero exit.
See ticket 93 for the exact candidate, module paths, limits and receipts.

The build script creates no installed links. The separate reviewed
`scripts/link-smart-gate.ps1` can link a verified preview as a local development
install. On this machine the global CLAP folder denied access, so the official
user locations now contain Gate-only junctions: `%LOCALAPPDATA%/Programs/Common/CLAP/CleanSlate Smart Gate/`
and `%LOCALAPPDATA%/Programs/Common/VST3/CleanSlate Smart Gate.vst3`.
The follow-up per-format delivery also created the documented FL route at
`%COMMONPROGRAMFILES%/VST3/CleanSlate Smart Gate.vst3`. The identical user VST3
duplicate remains because automatic approval review rejected link-only cleanup;
all three junctions target the same verified immutable preview.
New/restored instances remain bypassed and unarmed. Probe/Podcast links are
unchanged. [Exact paths, hashes, scan guidance and rollback](../../docs/research/gate93-local-development-delivery-2026-10-30.md).
Actual host discovery and native/armed behavior remain separate acceptance.

### Prepared native VST3 consent check (unverified)

For ordinary GUI development, the separate Windows mode
`smart_gate_vst3_check.exe --gui-preview <Gate VST3 binary>` uses the plugin's
actual default bypass on and fresh unarmed instance. It loads the real editor,
shows a static preview label, and stays open until the user closes it. It does
not start audio processing, render synthetic audio, open a device or claim armed
acceptance. Real editor controls remain interactive. Use the checked preserved
host with the separately prepared/installed GUI-preview plugin.

Root performs a bounded startup watch for `GUI_PREVIEW_READY`, then leaves the
interactive preview outside the global compiler mutex so GUI work does not block
builds. Close produces `GUI_PREVIEW_CLOSED` after normal teardown. Do not run this
user-close mode inside the bounded acceptance wrapper for a long GUI session.
Compilation/normal regression must pass before the launch handoff; native
presentation remains observational until that launch is inspected.
The checked preview host is preserved under
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261030-gui-preview/` from source
`aaf9f44`; selected build, strict example Clippy, formatting and normal coexistence
regression pass. Use `host-artifact.json` there for its SHA256 and installed Gate
argument. This worker has not launched the preview; root owns the UI handoff.

The Windows-only optional mode `smart_gate_vst3_check.exe --native-editor <candidate VST3 binary>`
embeds the actual editor in a new host-owned parent. It first configures bypass
off through the ABI, starts processing, and checks an unarmed unity baseline.
Click the actual editor's **Arm experimental gate**, then the separate **Run
armed ABI check** host button. That button only requests synthetic processing;
it does not arm the plugin. The same active session remains live until close.
Real bypass checkbox edits are forwarded at the next block through the VST3
parameter-change interface. No audio device is opened.

After the scoped build/launch grant, use a new isolated evidence directory and
the existing timeout-only wrapper, for example:

```powershell
$run = Join-Path $env:LOCALAPPDATA ("CleanSlate/NativeConsentChecks/" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $run | Out-Null
& ./scripts/resource-guard.ps1 -FilePath '<preserved-host>/smart_gate_vst3_check.exe' `
    -ArgumentList @('--native-editor', '<candidate>/CleanSlate Smart Gate.vst3/Contents/x86_64-win/CleanSlate Smart Gate.vst3') `
    -WorkingDirectory $run -ReceiptPath (Join-Path $run 'native-consent.json') -TimeoutSeconds 180
```

The window ends after 120 seconds or explicit close. A run records 36,000 stereo
samples at 48 kHz in offline mode, gain extrema/final gain, marked sample alignment
and reported latency/tail. A PASS requires actual attenuation and successful
teardown. Retain the log plus native editor screenshots showing the actual
consent/status. The editor requests its existing 640×440 logical size; the host
adds a 48-unit control strip and uses default process DPI behavior. No global
display setting changes. Physical presentation, realtime performance, ordinary
DAW/PDC behavior and listening acceptance require separate evidence. This mode
compiled successfully at `1297164`, with strict example Clippy and formatting
passing. The bounded native attempt reached an exact-unity baseline and editor-
ready log, but Windows automation was unavailable and no consent/input was sent.
It ended at its own timeout with exit 101 and verified cleanup; real arming,
native presentation and armed output remain unqualified. The ordinary coexistence
invocation subsequently passed after the shared ABI block extraction, without
opening a native editor.

## Source/checkpoint

Product source starts at Gate integration `3bbda73010143f25a36de78a8bac56e5dac89bf3`.
Canonical planning/design/host-matrix records were selectively synchronized from
`bb356cb`; the shared guard already matched that revision. No development branch
merge or probe source change was used. Current API evidence combines the
Context7 NIH-plug predecessor reference and egui 0.37.0 documentation with the
installed nice-plug 0.4.2 / nice-plug-egui 0.6.0 source; the fork source controls
the actual trait signatures.

Ticket 91 is done for its preserved pass-through candidate. Ticket 93 remains
claimed: current engineering source passes 25 library/editor tests and formatting, including
real offline non-unity callback processing, mono/stereo parity, block-size
independence, bypass/reset, invalid-input fallback and actual pointer/keyboard
consent interaction, latched callback-fault display and reset recovery. Strict
library/test Clippy passes. Four actual-widget CPU captures at 640×440 / 1× were
inspected; they do not establish native font/DPI, wake timing or performance. Realtime
allocation/deadline, actual CLAP/VST3 gating, native arming/readiness, standalone
runtime packaging and listening acceptance remain open. No installed path or old
candidate was replaced. See the canonical tracker/report for current receipts.
