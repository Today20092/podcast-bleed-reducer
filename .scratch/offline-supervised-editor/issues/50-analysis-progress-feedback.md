# Show useful feedback during whole-recording analysis

Status: done
Assignee: codex/analysis-progress
Blocked by: none
Spec: owner request, 2026-10-20; active migration specification cache/jobs contract

## Scope

The owner saw only Working for several minutes while analyzing a multichannel recording. Connect source preparation and speech-detection progress to the existing background analysis UI. Show the current track, processed recording time, elapsed wall time, and honest stage names. Keep existing library entry points, source integrity checks, cancellation, caches, and result installation behavior. Report measured completed-stage timings. Do not change models, source audio, or pretend stage progress predicts remaining wall time.

## Acceptance checks

- A real analysis fixture reports source preparation, within-track decoding, speech detection, training, temporal processing, and preview stages to its observer.
- Cached analysis and cancellation preserve existing behavior and previous completed results.
- The GUI consumes the same observer and keeps elapsed time and cancellation available; remove obsolete optional-speaker-model wording.
- Relevant analysis/import tests, formatting, and a GUI build through scripts/gui-dev.ps1 pass.
- Record observed owner-run evidence and distinguish it from controlled performance measurement.

## Comments

2026-10-20 completed checks: `scripts/gui-dev.ps1 -BuildOnly -TargetDir <retained-target>/analysis-feedback` passed (349.08 s for the resumed fresh-cache build). The separate executable is `<retained-target>/analysis-feedback/debug/rust-app.exe`; the owner's original `owner-ui/debug/rust-app.exe` retains its 2026-10-17 timestamp. `cargo test --locked --offline -j 1 --lib --test editor_workflow --test import_audio --target-dir <retained-target>/analysis-feedback --config scripts/gui-dev.toml -- --test-threads=1` passed: library 115 passed/10 ignored; editor workflow 12 passed/1 ignored; audio import 17 passed/1 ignored. This includes real within-track feedback, warm-cache reuse, and cancellation retaining the previous result while removing staging data. Formatting and diff checks pass. `cargo clippy --all-targets --all-features --locked --offline -j 1 --target-dir <retained-target>/analysis-feedback --config scripts/gui-dev.toml -- -D warnings` passed. Full owner-recording reanalysis, a native visual check, listening, and speed qualification have not been performed. These limits do not complete performance/quality gates 14/15.

Implementation: existing analysis entry points retain their behavior and delegate to an observer-enabled path. Source conversion reports decoded native frames; the GUI formats track/recording time. Speech detection reports track/frame position through the same observer instead of relying on its private atomic percentage. The GUI retains only the latest message, keeps elapsed time/cancellation, removes the obsolete optional-model status, and reports the existing completed-stage measurements after successful installation. Audio/model/cache recipes are unchanged.

Validation environment: initial separate-cache builds failed, first with missing cached rlib artifacts and then native compiler allocation failures in dependencies. A system counter sample showed 51,430,129,664 committed bytes against a 51,456,376,832-byte commit limit (about 26 MB free), despite approximately 6 GB of physical memory available. A compiler reported failure to allocate 2,096,640 bytes. This establishes build-time memory pressure; its contribution to the earlier analysis duration was not measured. No applications/services were stopped and no Windows memory settings were changed. A later sample showed approximately 13 GB of commit capacity available, and validation resumed with a separate target directory.

Source baseline: rebased onto 1e837dd5de435687f865fdd068819775d3af1890, the source identified by the running executable's dependency receipt, to preserve its alpha.5 metadata, native icon and intervening safety fixes. Ticket numbering was reconciled to 50 against this checkout's current inventory. The original shared checkout's uncommitted documentation is untouched.

Performance follow-up: measure the completed-stage timings before considering shared decoding/resampling of selected channels from one container. Preserve input and artifact integrity checks. No speedup, full-recording rerun, independent quality or release qualification is claimed by this feedback change.

2026-10-20: Claimed before code changes. Running PID 28528 remained responsive and accumulated 17.61 CPU seconds during a 3.07-second sample. The saved G:/261019/CleanSlate-2.cleanslate references T002, three channels of 143,855,712 frames at 48 kHz (49:57), despite the initial T001 report. Prepared-cache manifests finished at 15:13:38, 15:15:32, and 15:18:06 UTC. These are filesystem observations, not stage benchmarks. The owner confirmed completion before any process or binary changes. The executable dependency record identifies the application-icon source checkout; the offline-timebase path is a reused target directory.
