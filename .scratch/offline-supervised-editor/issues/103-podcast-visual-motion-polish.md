# Podcast visual, motion and performance refinement

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 96, 102
Related: 80, 84, 90, 101

## Scope and acceptance

Implement the owner's FabFilter/sonible quality target in the existing Podcast editor using the shared contract and 102's original visual study. Preserve the complete capture/label/Analyze/audition/recovery workflow; the waveform remains the main work surface.

- [ ] Replace default-looking control density with deliberate type/spacing, restrained sections, readable numeric controls, source/gain layering, a stable playhead and visible selection/focus. Keep actions reachable at 720×560 and 1000×680, 100/150/200% DPI, eight tracks and all readiness/error states.
- [ ] Implement 90's interruption-safe Catch up and visual value smoothing, useful meter/playhead feedback and reduced motion. Immediate audio/automation changes remain independent of visual interpolation. Do not smooth away captured audio transients or human-label boundaries.
- [ ] No ongoing animation when the editor is closed/hidden or when settled; schedule only current visible work, bounded by display width and visible lanes. Preserve keyboard, focus, hover help, selection and scroll semantics.
- [ ] Compare the same pre/post fixtures and build profile: event-to-state, peak/geometry/tessellation work, allocations, per-instance memory/CPU, idle/playback/resize and simultaneous instances. Treat 60 Hz as a presentation target, not a passed result. Report p50/p95/p99 and misses; measure native presentation separately from CPU geometry.
- [ ] Verify unchanged effective-envelope/format/state/automation behavior with relevant existing checks. Keep native listening/PDC/export acceptance separate; no 80 closure by screenshots or synthetic timing.
- [ ] Preserve tested source/artifacts and compact/full/DPI captures; independent Standards/Spec review before integration. Use existing guarded product scripts, one compiler job globally, no new GUI dependency without a demonstrated blocker and a separately measured integration proof.

## Results

2026-10-30 native adapter scheduling correction: review found that `Panel::ui`
still scheduled 16/100 ms polling after the hidden component returned (legacy
workspace: 50 ms). A frame-independent `Panel::show` body retains the actual
trait caller with a one-line forwarder. Its public egui regression reproduced
the hidden polling clock before the fix, then passed after an early invisible
return cancels Session transients and the legacy drag anchor. Committed legacy
selection/frozen history and visible polling are preserved. Actual Podcast now
passes all 80 tests (1.36 seconds), strict library/test Clippy, formatting and
diff checks. [Exact receipts and preserved executable](../../../docs/research/podcast103-seek-2026-10-30.md#native-panel-hidden-scheduling).
This qualifies the adapter's scheduling logic, not native window visibility or
presentation. Bounded Analysis gain, final comparisons and native acceptance
remain; 103 stays claimed. No memory gate was used.

2026-10-30 hidden-workspace continuation: the actual Podcast manifest
`plugins/podcast/Cargo.toml` now passes all 79 tests, strict library/test Clippy,
formatting and diff checks. Hidden Catch up RED reproduced an obsolete viewport
jump; hidden gesture RED reproduced committing a stale primary selection.
The invisible component now reuses the existing close cancellation path and
returns before layout/query/animation work. Viewport, committed selection and
follow are retained; selection/Alt zoom/Middle pan are covered. No source-capture
locking, gain display, processing or native window behavior changed.
RED receipts `215052-7f23c247` and `215735-e555976e`; final compile
`215747-d0eb0d85`, full suite `215915-cfa01c35` (79 passed, 1.35 seconds),
strict Clippy `215918-fb1a2b08`. All finalized through the timeout-only wrapper,
without memory gates. Actual native visibility remains unqualified.

Manifest qualification correction: the first resumed Clippy command used the
legacy `fl-probe` manifest. It did not qualify the Podcast product. A subsequent
zero-match test run was likewise not a test pass. The final results above use
the actual Podcast target and its exact compiler-artifact executable, explicitly
superseding those attempts. Failed harness type/array initialization checks were
corrected before semantic evidence; no lint suppression or dependency change.

2026-10-30 owner resumed. Indicator strict library/test Clippy now passes in
`20261030-213701-9008c2abe68a4142ac644bf1d5acd7ac`: root exit 0, completed,
recorded cleanup verified and no remaining job PIDs. The initial resumed check
found six approximate diagonal constants in the test oracle; they now use
`std::f32::consts::FRAC_1_SQRT_2`, preserving the independent mathematical
expectation. No production indicator behavior changed; the existing 78-test
runtime/capture evidence remains applicable. Formatting and diff checks pass.
The wrapper has timeout-only policy, with no memory preflight, reserve or cap.
The first failed lint's owned VCTIP wait was cancelled and the compiler lock
released before the successful warm check. Hidden Catch up and bounded gain
consumer work remain; native and broader 103 acceptance are not closed.

2026-10-30 OWNER PAUSE: indicator source `18fcba8` is preserved after 78 passing
library tests, actual CPU captures and formatting. Three110ms decorative cues
preserve immediate settings/Analyze-required staleness; retarget, reduced-motion,
close and hidden scheduling are tested. Clippy was cancelled during dependency
checking; no passing Clippy/native/presentation claim. Scoped process exit and
global lock release were verified. [Exact partial-checkpoint source, artifacts,
receipts and limits](../../../docs/research/podcast103-seek-2026-10-30.md#owner-pause-after-indicator-checkpoint).
103 remains claimed and isolated. No shared110/gain consumer implementation or
permanent product integration occurred. Retain the worktree and await owner resume.

2026-10-30 containment pause: compiler commands used rustup-proxy Cargo/Rustc;
recorded named-job process counts/peaks do not establish complete compiler-chain
membership. Ticket109's containment proof is now required before further
compilation. The direct actual-executable76-test result for `4083b0f` and its
preserved artifact/receipt remain unchanged. An indicator RED is checkpointed
**uncompiled and unrun**, with no indicator behavior implementation.
[Continuation and accounting limits](../../../docs/research/podcast103-seek-2026-10-30.md#containment-pause-and-continuation-draft).
103 remains claimed; retain this incomplete scoped worktree without merging the
untested draft into the permanent Podcast branch. No Clippy/native/render checks.

2026-10-30 unfinished-gesture slice `4083b0f`: actual component pointer input
RED/GREEN covers selection-drag, Alt-range zoom and Middle-pan across close/
reopen. One cancellation path restores the committed selection and clears
transient gestures for close and existing Escape/focus loss. All76 library tests
and formatting pass with finalized cleanup-verified default guards.
[Source, receipt and artifact pointers](../../../docs/research/podcast103-seek-2026-10-30.md#unfinished-gesture-close-slice).
Clippy, indicators, Analysis gain, final comparisons and native acceptance remain;
103 stays claimed and isolated.

2026-10-30 editor-close slice `fa1f3f4`: an actual component-frame semantic RED
then GREEN verifies the lifecycle boundary cancels the obsolete Catch up tween
without changing viewport/selection/follow. Panel forwards its close callback.
All75 library tests and formatting pass; selective pinned-LLD test compiles and
direct suite guards finalize with root0, no remaining PIDs and cleanup verified.
[Exact commands, receipts, source and preserved artifact](../../../docs/research/podcast103-seek-2026-10-30.md#editor-close-lifecycle-slice).
Clippy, gesture-close cleanup, actual native hidden/reopen/context scheduling,
remaining visuals/performance and native acceptance stay open. No103 closure or
permanent product integration is claimed.

2026-10-30 restored-checkout continuation: source `595d0aa` fixes the saved
backward host-seek RED; the targeted test passes (one passed, 73 filtered out).
The default fresh-build guard then waited for actual job member VCTIP.EXE PID4112
after Cargo exited0. Scoped cancellation and mutex release were verified; no
finalized successful guard receipt or full-suite/native result is claimed.
[Exact source, artifact hash and job evidence](../../../docs/research/podcast103-seek-2026-10-30.md#restored-checkout-continuation).
Ticket109 owns the shared guard prerequisite. This slice remains isolated and
103 remains claimed; permanent Podcast integration and other acceptance stay open.
The preserved source595d0aa executable then passed all74 library tests directly,
without compiler/linker work, under the unchanged default guard. Finalized receipt
`20261030-192408-844f49a11ce642b69b72908ad31d801c` records exit0, cleanup verified
and no remaining processes. This excludes the uncompiled lifecycle RED draft and
does not resolve the MSVC build-helper stall or claim native acceptance.

2026-10-30 scoped continuation: `codex/podcast103-seek` starts at exact `2b0595f`.
The public egui/Session seek-interruption regression produced semantic RED:
after a backward seek into the visible range, Catch up continued to its old
146000–226000 target instead of retaining the interrupted 73000–153000 range.
Production code is unchanged. The outer resource guard did not complete after
the test exited; its exact owning PowerShell process was cancelled, and global
mutex release was verified. No GREEN or retry ran. See
[the incomplete checkpoint](../../../docs/research/podcast103-seek-2026-10-30.md).
Keep this failing-test branch isolated; ticket remains claimed.

Post-app-recovery source `da703a556360f2862abc9f7ac27c0ea8d0bdbf7d` exposes real
shared sound numeric controls inline and reserves compact footer space. Semantic
RED was observed; all 73 library tests now pass, including value/unit clipping,
actual stale-analysis readiness and compact/full/DPI layout. Formatting, targeted
strict Clippy and six actual-widget CPU captures pass. Receipts, guarded-memory
measurements and a preserved test-only executable are recorded in the checkpoint
report. The compact 100% PNG was visually inspected. Status remains claimed;
broader motion/gain/performance/native acceptance and independent reviews remain
open. No permanent-lane integration or native host qualification is claimed.


Current machine-green motion source `671c4b5c556bef51a6b1ccb53055f09e2943f274`: all 72 tests, formatting and targeted strict normal Clippy pass in receipt `20261027-105758-d786de06357743d2bef92408eb328502`. The finite 250 ms slide, local Reduced motion snap/stop behavior and existing navigation timing case are verified. The owner-authorized warm profile is fixed at 8 GiB start / 3 GiB running reserve / 2.5 GiB owned cap / 600 seconds, with immediate native physical >=8 / commit >=6 GiB preflight; script defaults are unchanged. Peak owned commit 1,801,764,864 bytes, minimum physical 11,908,128,768 bytes, minimum commit headroom 8,564,899,840 bytes, exit 0 and cleanup verified. A prior linker failure was repaired by reversible quarantine of the exact failed Podcast test incremental leaf, preserving other caches/dependencies and tested binaries; exact paths and receipts are in the report. The green test executable/source hash is preserved outside the worktree. This is not a normal plugin build or 103 completion. The heavy slot was released to 93; no further compiler/renderer/packaging work follows without coordinator grant.

Remaining: hidden/seek/interruption completion, 110 ms indicators, actual visual refinement preserving the full 96 workflow, final comparable performance/captures, relevant format checks and independent review. Status remains claimed and all broader acceptance unchecked; permanent base is still `13e6dfd`. Native 80 and blocked 81–83 remain unchanged. Historical checkpoints below are retained rather than rewritten.

While 93 owns the compiler, one next compact public UI RED was prepared (unrun): actual shared -18 dB / 22 ms / 120 ms controls plus Analyze and stale readiness must be visible without opening details. Inline-value behavior has not been implemented. Authorized pure styling refinements to header/audition/separators/Analyze are drafted but unverified; no compiler/indexer/renderer ran. The green motion source remains `671c4b5`, distinct from this pending visual/test draft. A bounded source-valid published Analysis gain view was proposed before any data/curve implementation.

After the coordinator returned the slot and approved the bounded Analysis gain seam, the single inline-values RED attempt on clean `a97e96b` was stopped during compilation: receipt `20261027-111158-855e01bb4f334fbe8c6cfc62f06f73fd`, `resource-blocked` / `low-commit-running`, cleanup verified. Same-process preflight passed; minimum commit headroom reached 3,207,901,184 bytes (2.99 GiB <3 GiB running reserve), minimum physical 9,217,495,040 bytes, peak owned commit 1,716,113,408 bytes. Fixed 8/3/2.5 GiB / 600-second warm profile stayed unchanged. No test executed, no semantic RED is claimed, no behavior implementation or retry followed. No compiler/helper remains. Visual WIP is unchanged; last qualified motion source remains `671c4b5`. Status remains claimed/unmerged; all broader acceptance stays unchecked.

Claimed 2026-10-27 from permanent Podcast tip 13e6dfd after 96 integration and 102 completion. Scope uses the existing egui/Session seams and shared visual contract; no framework migration. Gate 93 owns the sole heavy slot: only document/source inspection and one first RED test may be prepared until the coordinator grants execution. No production implementation or renderer/compiler execution has started. Native ticket 80 and dependencies 81–83 remain unchanged.

Coordinator subsequently granted the sole heavy slot. First public-egui/Session Catch up test produced semantic RED (instant jump) then GREEN for a finite 250 ms eased slide with selection/host transport unchanged. Comparable eight-track generated native-PCM baseline records cold and 40 warm query/draw/tessellation samples, p50/p95/p99, mesh counts/capacity at compact/full sizes. No visual refinement, reduced-motion implementation, gain/processing change or native/FPS qualification has landed. Context7 quota was exhausted; pinned egui 0.37 source supplied API documentation.

The next reduced-motion/stop regression could not execute: unchanged guard `20261027-102017-498887ecffda405da9a8ede99fe30316.json` stopped compilation on low system commit headroom (7.74 GiB, below 8 GiB running reserve), with cleanup verified. No semantic RED or GREEN is claimed for that test, and no reduced-motion implementation followed. No retry or threshold relaxation occurred. Keep all acceptance unchecked; finite-motion checkpoint still needs interruption/seek/hidden/reduced-motion completion, formatting, full relevant checks, visual slices and comparisons, final artifacts/captures and independent review. [Checkpoint and raw baseline](../../../docs/research/podcast-ui-polish-2026-10-27.md).

2026-10-27 owner-authorized resumption used the new ticket-specific outer compiler profile documented by canonical `0408d03`: 10 GiB physical start / 5 GiB running reserve / 4 GiB owned cap / 1,200 seconds, with one >=9 GiB commit-headroom preflight. Script defaults stayed unchanged. The single preflight passed at 12.84 GiB physical / 9.68 GiB commit headroom. The immediate guarded launch refused before process creation at 9.60 GiB physical and 5.57 GiB commit headroom: receipt `20261027-104400-65063aac05df4a0d8e23c0a16c8469fe.json`, `resource-blocked` / `low-memory-before-start`, process null, peak owned commit zero, cleanup verified. No compiler or test ran, no source implementation changed, and no retry/lower threshold followed. The reduced-motion/stop test remains unexecuted; status remains claimed and all acceptance unchecked.

A coordinator-reviewed new resource state authorized one fresh attempt under the same fixed profile. Immediate same-process preflight passed; `20261027-104941-ef7d976aada64eafb74aab5e726ae6dd` executed the prepared test and confirmed semantic RED: missing Reduced motion control. A narrow local checkbox/snap implementation followed. Its GREEN compile was then stopped by `20261027-105059-5c1c52f2990e4219a2f278b360beed45` for `low-commit-running` before any test executed; cleanup verified and no compiler/helper remains. The reduced-motion implementation is preserved but unverified; no visual refinement followed and no retry occurred. Exact successful-RED and stopped-GREEN resource measurements are in the report. Status remains claimed; no integration or native qualification.
