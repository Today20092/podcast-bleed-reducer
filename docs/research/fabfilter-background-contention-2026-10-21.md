# Background analysis contention observations

Date: 2026-10-21. Ticket08 adds an opt-in measurement to the existing benchmark command. No worker admission, priority, parallel DSP, SIMD or user worker-count setting changed. These component observations do not establish native application or device deadlines.

## Reproduction and identity

Use the pinned toolchain and shared settings, with no competing compilation or workload during measurement:

```powershell
$settings = & ./scripts/local-build-settings.ps1
cargo build @($settings.CargoArguments) --bin preview_benchmark
$exe = Join-Path $settings.TargetDir 'debug/preview_benchmark.exe'
& $exe --contention <session.cleanslate> <new-report.json> <new-cache-directory>
```

Each run requires a new report and cache directory. The command runs standalone full analysis with a fresh worker/private application cache, then a two-second foreground baseline, then full analysis with another fresh worker/cache concurrent with foreground work. OS/shared playback caches are not flushed, standalone always runs first, and the accepted baseline result stays in memory during concurrent work. This is an ordered pair, not a randomized causal study.

The foreground uses public transient region edits, the shared effective-envelope evaluator, envelope delivery and 960 actual software playback samples at a 48 kHz output grid. It rests 20 ms after each operation rather than catching up after a missed interval. Completion time includes wake lateness; work elapsed excludes that lateness. Up to 4,096 observations per metric are retained; percentiles use empirical nearest rank. This consumer can slow its playback progression when delayed; it does **not** emulate a device deadline. Drafts are never saved, and no recording contents, file names, source paths or fingerprints enter the report. Existing report files and cache directories are refused.

The final binary was built on integration `6ea58183918ad077cf2f34afa982c354bee1e9b9` plus this ticket's changes. SHA-256: `33c2bfe4de1e98e609f1c974ce3ae2fa3c63b1ac26a24d99a225e0f945514d56`. The measured `src/editor_benchmark.rs` Git blob is `2350af5bce75acab0d464e4fbfdd5b17e8c97e8c`. The exact code is retained in this ticket's commit history; no dependency/profile/toolchain changed.

## Three-track recording, two repeats

The private baseline project references three 600-second 48 kHz tracks and 18 references. It is the same recording/grid used for the earlier native baseline. The project SHA-256 stayed `6016da6272c77110b32ca5a97eb9bbbc5c5e6a9ed93b5777dcb1bdd090936325` before/after both runs. This is the private development copy, distinct from the untouched original project's `20bb912825964a2ba9b717525477f6af365abb3e434084cf7836e6b5725cbd98`.

| Run | Standalone / concurrent analysis s | Standalone / concurrent track-seconds per wall-second | Preparation s, standalone / concurrent | Ownership s, standalone / concurrent |
| --- | ---: | ---: | ---: | ---: |
| 1 | 22.660 / 23.258 | 79.44 / 77.39 | 10.812 / 11.136 | 10.692 / 10.968 |
| 2 | 22.321 / 23.739 | 80.64 / 75.82 | 10.722 / 11.283 | 10.317 / 11.185 |

Full analysis includes source validation, analysis-audio preparation, reference readiness, ownership, temporal processing and overview generation. The concurrent paired durations were 2.64% and 6.35% longer; two ordered repeats do not establish a statistically significant slowdown.

| Run / phase | Observations retained / overwritten | Completion median / p95 / p99 / max ms | Work p99 / max ms |
| --- | ---: | ---: | ---: |
| 1 foreground baseline | 92 / 0 | 1.871 / 2.467 / 2.864 / 2.864 | 2.225 / 2.225 |
| 1 during analysis | 1,055 / 0 | 2.051 / 2.504 / 2.695 / 3.256 | 2.197 / 2.519 |
| 2 foreground baseline | 92 / 0 | 1.801 / 2.362 / 2.557 / 2.557 | 2.273 / 2.273 |
| 2 during analysis | 1,077 / 0 | 2.050 / 2.505 / 2.771 / 3.285 | 2.247 / 2.576 |

Every phase observed 960 silent software samples without position advancement. Each phase includes an explicit seek; the aggregate does not attribute starvation to a particular instant or distinguish seek/startup from later work. These are software-consumer observations, **not native device underruns**. Callback deadlines, physical latency, listening and GPU presentation remain unmeasured.

An external PowerShell sampler observed the actual benchmark process every 250 ms, with 185 samples per run. Maximum sampled working sets were 59,699,200 bytes (56.93 MiB) and 59,072,512 bytes (56.34 MiB). Each approximately 49.4-second sample window includes standalone analysis, setup, foreground baseline and concurrent analysis together. These values are neither phase-aligned nor guaranteed memory peaks. Both report fields `peak_process_memory_bytes` and `native_device_underruns` remain explicitly null.

Numerical automatic regions and complete effective-envelope structures were exactly equal between standalone and concurrent results in both runs. Final source validation passed and saved project bytes were unchanged.

Portable raw reports: [run1](fabfilter-contention-run-1-2026-10-21.json), [run2](fabfilter-contention-run-2-2026-10-21.json). Their SHA-256 values are `fead331747c623a0684bad39490188255c5b29a29fb579f1d1c6b7ebcab84613` and `5ca4b9d357f79944d9d6baf859b8a67e4b36fa74985c3cb6a8af3d8707f79e55`. Private originals, the one-off sampler and raw OS companions are retained under `C:/Users/User/Documents/Codex/Investigations/cleanslate-performance-implementation-2026-10-21/` as `perf08-contention-run-{1,2}*.json` and `perf08-sample.ps1`. OS companion hashes are `0f9a154b6bd0286138829d5bcdf1278f09b04f8be787f2b861ca65a94e31b01a` and `02d8c8e9528728cf52a86f883a09db950bec0fe35ceb98ab7cac08836494aae2`.

## Second count/rate fixture and correctness

The public command/report test uses the existing source-bound supervised fixture: two four-second tracks at 44.1/48 kHz, resampled by the actual playback preparation to a 48 kHz output grid. One scoped run measured 455.48 ms standalone and 414.36 ms concurrent analysis, with 94 baseline and 19 active foreground observations. Completion median/p95/p99 were 1.249/1.657/5.424 ms baseline and 1.218/10.664/10.664 ms active; work p99 was 2.153/1.218 ms. The small active count makes its tails especially weak performance evidence. It verifies the report contract and rate/count path, not recording or hardware performance. Process memory was not separately sampled for this fixture.

TDD RED: `background_contention` failed because the existing public command rejected `--contention`. GREEN: the real mixed-rate pipeline/report test passed, including numerical parity, bounded-report counts, unsupported metrics, path/content privacy, project immutability and existing-report preservation. Scoped `background_contention`, `editor_workflow` and `playback_reclamation` passed 14 tests with one existing ignored owner-project preparation test. This includes cancellation retaining accepted analysis, source/label stale-result rejection and public playback output/reclamation coverage. Final scoped strict Clippy passed for the library, command and those tests after an equivalent integer-style fix; the final command build, formatting and diff checks passed. Root retains full combined checks/native GUI acceptance.

## Worker decision and remaining acceptance

`CleanSlateApp::launch_preview` admits one active analysis job and moves its retained worker into a dedicated background thread. `EditorAnalysisWorker::analyze_with_feedback` validates source identities and prepares each source before acoustic/temporal work. Playback independently reads bounded 4,096-frame blocks with a four-item forward channel; the consumer is nonblocking and retains the ticket03 reclamation behavior. Pinned inference runtime configuration is unchanged.

Queued Add tracks is distinct from full source/project replacement: `begin_source_loading(AddChannels)` calls `reserve_import`, `start_next_import` admits one recording job, and `poll_import_jobs` handles at most four events per poll. Acceptance invalidates stale preview generations while preserving the current playback mix until the next start. Full Open/Relink/Validate uses the separate source-loading path, which stops audition. This benchmark does not replace queued import's native acceptance or measure its waveform-cache throughput.

For measured analysis/preparation, foreground component work stayed well below the 20 ms rest interval, with active completion maxima below 3.29 ms and useful analysis throughput. No scheduling contention requiring a new budget, executor or priority policy was demonstrated. Retain the existing worker boundaries. Native redraw/input response, queued import throughput/current-mix behavior, phase-specific memory and actual device underruns remain separate acceptance; this ticket stays claimed until root records that native coverage. No long-session release or independent sound-quality qualification is claimed.
