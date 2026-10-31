# CleanSlate Smart Gate 0.0.1

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

Use `scripts/build-smart-gate.ps1 -RuntimeDirectory <verified-flat-runtime-directory> -RuntimeReceipt docs/licenses/directml-1.15.4-runtime.json`
from this product checkout. Both runtime inputs are explicit; this entrypoint and
`scripts/package-smart-gate.ps1` never download dependencies. The package helper
checks the runtime and all three official notices against the pinned observed
NuGet inventory, copies the DLL and notices next to both format binaries, and publishes a
new candidate with an atomic directory rename after copy/hash verification.
It also retains the repository's existing embedded-model, dependency and MSVC
notices. The staging fixture is
`scripts/test-package-smart-gate.ps1 -VerifiedRuntimeDirectory <verified-flat-runtime-directory>`;
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
The pinned inventory currently covers DirectML and its notices. The new Gate
DLL's actual imports still need inspection before deciding which MSVC files must
also be staged. Inherited development search paths can satisfy existing ABI
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

Reserved future Windows links are `%COMMONPROGRAMFILES%/CLAP/CleanSlate Smart Gate.clap`
and `%COMMONPROGRAMFILES%/VST3/CleanSlate Smart Gate.vst3`. They are distinct from
probe and Podcast paths. This ticket/build script creates no installed links.

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
