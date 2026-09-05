# Integration checks

Integration branch: `codex/cleanslate-ownership`, based on responsiveness build `135fbb1`. No changes merged into main.

## Merged implementation

- Calibration editor/project: `3c69856`, `edda647`; completion `561d5eb`.
- Cached joint detector: `0cd5f6e`, persistent-overlap safety fix `ae33bc3`; completion `a46f733`.
- Editor/API wiring, shared calibration validation, calibration benchmark: `896141e`.
- Render completion checks project, source stamps, parameters, protections and ownership settings. Live slider/export guard: `945314f`.

The editor-test merge conflict was resolved by retaining both the calibration tests and render-completion tests.

## Verification so far

- `896141e`: `cargo test --all-targets --quiet` passed 75 tests; four existing/explicit optional benchmarks or local visual checks ignored.
- `945314f`: all 17 editor tests passed, including render completion during an unreleased slider drag and direct export refusal during that drag.
- `945314f`: `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings` passed.
- Independent Standards review: no actionable findings at `896141e`.
- Independent Spec review: live-slider completion/export gap fixed and re-reviewed clean at `945314f`.
- Quality corrections `efcf0e7` and `1dc1f2a` merged in `fb321b5`: independent final Standards and Spec delta reviews both report no actionable findings. Their scope includes padding, per-track speech evidence, reliable residual fallback and rejection of incoherent solo examples.

These checks do not constitute real-recording quality acceptance or a listening sign-off. Task 03 measures the recordings; final performance, release build and packaging results follow integration of its findings. The user will open and listen to the application themselves.

## Protection regression found during quality testing

The prior engine only set a unity target inside protection ranges, so samples remained attenuated during recovery. A targeted test reproduced gain 0.20826721 at the first protected sample, where the domain contract requires 1.0. The shared envelope now pins gain to 1.0 for the half-open protection interval, including entry during an existing unity-target recovery. Overlapping/session protections and sequential/random access agree. Ordinary unprotected one-pole behavior is unchanged. The rendered-float regression now requires every protected sample to equal its original value, replacing the old test that explicitly expected a recovery ramp within protection. Entry is immediate; the guide explains boundary placement.

Commit `003d372`: six engine tests passed (one optional benchmark ignored), all six public render tests passed, and formatting passed. Independent Standards and Spec reviews reported no actionable findings for this delta.

## Final integration

- Final quality test/evidence commit `67f8643` integrated as `17a8805`. The conservative production classifier remains the reviewed `1dc1f2a`; experimental suppression changes were rejected.
- Integrated all-target locked tests: **82 passed, five ignored, zero failed**. Formatting and strict all-target locked Clippy passed. This resolves the quality branch's older editor warning through the already integrated editor changes.
- Independent final Standards and Spec delta reviews both clean. They explicitly distinguish clean code review from unmet screenshot suppression and exact-unity unprotected wanted-speech criteria.
- Optimized full-session benchmark: one preparation, unchanged restored preview, joint-control p95 below 49 ms. See `performance-results.md` and raw results for memory and measurement scope.
- The screenshot's A02/A03 bleed at inferred 828–831 seconds remains open. Guided classification modestly improves aggregate labeled bleed. This is a testable implementation with documented limits, not full quality acceptance or a listening sign-off.
