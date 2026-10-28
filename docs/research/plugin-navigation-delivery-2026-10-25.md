# Plugin waveform navigation, 2026-10-25

Ticket: [84](../../.scratch/offline-supervised-editor/issues/84-plugin-waveform-navigation-performance.md). Source: development at `88dda154daa9f442018302e19f1c792513fd0874` plus uncommitted work. No commit, release or vendor installation.

## Implementation

CleanSlate FL Probe 0.0.6 adds pointer-anchored wheel/Ctrl-wheel/pinch zoom, Shift/horizontal-wheel pan, middle-drag pan, Alt-drag range zoom, Fit selection, a captured-time navigator, playhead/Center and optional Catch up/Lock. Buttons, focused keyboard commands and the context menu use one local navigation dispatch. DAW transport remains host-owned. Bounds preserve the viewport width at zero/two hours rather than collapsing it. Normal selection and navigation are independent.

The UI retains its last display snapshot when the capture worker holds the state lock; captured data refreshes at most every 33 ms. Open-editor polling during playback is 16 ms instead of 80 ms. This is a scheduling interval, not measured presented FPS. Mesh batching reduces per-peak drawing objects. Incremental eight-way peak summaries bound overview work without changing disk-cache format or PCM validation. Their additional storage is approximately one seventh of base peak storage (about 2.94 MiB for eight two-hour peak indexes), excluding containers/allocator overhead.

Native testing exposed a fast-drag bug: press/move/release delivered between redraws lost the selection. `fast_drag_released_between_redraws_still_selects_source_time` failed before the fix. Gesture ownership now starts from actual pointer press events and honors a release in the same frame, including the focused Fit selection shortcut. Existing multi-frame pan/range tests still pass. Native testing also exposed misleading setup advice after switching legacy state: the missing-capture branch hid Restarting. Restart/binding errors now precede setup advice and explain saving/reloading when the host does not reinitialize. Session now explicitly paints the application background instead of inheriting a black host clear color.

## Verification

Final `scripts/build-fl-probe.ps1` software stages passed: **59 tests**, formatting, strict all-target Clippy, plugin/examples and both actual SDK hosts. CLAP checked 1,858 bit-exact blocks/1,168 shared windows; VST3 checked 4,048 blocks/3,392 windows. Sampled final test-process peaks: CLAP 20.75 MiB/0.51 s, VST3 24.57 MiB/0.92 s. These unchanged-audio ABI scenarios do not qualify native learned attenuation.

Event tests cover pointer anchoring, selection preservation, middle pan, Alt range zoom, boundary spans, one-frame complete drags/focused F, worker contention and visible restart/binding errors. Peak-summary queries match exact naive ranges after sparse edits and a rebuild. Actual CPU renders at 1000×680 and 720×560 were inspected; captures are in `%LOCALAPPDATA%/CleanSlate/SessionUiCaptures-0.0.6/`. Impeccable's mechanical detector returned no findings. `git diff --check` passed for scoped changes. Root desktop code/algorithm is unchanged in this follow-up; the previous root checks were not rerun.

Timing observations, same machine, 40 frames, eight tracks/1,000 columns: old CPU waveform draw+tessellation 4.328 ms/frame and 8,038 shapes; new observations 1.332–1.880 ms/frame and 46 shapes. Two-hour view extraction changed from 4.448 ms to an observed 1.842 ms; ten-second observations were 0.179–0.247 ms. These are small, nonisolated synthetic observations, not physical input latency, native FPS or a formal vendor comparison. No two-hour native session measurement was performed.

## Native evidence and publication boundary

Used the existing FL Studio 2026 installation and a copied project, `C:/Users/User/Documents/Image-Line/FL Studio/Projects/cleanslate_testing/cleanslate_navigation_0.0.6.flp`. Original project was not overwritten. Switched two old saved instances to Session, saved the copy via File → Save, then reopened it via the recent-project menu because FL did not reconnect the live mode change. Both Session lanes then captured actual looped audio and displayed the host playhead. Wheel zoom during playback changed one editor from 00:00–02:00.43 to 00:52.21–01:55.71 while the other editor retained its viewport. The fast native drag test did not select; the resulting regression led to the final fix above. Keyboard forwarding and other native gestures remain unqualified.

That native pass used the first checked 0.0.6 binary `bb90e09e18a61b9dd9c5d7b722835ebe76c6a192fe73d70db067a5a29cf49c05`. The final checked candidate with fast-drag/connection/background fixes and readable minimum lane height at compact sizes is SHA256 **`94cf0dc682447677e1d942397dfc8aea929c9fe697a5a0035127968dace54aaa`** for both files in `dist/fl-probe-candidate/`. The script refused to replace loaded development files because FL still holds `dist/fl-probe/CleanSlate FL Probe.clap` open. Automated close/Exit attempts left the host running; the owner was asked to close it. Test playback was stopped through the host transport. The old checked binary remains installed until publication succeeds. Do not call the final candidate installed or natively accepted until the hashes match and it is retested.

Prior 0.0.5 is preserved under `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/fl-probe/bcdd98812733ceb440ab61a899935631fd47d3f5ff053ca240e80848a99f1548/`.

### Final installation

After the owner confirmed FL Studio was closed, the separately launched navigation-test process still held both files. Its exact PID/name/copied-project title were checked before stopping that dedicated test process. The build script then passed again and published the final **94CF0D** binary to `dist/fl-probe/`. Both existing Common Files VST3/CLAP development links were hashed and match the full final SHA256 above. The first checked 0.0.6 **BB90E0** binary is now preserved under `VerifiedBuilds/fl-probe/<its-full-hash>/` as well. This supersedes the publication-blocked state above; final native drag/feel acceptance remains pending. No owner original project was overwritten.

## Final installed-build native test

At the owner's explicit request, reopened FL Studio 2026 using the same copied navigation project and installed **94CF0D** build. No additional code/binary change was needed. Native screenshot observations confirmed:

- A quick primary drag selected **00:02.12–00:03.83**, and focused **F** fitted that exact range. The earlier native input-loss symptom is resolved in this check.
- **C / Center** moved the 1.71-second view to approximately **01:49.31–01:51.02**, with actual captured speech visible during host playback.
- Wheel down widened the view; horizontal-wheel input panned it while retaining the original selection. Fit captured restored **00:00–02:00.43**. Wheel up near the captured speech then narrowed it to **01:02.32–01:56.43**, preserving the pointer's source-time anchor to displayed precision.
- Lock playhead kept the playhead centered while the waveform moved; both tracks showed verified captured input in covered ranges. Catch up advanced at the view edge and continued following the DAW's repeated loop without erasing the captured tail. The second editor retained its own full-history viewport throughout navigation in the first.
- The application background and legible controls rendered in the native host. No disappearing waveform or lost quick selection recurred in these interactions. Playback was stopped afterward and FL was left open on the separate test copy.

A six-second OS sample during two-editor loop playback observed **353.0 MiB** maximum sampled working set and **74.9% of one CPU core** for the entire FL process. A separate file copy was active; neither these values nor screenshot observations isolate plugin cost or establish presented FPS/input-to-photon latency. No vendor runtime was used for comparison. The documented computer-input API cannot hold a modifier or middle mouse button through a drag, so native Alt-drag/middle-drag combinations remain unqualified; their deterministic event regressions pass. This completes ticket 84's scoped native functional check, not ticket 80's broader sound/export acceptance or full desktop/vendor parity.

## Remaining limits

This improves waveform interaction; full desktop editor parity is not complete. Peaks remain 32 ms aggregates. Plugin shortcut customization, desktop gain-point editing, original/processed waveform overlays and the desktop's exact eased Catch up behavior are not implemented here. Source validation, analysis, sound settings, format/platform limits and constant Session latency retain 0.0.5's contract. Native final drag/keyboard/follow/feel, new sound/export/PDC and vendor performance equivalence remain unverified. No sonible installer or vendor runtime was used.
