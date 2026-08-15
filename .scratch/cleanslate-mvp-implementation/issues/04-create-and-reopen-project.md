# 04 — Create and reopen a CleanSlate project

**What to build:** A user can follow the desktop workflow rail through import, validation, and processing, then close and reopen the project without losing source identity, review state, protections, or render freshness.

**Blocked by:** 03 — Pass the engine release gate.

**Working branch:** `feature/cleanslate-mvp`

**Status:** done

- [x] Present the selected Import → Validate → Process → Review → Export workflow rail with one primary action for the current step.
- [x] Import one or more stems and show render blockers and the unverified-alignment warning using the engine's validation contract.
- [x] Start and cancel processing, report progress, and enter Review only with a successfully committed render.
- [x] Autosave source identities, protections, warning review states, and current or stale render identity in the project.
- [x] Reopen the project with the same workflow and review state.
- [x] Require relinking and full revalidation when a source is missing or changed; sources remain untouched.

## Evidence

- `cargo test --all-targets`: 12 passed, 0 failed.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`: passed.
- `tests/project.rs` covers durable reopen state and changed-source invalidation; existing render tests cover atomic cancellation and untouched sources.
