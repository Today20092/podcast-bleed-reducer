# Lightweight autonomous plugin validation

Status: done
Assignee: Codex
Type: task
Blocked by: none
Related: 67, 75, 76

## Owner request — 2026-10-24

Automate routine plugin building/validation without FL Studio, Resolve or another full DAW and without requiring the owner to remain present. Prefer low-memory command-line hosts and recorded-schedule regression cases; reserve native smoke tests for host-specific behavior that the harness cannot establish.

Owner chooses broad coverage first: VST3 and CLAP across Windows, macOS and Linux; VST2 remains a legacy follow-up with adapter/licensing unresolved. AAX/Pro Tools and AU/Logic are outside current scope unless future users request them. Native builds/tests on other operating systems remain required before compatibility claims; a Windows headless pass is not all-platform qualification.

## Scope and acceptance

- [x] Extend actual-binary checks from CLAP to VST3 processing/parameter discovery without an editor or device.
- [x] Run both headless hosts automatically against the candidate before publishing to development links.
- [x] Add a deterministic regression from the observed CLAP export scheduling failure, without retaining recording contents.
- [x] Measure runtime/memory of headless checks and bound execution with sequential runs/timeouts.
- [x] Document automated versus native-host/physical/listening coverage and optional standard validators from primary sources.
- [x] Run appropriate checks and record receipts. No DAW UI automation or new public compatibility promises.

## Results — 2026-10-24

Added `plugins/fl-probe/examples/vst3_host_check.rs` using exact vst3 0.3.0 bindings (already present transitively, now an explicit dev dependency). It loads the real Windows VST3 factory/component/controller/processor, creates two instances, discovers/sets all three published controls and verifies createView intentionally returns null, then checks zero latency and bit-exact mono/stereo output across 44.1/48 kHz, realtime/offline setup, variable blocks, separate/in-place buffers, missing timing and seeks. References and plugin threads are released before DLL unload. It verifies Ready shared PCM windows and finalized loss-free reporting. This is a focused adapter check, not a complete VST3 SDK validator: full automation queues, VST3-specific saved-state/GUI/host-context contracts remain future coverage.

`scripts/test-plugin-hosts.ps1` runs CLAP then VST3 with no window/audio device/DAW. Each child has a 30-second timeout, captured output and nonzero-exit/no-PASS rejection. Working-set peaks are polled every 10 ms and reported as observed samples, excluding build tools and parent PowerShell. `build-fl-probe.ps1` now builds examples and calls this runner before publishing the candidate. It uses PowerShell 7; the one-time link setup remains compatible with the owner's administrator Windows PowerShell.

Measured successful run: CLAP 0.32 seconds, 10.98 MiB largest sampled peak working set; VST3 0.33 seconds, 12.04 MiB. These are observations for the two-instance dry prototype, not inference-model or compiler memory budgets. CLAP passed 1,858 bit-exact blocks, 1,168 shared windows, old/current state checks and render-mode/manual-label tests. VST3 passed 1,648 bit-exact blocks, 1,136 shared windows and parameter/editor/processing checks. Ten unit tests pass, including a synthetic regression for the observed 73,072-sample CLAP export timing skew: safely reject that mismatch and recover on matching timestamps. This does not solve or qualify the native CLAP export route.

Plugin formatting and strict all-target Clippy pass. Both scripts execute successfully; PowerShell parser/diff checks pass. Desktop sources/manifest/lockfile are unchanged; the preceding 370-pass/20-ignored desktop check receipt remains separate rather than rerunning unrelated application tests. No native host or computer control was used for this delivery.

Base source remains development `88dda154daa9f442018302e19f1c792513fd0874` plus uncommitted changes. Tested CLAP and VST3 content SHA-256 is `3f20ffa26616a60069da861087bef65ba219cbae71438a064f733716c0b81ef7`, installed through the same linked `dist/fl-probe/` artifacts after checks; the previous `073d5b...` build remains preserved under the existing VerifiedBuilds path. No DSP behavior or plugin version/identity changed; the library rebuild includes the additional test source metadata. Older native evidence retains its original hash identity.

Primary-source options: [clap-validator CLI](https://github.com/free-audio/clap-validator#usage), [pluginval headless mode](https://github.com/Tracktion/pluginval#running-in-headless-mode), [VST3 bindings](https://github.com/coupler-rs/vst3-rs). Independent validators were researched, not downloaded/run; no validator pass is claimed. Owner explicitly defers AAX/AU, prioritizes VST3/CLAP and keeps VST2 as a legacy follow-up. Existing four-target desktop matrix is the plugin build goal; only Windows x64 has actual plugin artifacts/evidence. macOS/Linux need reporting-path, packaging and native-runner work. GUI gestures, listening and real-host scheduling remain occasional native checks. Closed scope here is lightweight autonomous testing infrastructure; ticket 76's CLAP failure and ticket 67's calibration/inference goals stay open.
