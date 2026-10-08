# Retain complete Nemotron evidence for long recordings

Status: done
Assignee: diarization_finish
Blocked by: 03, 05
Spec: ../spec.md

## Scope

Wrap the pinned standalone C diarization ABI narrowly in Rust with checked statuses, owned lifecycle and copied results. Use CPU mode and v3-offline preset through one continuous stream per recording, with bounded pushes and cancellation on a worker. Do not use long-input full-attention inference or restart chunks while assuming speaker IDs persist.

Copy newly available probability frames into project-owned cache using global frame indices before native compaction. Include the final tail and keep all required history for two-hour/eight-speaker analysis. Record runtime/model/preset/cadence/mix provenance. Compare deterministic nonclipping analysis downmix recipes on development fixtures and freeze one; preserve separate per-track measurements.

## Reach

New native diarization module, src/prepared_analysis.rs fixed-evidence cache, source preparation workers and fixtures. C buffers/errors must be copied while valid; never expose opaque handles to the GUI or audio callback.

## Acceptance

- Native load/infer/destroy succeeds and failures/cancellation/stale completion cannot install partial current results.
- A recording exceeding native compaction duration retains early, boundary and final frames with no duplicates or gaps; a two-hour run measures memory/cache growth.
- Eight-channel capacity and too-many-speaker policy are checked; channels remain hypotheses until human mapping.
- Long-gap speaker recurrence, processing boundaries, quiet speech and overlap are tested against references.
- Chosen downmix and repeat-run behavior are recorded; native API support is not substituted for measured accuracy or throughput.

## Implementation checkpoint, 2026-10-08

Continuous `v3-offline` owned-helper processing now reads prepared 16 kHz copies in one-second blocks, copies new global probability rows before native compaction, retains the finish tail, and records source/runtime/model/preset/mix provenance. Complete history is disk-backed with bounded reads, length/hash validation and atomic cache publication. Parent cancellation/native abort cannot publish a partial result. The project consumer must still check its current source/job generation before installing fixed evidence into a preview.

Three regular public-boundary tests passed after fresh compilation in the native worktree. Actual CPU native inference on a 100 ms fixture retained all 11 frames/eight columns with verified history and provenance (3.57 s). The independent cache lifecycle probe passed cold acquisition, reuse with an absent helper proving no second inference, and cancellation retaining accepted bytes (3.67 s). These are deployment/infrastructure checks, not speech accuracy or long-session qualification.

At this checkpoint, native compaction, two-hour memory/cache measurement, repeat/downmix comparison and controlled speech/overlap/recurrence checks were pending; the qualification below records their subsequent results. Scripted Windows SAPI development fixtures are local at `C:/Users/User/.codex/backups/podcast-automix-rust-gui/ticket06-development`; they do not replace independent real recording labels or owner listening approval.

## Qualification results, 2026-10-08

Prepared audio payload SHA-256 now comes from the verified preparation manifest, participates in the evidence key, and is checked before cache reuse, while reading inference input, and before publication. A same-length payload mutation is rejected. Missing/malformed/corrupt cached evidence is rebuilt and the invalid previous directory retained in quarantine; cancellation and ordinary I/O failures preserve the destination. A Windows exclusive payload lock regression checks that an ordinary I/O failure does not quarantine accepted evidence. Writer exclusion uses an OS-owned `File::try_lock`; the owned subprocess crash regression proves a live writer excludes another writer and process termination releases the lock. Native helpers keep bounded diagnostic tails, preserving the final runtime provenance after an 80 KiB verbose prefix.

Scripted development audio uses Windows SAPI Microsoft David Desktop and Microsoft Zira Desktop. Known windows include separate speakers, simultaneous speech, 5% amplitude quiet speech, and the first voice returning after a long gap. These are construction-known development references, including pauses inside utterances, not independent real speech accuracy labels.

| Actual CPU probe | Frames / history bytes | Compactions / maximum retained frames | Wall seconds | Sampled peak private / working bytes |
| --- | --- | --- | --- | --- |
| 120 s, two inputs, global mean | 12,001 / 384,032 | 0 | 7.005 | not sampled |
| 120 s, two inputs, per-track peak mean | 12,001 / 384,032 | 0 | 7.199 | 1,599,614,976 / 150,065,152 |
| 1,500 s, two inputs, global mean | 150,001 / 4,800,032 | 1 / 118,272 | 63.606 | 1,603,829,760 / 152,522,752 |
| 7,200 s, eight inputs, global mean | 720,001 / 23,040,032 | 10 / 119,193 | 469.784 | 1,606,377,472 / 155,566,080 |

The eight-input run has two scripted voices and six silent contributors. It proves eight-input capacity and complete two-hour history with actual compaction, not eight distinct speakers' accuracy. Global frame copying checks reject gaps/regressions, skip already copied overlap, and include the terminal tail. The resulting two-hour history hash is `c60d7d1f1a4006a502815026389d2ec8357f3bd3a9b6a10eebc172c166c9ae42`. Early speaker-1 hypothesis mean is 0.7985; its return at 7,186–7,198 s retains hypothesis 0 with mean 0.7994. Quiet/overlap windows retain the scripted hypotheses. These descriptive means include utterance pauses and are not word-recall scores.

The ignored public native inference test `continuous_stream_repeat_and_push_boundaries_preserve_probabilities` passed on the 120-second fixture (14.18 s): two fresh streams with 16,000-sample pushes and another with 512-sample pushes produced exactly equal 12,001-by-eight probability arrays. Changing push boundaries does not restart streams or speaker hypotheses.

Freeze `mean-global-peak-floor-one-v1`: sum dedicated inputs divided by input count, with a global peak ceiling and floor-one denominator. The alternate per-track peak recipe made no material improvement on the two-voice development windows. Both are analysis-only; source tracks retain their own timebase and measurements. Runtime/model/preset/mix/cadence/source identities and payload hashes remain in each result. Required runtime revision is `8642eaa5cc51efbc17ad0f3e433944ba858a873f`, runtime recipe SHA `5f3102215258cc6cf56c743da1f3f2d0cbbf111cd27b646a03d1693caddcfa8e`, model SHA `08456d9e22cd9a323c0364d98375f3746d6e68507ebb705cd46438c534c7a3a1`, CPU `v3-offline`, reported cadence 0.009999999776482582 s/frame.

Raw requests, probabilities, stdout/stderr and measured reports remain under the external development directory, in `fixture-1500-2` and `fixture-7200-8`. Reproducible generator `build-fixture.ps1` SHA is `375659a5cf61a14841e33db145c04a39ee3b6fae2293344430e7b411ad5ffd1d`; measurement script `run-probe.ps1` SHA is `422a3314ebed0025957ff59e054277f56c0c7e16a159be4ca869bb4c55ae5945`. Request identities bind actual prepared bytes. This sparse fixture throughput does not promise real full-show analysis latency. Independent labels, eight-speaker quality, bleed improvement and owner listening remain ticket 14/15 release gates.

Final integration validation: `cargo test --all-targets --locked -j 1 --target-dir target/diarization --config 'profile.test.package.rust-app.debug=0'` passed 131 tests, zero failures, with 14 intentionally ignored native/owner/benchmark fixtures. Required native fixtures were explicitly exercised as recorded above. Strict all-targets Clippy passed after replacing a constant-size chunk iterator in the new qualification test with `as_chunks::<4>()`; formatting and whitespace checks passed. No dependency was added. The branch incorporates integration `6dc5297` before completion. Fixed-evidence consumers in tickets 07/12 must validate current project source/job generations before installing returned results; this module never installs GUI previews itself.
