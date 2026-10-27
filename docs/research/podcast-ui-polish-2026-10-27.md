# Podcast visual refinement checkpoint

Current green motion source: `671c4b5c556bef51a6b1ccb53055f09e2943f274`. All 72 unit/event tests, targeted strict normal Clippy (`--lib --tests`) and formatting pass. This verifies the finite 250 ms Catch up slide, local reduced-motion snap/stop behavior and the adjusted existing navigation case. It is **not** completion of 103: visual refinement, hidden/seek/interruption completion, 110 ms indicators, comparable final performance/captures and independent review remain open. Nothing has been merged into the permanent Podcast lane.

Ticket 103 is claimed on `codex/podcast-ui-polish-103`, from exact permanent Podcast tip `13e6dfd3781a3371e429f9760a9f7f30fc77df90`. Only the required ticket, premium contract amendment and original 102 visual-study sources/frames were imported from the primary tracker; no desktop branch was merged. Existing egui/Session seams remain the test boundaries. Context7 resolution returned its monthly quota error; pinned egui 0.37 source documents `RawInput.time`, viewport visibility and repaint APIs.

Partial source checkpoint `d2b30c91fa5a7f078ca75cdbf6999defc69ed513` preserves the first Catch up RED/GREEN source and comparable fixture/baseline instrumentation. It is not integration-ready: no full suite or formatting/Clippy pass is claimed. A separate explicitly unverified test commit preserves the reduced-motion/stop case and the adjusted existing navigation timing expectation; neither executed after the guard stop. The exact recoverable test patch and original full source are outside the worktree at `C:/Users/User/AppData/Local/CleanSlate/Podcast/Workflow103Checkpoints/f0da41de356240d9b7f502fa0af5adf4/` (`unverified-motion-tests.patch`, `session_editor-with-unverified-tests.rs`). No source from this checkpoint has been integrated into the permanent Podcast lane.

## Initial scope (historical)

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

## Verified warm motion checkpoint

The owner authorized a warm targeted compiler/test profile documented by canonical `e164c48`: 8 GiB physical start / 3 GiB physical and commit running reserve / 2.5 GiB owned cap / 600 seconds. Immediate same-process native preflight requires physical >=8 GiB and commit headroom >=6 GiB. This is a scoped warm check, not a changed default or packaging authorization. Dependencies, Cargo -j1, pinned locked/offline shared development profile and allocation guard remain unchanged.

1. Immediate preflight measured physical 13,419,827,200 bytes / commit headroom 10,286,313,472 bytes. Receipt/log `20261027-105404-d7316556d85e4a3db76dfb003d513b50` failed linking the existing test target with 45 unresolved LLVM/drop-glue symbols (`LNK1120`); no test ran. This was a command failure, not a resource stop: peak owned commit 1,802,608,640 bytes, minimum physical 12,417,937,408 bytes, minimum commit 8,517,373,952 bytes, cleanup verified.
2. Failed `06jxpkslw4gbg8ox7rvvirbav.o` matched only test incremental leaf `C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/incremental/cleanslate_podcast-1df7lwupoayy7` (two cached sessions); fingerprint `cleanslate-podcast-e73117c511252bed` confirmed the default-feature test target. That exact resolved leaf was moved with `Move-Item -LiteralPath` into fresh `C:/Users/User/AppData/Local/CleanSlate/Podcast/Workflow103Checkpoints/incremental-quarantine-b51c17f321354b5580d2512350f01c99/`. Parent roots and preserved subtree are non-reparse. No deletion/broad clean, dependency rebuild, profile change or tested-binary replacement occurred. Regenerating the leaf resolved the linker failure; interrupted incremental data is the supported inference, not a proven compiler root cause.
3. Repair preflight: physical 14,198,210,560 bytes / commit 11,443,707,904 bytes. Receipt/log `20261027-105543-3130cd83cef6487492684d8306ea51fd`: both Catch up tests pass, exit 0, cleanup verified. Peak owned commit 1,803,067,392 bytes, sampled aggregate working set 1,543,811,072 bytes, minimum physical 9,059,057,664 bytes, minimum commit 4,319,195,136 bytes (above the 3 GiB reserve). Elapsed 120.497 seconds includes waiting for an owned helper, not a compile benchmark.
4. Warm suite preflight: physical 13,479,010,304 bytes / commit 10,379,407,360 bytes. Receipt/log `20261027-105758-d786de06357743d2bef92408eb328502`: formatting, all 72 tests (3.01 seconds test runtime), targeted strict Clippy and final format check pass; exit 0, cleanup verified, no competing compiler. Peak owned commit 1,801,764,864 bytes, sampled aggregate working set 1,620,156,416 bytes, minimum physical 11,908,128,768 bytes, minimum commit 8,564,899,840 bytes. Elapsed 151.765 seconds includes helper cleanup. The receipt records base `b2eea64`; formatting performed inside the job was committed unchanged as `671c4b5`. Original receipts were retained, not rewritten.

The coordinator proved named-job ownership and stopped only VCTIP after each compiler exit. No helper/compiler remains. The sole heavy slot was released to Gate 93 after this checkpoint; no renderer, packaging or further compiler work was started.

The exact green test executable is preserved at `C:/Users/User/AppData/Local/CleanSlate/VerifiedBuilds/plugin-products/Podcast/671c4b5-motion-tests/cleanslate_podcast-motion-tests.exe`: SHA256 `f95f04cedfccacc67bc91a0abccdc9cd75f98cd1fc0b3aa7894aba081bdea77a`, 40,079,872 bytes. The copied/source hashes match; `source-receipt.json` records source/check identity and `TEST-ONLY.txt` prohibits installation/distribution. This is a test executable, not a new normal plugin candidate. Prior 96 normal/acceptance artifacts remain untouched.

The comparable raw baseline remains `C:/Users/User/AppData/Local/CleanSlate/BuildReceipts/20261027-101820-c79286ea001f451e82c4176fabc24ce2.log`. No post-visual comparison exists yet. Ticket 103 remains claimed/unmerged and its broader acceptance stays unchecked; 80 and 81–83 retain their native limits.

## Light preparation while Gate 93 owns the compiler

One next public egui/Session test is prepared but unexecuted: an eight-track compact workspace with actual shared settings of -18 dB, 22 ms Fade in and 120 ms Fade out must expose those numeric controls, shared scope, Analyze and the actual stale-analysis reason without opening a menu. These values deliberately differ from legacy host parameters and the illustrative study. Inline controls have **not** been implemented before this RED runs.

The coordinator separately authorized pure styling work without implementation-mirroring tests. The unverified draft keeps the existing palette/actions and adds a regular 22 px product title, aligned equal-width audition buttons, quiet header/lane separators and stronger primary Analyze styling. Existing geometry tests and actual captures must validate it when the heavy slot returns. No compiler, indexer or renderer ran during this preparation. The verified motion source stays `671c4b5`; the new visual draft is not covered by its 72-test receipt.

A proposed gain layer was sent for review before data changes: bounded optional source-time **Analysis gain** points from the current-revision published member mapping and maintained evaluator, with exact human-label boundary points and missing/unverified evidence omitted. It must not claim instantaneous applied audition gain or bridge unknown source spans. No view field, gain data change or painted processing curve has been implemented yet.

## Visual RED launch: resource stop

The coordinator accepted the inline-value test and bounded gain-view scope, then returned the sole compiler slot. Gain boundaries must fit a fixed point budget; dense boundary sets must suppress detail with truthful zoom guidance instead of shifting boundaries or allocating unbounded geometry. This approval did not authorize implementation before the relevant behavioral RED.

On clean visual WIP `a97e96b26b15d178f94af5b9091ab94c80478f8f`, one immediate same-process warm preflight measured physical 12,339,937,280 bytes and commit headroom 8,172,863,488 bytes, meeting the fixed >=8 / >=6 GiB requirement. Guard receipt/log `20261027-111158-855e01bb4f334fbe8c6cfc62f06f73fd` then stopped during compilation for `low-commit-running` at minimum commit headroom 3,207,901,184 bytes (2.99 GiB, below the fixed 3 GiB running reserve). Minimum physical was 9,217,495,040 bytes; peak owned commit 1,716,113,408 bytes; sampled aggregate working set 516,300,800 bytes; elapsed 7.517 seconds. Cleanup verified and no compiler/helper remains. Policy stayed 8/3/2.5 GiB / 600 seconds, single Cargo job/pinned locked shared profile.

No test executed, so the prepared inline-values test remains **unexecuted**, not a semantic RED. No inline controls, gain view or other behavior was implemented. No retry, budget change, uncached dependency download, renderer or packaging followed. Source is unchanged from `a97e96b`; this checkpoint only records the failure. The last qualified motion source remains `671c4b5`; 103 stays claimed/unmerged and broader/native acceptance remains open.
