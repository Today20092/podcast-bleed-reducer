# CleanSlate Smart Gate 0.0.1

Unqualified pass-through prototype for one mono/stereo insert. Input samples remain
unchanged with Bypass on or off, with zero added latency/tail. Speech detection,
gain reduction and automatic volume leveling are not implemented. There is no
group/member, Session capture or desktop/trainer prerequisite.

The compact editor identifies the prototype and exposes the saved Bypass control.
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
| State | `org.cleanslate.smart-gate.state-version` = `1`; parameter `smart_gate_bypass` | Existing probe schema and parameters unchanged |

VST3 applies the existing SDK's Windows GUID byte-order conversion to these fixed
16 bytes. IDs are literal constants, independent of branch names, build paths or
random installation identifiers. Keep them stable when the implementation grows.
Invalid/missing/future state markers or a missing/wrong-type bypass parameter
reset this product to bypass; no probe settings or calibration are imported.
Valid Gate state round-trips its independent namespace and bypass value.

Prototype 0.0.1 has its own maturity under [the version policy](../../docs/releasing.md#versioning-and-stability).
It does not inherit desktop stable status. No signed/published package, VST2,
AAX/AU, native host or other-platform compatibility is claimed.

## Windows candidate workflow

Use `scripts/build-smart-gate.ps1` from this product checkout. It keeps locked,
offline dependencies, the shared local Cargo target, one build job, incremental
compilation and the global 14/8/6 GiB resource policy. It formats/checks/tests this
separate manifest and builds uniquely named actual-format check executables.
The graph contains no `rust-app`, ownership trainer or probe dependency.

Every successful build creates a new `dist/smart-gate-candidates/<unique-id>/`
with `CleanSlate Smart Gate.clap`,
`CleanSlate Smart Gate.vst3/Contents/x86_64-win/CleanSlate Smart Gate.vst3`
and a hash/source receipt. Earlier candidates and installed paths are untouched.
The entrypoint rejects linked or non-directory output parents before build setup
and rechecks them before creating candidate outputs.
These are development candidates, not a distributable signed installer.

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

Ticket 91 remains claimed until real compilation, format coexistence/state and
bypass checks pass. The initial default-policy test command refused before
starting a compiler because available memory was below 14 GiB. That is a resource
refusal, not semantic test red/green. Offline lock generation and locked metadata
resolution passed; Rust type/ABI correctness, executable UI and native acceptance
remain unverified until the guarded checks can run.
