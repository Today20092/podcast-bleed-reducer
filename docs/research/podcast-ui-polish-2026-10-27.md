# Podcast visual refinement checkpoint

Ticket 103 is claimed on `codex/podcast-ui-polish-103`, from exact permanent Podcast tip `13e6dfd3781a3371e429f9760a9f7f30fc77df90`. Only the required ticket, premium contract amendment and original 102 visual-study sources/frames were imported from the primary tracker; no desktop branch was merged. Existing egui/Session seams remain the test boundaries. Context7 resolution returned its monthly quota error; pinned egui 0.37 source documents `RawInput.time`, viewport visibility and repaint APIs.

Partial source checkpoint `d2b30c91fa5a7f078ca75cdbf6999defc69ed513` preserves the first Catch up RED/GREEN source and comparable fixture/baseline instrumentation. It is not integration-ready: no full suite or formatting/Clippy pass is claimed. A separate explicitly unverified test commit preserves the reduced-motion/stop case and the adjusted existing navigation timing expectation; neither executed after the guard stop. The exact recoverable test patch and original full source are outside the worktree at `C:/Users/User/AppData/Local/CleanSlate/Podcast/Workflow103Checkpoints/f0da41de356240d9b7f502fa0af5adf4/` (`unverified-motion-tests.patch`, `session_editor-with-unverified-tests.rs`). No source from this checkpoint has been integrated into the permanent Podcast lane.

## Current scope

The first semantic RED proved that Catch up jumped immediately to its 5% inset target. A minimal finite 250 ms smoothstep slide passes the same deterministic 0/125/250 ms test while preserving the range selection and host transport. Manual viewport dispatch and playback stop clear that transient target. This is a partial motion checkpoint: seek/hidden/reduced-motion behavior, interruption coverage, visual refinement and full checks remain unfinished. No new GUI dependency or processing/sound parameter change occurred. Shared sound retains actual dB and Fade in/Fade out milliseconds; study percentage values and per-track overrides are not product behavior.

The next reduced-motion/stop test was prepared before production implementation. Its compilation was stopped by the resource guard; it has not executed and supplies no behavioral RED/GREEN. Full regression/formatting/Clippy checks have not run for this checkpoint. Existing navigation coverage was updated to allow the specified 250 ms slide rather than demand an immediate jump; that adjusted case is also unrun.

## Comparable baseline

The shared test fixture generates eight distinct 8 kHz PCM tracks, eight seconds each, with initial room, differing speech-like bursts, low background and sharp transitions. It supplies actual captured peaks, Room/solo/overlap labels, selection and playhead through the existing Session view and egui renderer. It is not speech-quality or native-host evidence. The same helper feeds future compact/full captures and performance measurements. Rendering uses 1,000 queried columns, readable lane heights and offscreen culling; the fixture/profile must remain unchanged for the post-refinement comparison.

One cold frame plus 40 warm frames were recorded at each logical size. Values below are microseconds, in p50 / p95 / p99 order over warm samples using nearest-rank indices. The retained raw log contains every sample, cold values, shape/vertex/index counts and retained mesh-capacity bytes.

| Logical size | Peak query | Geometry build | Tessellation | First warm vertices |
| --- | --- | --- | --- | --- |
| 720×560 | 208.197 / 236.133 / 288.584 | 243.187 / 398.864 / 515.778 | 188.292 / 319.804 / 326.508 | 12,664 |
| 1000×680 | 236.482 / 314.635 / 342.991 | 567.740 / 654.343 / 666.146 | 491.543 / 615.930 / 770.210 | 20,808 |

Timing spans include observer overhead and are CPU work only, without a native compositor, audio playback or presented-frame timing. Mesh capacities are scoped geometry storage, not allocation-call counts. nice-plug owns the debug global allocator; no replacement allocator or new dependency was introduced. Native per-instance CPU/memory, multiple editor instances, event latency, allocation counts, presentation misses and final before/after comparison remain unmeasured.

## Guard receipts

All receipt/log pairs are under `C:/Users/User/AppData/Local/CleanSlate/BuildReceipts/`. Source receipts record base `13e6dfd` plus the explicitly dirty 103 diff, subsequently preserved in the checkpoint commit. Original receipts are retained unchanged.

- `20261027-101346-390bc90e13ab4d8e8de0e28bec893b2d`: first Catch up semantic RED. Initial viewport was 146000–226000 instead of 0–80000.
- `20261027-101519-4d177d6339714ac59eb3b02cf82b23aa`: same Catch up test GREEN; 1 passed.
- `20261027-101820-c79286ea001f451e82c4176fabc24ce2`: varied-fixture CPU baseline GREEN; 1 passed, raw timing JSON in `.log`.
- `20261027-102017-498887ecffda405da9a8ede99fe30316`: next test compile stopped, `resource-blocked` / `low-commit-running`; no test executed. Peak owned commit 1,770,033,152 bytes, minimum available physical 11,223,363,584 bytes, minimum system commit headroom 8,315,363,328 bytes (7.74 GiB). Cleanup verified; no compiler/helper remains.

Every heavy job used the unchanged 14 GiB start / 8 GiB running reserve / 6 GiB owned commit cap, shared pinned development profile, locked/offline Cargo and one compiler job. No retry, policy relaxation, cache wipe or unrelated process termination followed the stop. Earlier helpers were stopped only by the coordinator after proving exact named-job membership and compiler exit.

Native 80 remains open and 81–83 remain blocked. No installation, owner audio/project write, release, DirectML/clean-room runtime qualification or vendor-equivalent performance claim occurred.

## Owner-authorized resource-profile resumption

The owner explicitly authorized continued implementation with a reduced commit reserve; canonical `docs/resource-guard.md` at `0408d03` records the fixed ticket-specific outer compiler profile: `-MinAvailableGiB 10 -ReserveGiB 5 -MaxCommitGiB 4 -TimeoutSeconds 1200`, retaining global containment/mutex, Cargo -j1 and the pinned locked/offline shared profile. Its separate one-time commit-headroom preflight requires at least 9 GiB. Guard script defaults remain 14/8/6 GiB.

On clean `bb9ceb3`, the single explicit preflight measured 12.84 GiB available physical and 9.68 GiB commit headroom, meeting the preflight requirement. The immediate guarded launch then observed 10,306,867,200 bytes available physical (9.60 GiB, below its 10 GiB start threshold) and 5,984,399,360 bytes commit headroom (5.57 GiB). The new receipt/log pair `20261027-104400-65063aac05df4a0d8e23c0a16c8469fe` records `resource-blocked` / `low-memory-before-start`, no process ID, zero peak owned commit, elapsed 0.115 seconds and verified cleanup. No compiler or test executed, so the pending reduced-motion/stop test still supplies no semantic RED/GREEN. Source implementation is unchanged from `bb9ceb3`; this resumption adds only the resource checkpoint documentation. No retry, policy adjustment, paging/service change or reboot followed this failure.

## Fresh measured state: reduced-motion RED and blocked GREEN

The coordinator reviewed a new resource state and authorized one fresh attempt under the same fixed 10/5/4 GiB, 1,200-second profile. Both preflight and immediate outer-guard invocation ran in one PowerShell process, with no intervening tool delay. Preflight required physical >=10 GiB and commit headroom >=9 GiB. No script default or reserve was changed again.

- RED preflight: available physical 14,377,865,216 bytes; commit headroom 11,612,958,720 bytes. Receipt/log `20261027-104941-ef7d976aada64eafb74aab5e726ae6dd` compiled and executed the pending test on clean `6d617d1`: semantic failure `Missing control Reduced motion`. Peak owned commit 1,687,654,400 bytes; minimum physical 12,955,394,048 bytes; minimum commit headroom 9,934,983,168 bytes. Cleanup verified; the coordinator stopped only the proven owned VCTIP helper after compiler exit.
- The narrow production response adds a local Reduced motion preference in the existing details menu. It snaps Catch up to its bounded target, including a transition already in progress. It does not change source/labels, audio, shared sound settings or host parameters. This implementation is **unverified**.
- GREEN preflight: available physical 13,722,701,824 bytes; commit headroom 10,314,125,312 bytes. Receipt/log `20261027-105059-5c1c52f2990e4219a2f278b360beed45` stopped during compilation with `resource-blocked` / `low-commit-running`; the test did not execute. Peak owned commit 1,802,674,176 bytes; sampled aggregate working set 599,691,264 bytes; minimum available physical 9,585,836,032 bytes; minimum commit headroom 4,933,238,784 bytes (4.59 GiB, below the 5 GiB running reserve); elapsed 20.946 seconds. Cleanup verified; no compiler/helper remains. No retry or further implementation followed.

Only the first 250 ms Catch up slice and earlier varied-fixture baseline have passing behavioral evidence. The new local preference, combined stop/reduced-motion test and adjusted existing navigation timing case still require GREEN, formatting and relevant checks. Visual refinement, hidden/seek/interruption completion, indicator easing, final performance comparison/captures and independent review remain unfinished. Ticket 103 stays claimed and unmerged; native 80/81–83 limits remain unchanged.
