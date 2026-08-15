# 05 — Review, protect, and rerender decisions

**What to build:** A user can inspect and audition automix decisions in the review workspace, persist reviewed states and unity-gain protections, and rerender until the project is current without directly editing gain or detector behavior.

**Blocked by:** 04 — Create and reopen a CleanSlate project.

**Working branch:** `feature/cleanslate-mvp`

**Status:** done

- [x] Show synchronized waveforms and a region-grouped review list with attenuation, ownership decision, evidence reasons, and warnings.
- [x] Selecting a review warning seeks playback to its attenuation region and highlights affected tracks.
- [x] Original/processed comparison preserves playhead and selection while switching audio.
- [x] Support track protection, time-range protection, and Protect this region; all force unity gain and combine additively.
- [x] Persist reviewed/unreviewed states and protections without hiding reviewed warnings.
- [x] Mark the render stale after protection changes and restore current comparison only after a successful rerender.
- [x] Do not expose confidence percentages, detector controls, force-attenuation, owner reassignment, or gain-envelope editing.

## Evidence

- TDD: `tests/review.rs` covers report-backed selection/comparison context and persisted additive protections with stale-render state.
- Verification: `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo build` pass.
- Standards/spec review completed against `53f2b98`; findings for warning grouping, typed decisions, playback, waveform rendering, selected-range protection, and surfaced persistence errors were addressed.
