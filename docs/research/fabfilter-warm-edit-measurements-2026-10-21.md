# Warm sound and gain editing measurements

Ticket 05 measures source-bound synchronous envelope composition independently of inference and drawing. It retains unchanged tracks' complete effective preparation when their source grid, automatic envelope and relevant manual/protection constraints are identical. A draft never installs its provisional result. The accepted cache retains one preparation per track; it does not accumulate draft history or expose playback snapshot ownership.

## Reproduction and scope

`tests/warm_edit_measurements.rs` is an explicitly ignored measurement at the existing public effective-envelope boundary. Run it with the pinned toolchain and shared local settings, while holding the coordinated Cargo lease:

```powershell
$settings = & ./scripts/local-build-settings.ps1
$env:CLEANSLATE_BUILD_COMMIT = (git rev-parse HEAD)
$env:CLEANSLATE_WARM_EDIT_REPORT = 'C:/absolute/path/to/a-new-report.json'
cargo test --manifest-path ./Cargo.toml --test warm_edit_measurements @($settings.CargoArguments) -- --ignored --nocapture
```

The report refuses to replace an existing file. It contains no private source paths. The commit field is build metadata supplied by the command, not artifact attestation: the retained baseline was built on `83ba48463f23ae3693985dea5cd7c02f28a0042a` with the uncommitted measurement driver and an observer initially reporting zero reuse; the production composition behavior was unchanged. The optimized run uses integration base `9446d3e` plus this ticket's changes. Both use the repository's local optimized package settings, test profile and enabled debug assertions on Windows x86_64. This machine reports 32 logical processors, `AMD64 Family 23 Model 113 Stepping 0, AuthenticAMD`. No simultaneous Cargo build ran while a measurement executable was sampling.

Each scenario performs 25 alternating attenuation updates, 25 single-track draft compositions and 25 pairs of envelope-vector deliveries. The independent expected draft gains alternate 0.4/0.6; drafts preserve the accepted result, reuse automatic preparation and leave the saved project unchanged. Setup, WAV creation, source import/fingerprinting and temporal optimization are outside the timed spans. Counts and all chronological repetitions are retained in the JSON reports.

| Scenario | Sources | Duration | Ownership frames per track | Ownership regions | Activity half-period |
| --- | --- | ---: | ---: | ---: | ---: |
| Native grid | Three 48 kHz mono silent WAVs | 60 s | 1,875 | 30 | 64 frames / 2.048 s |
| Long sparse | Eight 8 Hz mono silent WAVs | 7,200 s | 225,000 | 1,758 | 128 frames / 4.096 s |
| Long dense | Eight 8 Hz mono silent WAVs | 7,200 s | 225,000 | 28,125 | 8 frames / 0.256 s |

Each draft adds one constant manual edit with two boundary points, from one to two seconds on the first track. The dense fixture exercises automatic boundary density, not a large manual-point collection.

The vector-delivery observation times two outer-vector clone/drop operations. It excludes the audition setter, device handoff, callback processing and any end-to-end audio response; those costs require their own playback observations.

The long fixtures are genuine source-bound two-hour time coordinates using valid small 8 Hz files, with dense 32 ms temporal evidence. They are synthetic component fixtures, **not ordinary-rate two-hour PCM, import, inference, export or listening qualification**. They do not bypass or weaken source validation. Duration and boundary density are separate variables. The 48 kHz case supplies an ordinary-rate component check; it is still synthetic evidence rather than a trained real-recording benchmark.

## Decision and limitations

The first exploratory run showed long-dense draft p95 35.449 ms, dial p95 29.360 ms and two private-vector deliveries p95 0.003841 ms. The retained repeat below confirms that full per-track edge preparation, rather than deep copying, warranted a narrow reuse change. Effective `GainEnvelope` clones share their dense curve through an existing internal `Arc`; no new shared playback-vector API, generic asynchronous scheduler, inference rerun or processing-quality mode is introduced.

A 16.7 ms frame-sized diagnostic review threshold motivated investigating the dense synthetic case. It is not a measured monitor refresh interval, device deadline or universal interaction guarantee. Draft preparation can improve while dense attenuation changes still rebuild every affected automatic envelope; that residual is reported rather than hidden by an asynchronous completion claim.

The separate [native baseline](fabfilter-performance-baseline-2026-10-21.md) includes a warm attenuation gesture on a three-track 600-second 48 kHz private project. Its rolling mixed-run input/update samples cannot isolate that gesture's latency. These component observations do not measure actual displayed frames, physical input-to-photon latency, audio onset or callback deadlines; separate native and listening checks remain necessary for those claims.

## Retained observations

The [baseline JSON](fabfilter-warm-edit-baseline-2026-10-21.json) and [retained-preparation JSON](fabfilter-warm-edit-retained-2026-10-21.json) contain all 25 chronological repetitions per boundary. Measurements are elapsed wall time and include thread scheduling effects. Repeated sequential runs on one machine do not establish a statistical speedup or worst-case bound across hardware.

| Scenario | Dial median / p95 before, ms | Dial median / p95 after, ms | Draft median / p95 before, ms | Draft median / p95 after, ms |
| --- | ---: | ---: | ---: | ---: |
| Native grid | 0.344 / 0.522 | 0.343 / 0.402 | 0.412 / 0.588 | 0.405 / 0.538 |
| Long sparse | 2.562 / 3.179 | 2.486 / 3.678 | 2.948 / 4.259 | 2.051 / 3.027 |
| Long dense | 26.114 / 32.183 | 23.190 / 25.690 | 31.024 / 43.496 | 19.545 / 22.396 |

Every optimized draft retained two of three complete tracks in the native-grid case and seven of eight in both long cases. All 25 drafts per scenario retained their automatic preparation; 25 changing attenuation settings plus initial composition caused 26 automatic builds. The two private-vector deliveries' p95 before/after was 0.002374/0.001885 ms, 0.002863/0.002374 ms and 0.009917/0.004261 ms respectively. Those small observations support leaving delivery ownership unchanged; timing differences in that scale are not a claimed optimization.

Dense draft and dial p95 remain above the diagnostic 16.7 ms review threshold. This change removes demonstrated redundant preparation; it does not promise frame-sized completion for every dense synthetic edit. Native and sparse component cases do not justify a broader scheduler or inference change. Larger manual-point collections, other hardware and actual GUI/audio onset need separate measurements.

The benchmark driver SHA-256 at capture is `3094cbc66181b584362360b788a8fedc4161deb91dd5725673cf164c89c00bc7`. Original report byte hashes: baseline `11ccb5d03bc0f5fe9ed526f23f3a65a52fafbd1ef1e47d053cfaa3f1757f5994`, retained `89dde914616836d6bf30e778af2f4b22fc45ecb30e8abf12e316f0d0fcbb2be4`. Git checkout line-ending conversion may change report byte hashes; the JSON values and chronological samples are portable.

## Validation

At the owner-confirmed public cache/report seam, `a_single_track_draft_retains_other_tracks_complete_preparation` failed before the reuse implementation (expected one retained track, observed zero) and passed afterward. It independently checks the edited gain, unaffected gain, accepted-result retention, automatic invalidation and a subsequent edit to the other track. An added attenuation assertion initially revealed an incorrectly declared synthetic global-speech row; correcting that fixture to known silence preserved the production rule that uncertain audio stays at unity.

Final scoped checks passed: 14 effective-envelope tests, five supervised-delivery tests (including mixed native-rate audition/export shared evaluation), four gain/undo/analysis-identity/inference-reuse workflow tests, the latest-edit/obsolete-completion workflow test, and the ignored 25-repeat observation test. Strict Clippy for the library and changed tests, formatting and diff checks passed. Source changes, cancellation, one atomic undoable drag, manual authority and stale-result publication protections remain covered by those public tests. The integration tip was merged before these final checks. No physical feedback or listening check is marked passed by these component results.
