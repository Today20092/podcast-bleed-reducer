# Install and verify the required Nemotron model and native runtime

Status: done
Assignee: finish_runtime
Blocked by: 01
Spec: ../spec.md

## Scope

Pin a NeMo-Speech.cpp revision, supported Nemotron V3 GGUF revision and expected model hash. Bundle the Windows native runtime/dependencies. Add an explicit first-use model download with progress/cancellation and actual runtime/model license notices. The runtime is Apache-2.0; weights use OpenMDW 1.1. Verify pinned redistribution/acquisition obligations and source URLs.

Download to a temporary file, verify identity/integrity, then atomically install in the local model cache. A failed/cancelled download cannot replace a good model. After installation, model loading and inference work offline without Python, keys or a GPU requirement. Do not auto-update model versions or silently fall back to an optional detector when the required model is absent.

## Reach

New narrow model lifecycle/native deployment responsibility, src/app.rs first-use state, project analysis provenance, packaging/build scripts and Cargo integration. Follow current pinned SDK/build documentation in feasibility notes.

## Acceptance

- A clean Windows machine with no Python can install the runtime, download once and load the verified model.
- Network-disabled relaunch loads the installed model; no startup/background download is attempted.
- Missing/corrupt model, bad checksum, partial download, incompatible binary and unavailable network produce actionable errors with existing results intact.
- Cancellation leaves no installed incomplete model; notices distinguish native-code and weights licenses.
- The exact dependency/model recipe is reproducible in development and release packages.

## Implementation evidence (2026-10-08)

- Required model pinned at `f667ed73aee57d40cc39428eb768b4fd87a0a29e`, q8_0 artifact 107012128 bytes, SHA-256 `08456d9e22cd9a323c0364d98375f3746d6e68507ebb705cd46438c534c7a3a1`. Actual downloaded bytes verified. First-use acquisition is explicit and retains the OpenMDW agreement/origin notices beside the local model.
- Windows CPU SDK built from `8642eaa5cc51efbc17ad0f3e433944ba858a873f`, ggml gitlink `bd4f514db14d87fded667787a7a963bfbaa98e89`, vcpkg `9e593bb18ea69cc5095e012465dcd675a822ed0d`. MSVC CRT 14.44.35112, CMake 3.31.6-msvc6, Ninja 1.12.1. CPU AVX2/FMA/F16C; ASR/CLI, GPU, AVX-512, native-host tuning and OpenMP disabled. Actual binary hashes, native licenses, transitive vcpkg copyrights and MSVC redistribution notices accompany the installed package.
- Acquisition tests cover checksum/partial-transfer failure, offline local relaunch, unavailable native package, incompatible/corrupt binaries, stalled-transfer cancellation and retaining a good install. Native verification now runs in an owned `model-check` process: bounded/drained stdout/stderr, explicit cancellation, verified provenance and worker joining on close. An actual abort in an owned test-harness child produces an actionable parent error; the good model remains intact. No alternate detector is substituted.
- Prior isolated/proxy-only probes hit native allocation failures, including GGML's unconditional allocation assertion. The child-process boundary is required even if these failures cannot be reproduced. A controlled normal / isolated / normal sequence with the build lane idle passed all three probes: 101 frames, eight speakers, one second of silence; elapsed 3.44 / 3.26 / 3.33 seconds. The isolated run used only `%SystemRoot%/System32` on PATH and HTTP/HTTPS proxies `http://127.0.0.1:9`; it required no Python or GPU. This exercises unreachable HTTP proxies, not a claim that the computer's network interface was disabled.
- All three controlled runs used executable SHA-256 `981f7dbed89b856a79e57163591af99fd574e6900648a571c174917b665c9524`, native entry DLL SHA-256 `3abbb22cf6f390ee7904c164ee52dcf7d91df3a7f6641b5d6b06d87ff9c79fa4`, and the pinned model digest. Free system commit before the runs was 5.46 / 5.44 / 5.43 GB. The earlier resumption snapshot had 1.97 GB free commit; the precise cause of the earlier allocation failures is not established. No unowned process or global system setting was changed.
- Installed native binaries, actual notices and probe/build logs are preserved outside the worktree at `C:/Users/User/.codex/backups/podcast-automix-rust-gui/ticket05-native-runtime/`. Packaging includes the exact helper, app-local DLL closure and notices. A failed Rust release build retains the prior package.

Long-recording probability retention/performance belongs to ticket 06. Clean-machine Windows release qualification, physically network-disabled processing, quality/listening approval and release packaging verification remain gates in tickets 14/15; the one-second silence probes establish loading/inference compatibility only.

After merging integration tip `6d51799`: formatting check, `cargo clippy --all-targets --all-features --locked -j 1 -- -D warnings`, and `cargo test --all-targets --locked -j 1` passed. The model lifecycle suite passed eight tests; two ignored fixtures are executed only as owned child processes by the abort/cancellation regression tests. The helper regression followed a failing missing-public-function test before implementation. No Windows release or long-session success is inferred from these development checks.
