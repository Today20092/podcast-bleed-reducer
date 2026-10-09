# Rust tooling for agent workflows

Research date: 2026-10-09. Scope: practical tools for this Windows Rust/Iced audio editor. This investigation installed nothing and changed no dependencies or roadmap tickets.

## Recommendation

Start with the existing compiler, rustfmt, Clippy and focused tests. Give the agent structured compiler diagnostics and an explicit acceptance checklist. Add rust-analyzer semantic queries only when the existing code graph cannot answer a type, reference or macro question. Evaluate one MCP bridge before installing several. Consider nextest after measuring this project's test suite on Windows. Use cargo-deny when adopting dependency policy, or cargo-audit for the smaller advisory-only check. These are recommendations based on the documented capabilities below, not measured improvements in this repository.

## What this checkout already has

Local inspection found a single Rust 2024 package, Iced 0.14, audio/decoder/resampler dependencies, and native-runtime integration through ort and libloading in [Cargo.toml](../../Cargo.toml). Installed rustup components include Clippy and rustfmt for x86_64-pc-windows-msvc. `cargo --list` exposed fmt, clippy and a miri command, but installed components did not establish a working Miri toolchain. No nextest, deny or audit command appeared. No rust-analyzer component appeared in that installed-component listing; this does not rule out an editor-bundled or separately installed binary.

The inspected paths had no `.github/workflows`, `rust-toolchain.toml`, `.cargo/config.toml` or `deny.toml`. This is a scoped inspection, not an inventory of every configuration. The [development guide](../development.md) already requires all-target tests, formatting and all-target/all-feature Clippy with warnings denied. Preserve those final checks. The existing [GUI launcher](../../scripts/gui-dev.ps1) builds locked/offline with one build job, a separate target directory and [GUI build config](../../scripts/gui-dev.toml). Reuse it for GUI work rather than inventing a parallel build workflow.

The [active specification](../../.scratch/offline-supervised-migration/spec.md) requires cancellation, atomic saves/exports, native-rate timing, one envelope evaluator and independent audio-preservation evidence. Tool checks cannot replace those domain acceptance checks or owner listening approval. The existing graph-first discovery and Context7 instructions already cover much of an agent's navigation and documentation needs.

## Smallest useful feedback loop

| Tool | What the agent gains | Suggested use |
| --- | --- | --- |
| rustfmt | A consistent format without agent style debates | Check formatting on the final diff. Official usage documents `cargo fmt --check`. [rustfmt](https://github.com/rust-lang/rustfmt) |
| Clippy | Compiler-backed lint feedback, including test targets | Run scoped Clippy, then the required final targets. `-- -D warnings` can enforce a clean baseline once existing warnings are understood. Do not automatically enable every pedantic/restriction lint. [Clippy CI guidance](https://doc.rust-lang.org/clippy/continuous_integration/index.html) |
| Cargo JSON | File spans, error codes and suggestions in structured records | Request JSON for check/Clippy; filter `reason == "compiler-message"`, preserve useful child messages and exit status. Cargo emits other record types too, so do not treat every JSON line as a diagnostic. [Cargo external tools](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages) |
| rust-analyzer | Type information, semantic definitions/references, macro-aware analysis and code actions | Complement graph discovery when semantic resolution matters. Its official project is an LSP server; an editor extension alone does not give an agent MCP tools. It can run Clippy by setting `rust-analyzer.check.command` to `clippy`. [rust-analyzer book](https://rust-analyzer.github.io/book/), [diagnostics](https://rust-analyzer.github.io/book/diagnostics.html) |

Suggested commands for a future agreed check workflow, not commands executed by this research:

```powershell
cargo fmt --all -- --check
cargo check --locked --all-targets --message-format=json
cargo clippy --locked --all-targets --all-features --message-format=json -- -D warnings
cargo test --locked <relevant_test_filter>
```

Use `--offline` when the required dependency cache is ready, as the GUI launcher already does. Scoped iteration can use the relevant feature configuration; final Clippy must retain this repository's documented `--all-features` requirement. Require the ticket's broader acceptance checks after the scoped iteration passes. Cargo diagnostics describe compilation, not audio quality.

## MCP and plugin candidates

These bridges are third-party projects. Their documented capabilities do not establish reliability, maintenance quality or compatibility with this checkout. Rust-analyzer itself is the established Rust project; treat its MCP wrappers as candidates for a small trial.

| Candidate | First-party evidence | Fit and limitations |
| --- | --- | --- |
| [dexwritescode/rust-mcp](https://github.com/dexwritescode/rust-mcp) | README describes a native Rust server wrapping rust-analyzer, with definitions, references, diagnostics and refactoring/generation tools. It documents an override for the analyzer executable path. | A Rust-specific native-server candidate. Start with read-only queries. The published defaults and example executable paths need Windows adaptation. No Windows smoke test was performed. Broad generation/refactoring claims are README claims, not independently verified guarantees. |
| [r3bl-rust-analyzer-mcp-server](https://docs.rs/r3bl-rust-analyzer-mcp-server/latest/r3bl_rust_analyzer_mcp_server/) | Published crate documentation describes a stdio MCP-to-LSP bridge for hover, definitions, references, symbols, diagnostics, formatting and code actions. | A narrower Rust-specific bridge worth comparing. Its published documentation explains tool mappings, but does not establish successful native Windows operation for this project. |
| [blackwell-systems/agent-lsp](https://github.com/blackwell-systems/agent-lsp) | README describes language-server orchestration for agents and documents Windows installation paths. | A cross-language candidate if one bridge should serve several repositories. A documented Windows installer is not a tested Rust/Iced integration. Its wider tool set adds evaluation scope. |
| [raymondshum/rust-lsp-mcp](https://github.com/raymondshum/rust-lsp-mcp) | README documents container-hosted rust-analyzer/Python dependencies, stdio MCP, read-only project mounts and explicit indexing readiness states. | Useful exploration design, but Docker and a separate toolchain add overhead here. Container analysis can differ from Windows-specific compilation. Lower priority for this native Windows editor. |

An available-plugin-directory search for Rust, rust-analyzer and cargo surfaced no relevant plugin. This is a search result, not proof that no Rust plugin exists anywhere.

A bridge trial should answer a known symbol's type, find trait/macro-sensitive references, reflect an edited file and return one deliberately introduced compiler diagnostic. Compare its results with Cargo and existing graph tools. Test workspace paths containing spaces, indexing readiness and Windows-only code. Keep automatic edits disabled during this trial. Retain the bridge only if it answers useful questions the current tools miss.

## Tools to add when a concrete need appears

| Tool | Benefit | Decision for this project |
| --- | --- | --- |
| [cargo-nextest](https://nexte.st/) | Test filtering and separate test processes improve test management. Supports Windows. | Benchmark before adopting. Its Windows documentation warns of process-launch overhead; speedup is not guaranteed. Keep `cargo test --doc`, because nextest does not run doctests. [Windows guidance](https://nexte.st/docs/installation/windows/), [running limitations](https://nexte.st/docs/running/) |
| [cargo-audit](https://rustsec.org/) | Checks Cargo.lock against known RustSec vulnerability advisories. | Smallest advisory-only addition. Results depend on the advisory database, and passing does not prove dependencies are safe. |
| [cargo-deny](https://embarkstudios.github.io/cargo-deny/) | Checks advisories, licenses, banned dependencies and allowed sources. | Prefer this when the team wants explicit dependency policy. Do not routinely duplicate its advisory check with cargo-audit without a reason. Neither tool qualifies externally acquired runtime/model binaries. [RustSec tooling comparison](https://rustsec.org/) |
| [proptest](https://github.com/proptest-rs/proptest) | Generates constrained inputs and shrinks failing cases. | Add only for a focused invariant that ordinary boundary tests cannot cover adequately. Candidate properties are time conversion bounds, conservative preserved-region rounding, envelope parity and edit/save/reload equivalence. These are proposed tests derived from the specification, not an audit of existing coverage. Keep explicit extreme-value examples because randomized tests can miss isolated edge cases. |
| [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) | Runs libFuzzer targets against parsers and other input boundaries. | Target project-file parsing or small metadata/interval validators when justified. Official prerequisites require nightly, LLVM sanitizer support and Unix-like systems, not native Windows. Use an isolated Linux/WSL runner for those targets; it will not qualify Windows GUI/device behavior. |
| [Miri](https://github.com/rust-lang/miri) | Executes Rust under an interpreter to detect certain undefined behaviors. | Use a separate nightly toolchain for focused unsafe/pure-Rust tests if needed. Most platform APIs and arbitrary FFI are unavailable. The project documents Linux-target interpretation as useful on Windows and experimental native-library support only on Unix. Do not expect it to validate the whole Iced/ONNX/audio-device application. |

## Adoption order

1. Standardize the existing agent check loop and preserve the GUI launcher and ticket acceptance requirements.
2. Trial one semantic MCP bridge only if graph plus Cargo feedback leaves real navigation gaps. Prefer comparing the native Rust-specific candidates first; consider agent-lsp if multi-language reuse matters.
3. Choose cargo-audit or a reviewed cargo-deny policy for dependency checks. Measure nextest separately on the owner's Windows machine before changing test execution.
4. Add targeted property tests, fuzz targets or Miri checks only for a demonstrated boundary or unsafe-code risk. Preserve deterministic regression cases and the product's independent audio acceptance evidence.

Context7 documentation was fetched for rust-analyzer, nextest and Rust/Clippy compiler feedback. Other facts above come from the linked project-owned documentation and source repositories. No installations, benchmark comparisons, security audit, bridge smoke test or application test suite ran during this investigation.
