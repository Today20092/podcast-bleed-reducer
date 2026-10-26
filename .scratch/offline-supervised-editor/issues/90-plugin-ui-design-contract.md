# Waveform and live-gate UI design contracts

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 89
Related: 85, 86

## Scope and acceptance

Turn the reference ledger into implementable CleanSlate interfaces, reusing the application's design tokens/actions. Work with existing UI skills and inspect current compact/full plugin renders. Deliver one shared interaction/performance contract and separate Podcast and Gate layouts. Record static-only inspiration separately from observed motion. Research incompleteness does not prevent original designs with stated assumptions.

Podcast: waveform-first shared session view, source-time history, capture coverage/readiness, drag selection/labels, explicit Analyze, per-track sound, audition, recoverable errors. Document desktop control parity including wheel/pinch, pan, range zoom, fit, navigator, Center, Catch up/Lock, focus and editable actions; distinguish exact parity from planned gaps. Gate: speech confidence, open/attenuated state, reduction, detector readiness and bounded gain controls; omit multi-track capture/labeling.

Acceptance:
- [x] Save docs/design/plugin-ui-contract.md with compact/full layouts, type/spacing/color tokens, focus/keyboard rules, tooltips and empty/loading/error/stale states.
- [x] Specify motion purpose, interruption behavior and reduced-motion behavior; do not couple gain timing to visual smoothing.
- [x] Use captured baseline redraw/tessellation and peak-query timings from 84 as the performance baseline. Define reproducible interaction timing/frame-work/allocation measurements, bounded visible geometry and no audio-thread UI locks; do not invent presented-FPS or vendor equivalence.
- [x] Review layouts at 100%/150%/200% DPI and smallest supported editor size using deterministic rendering where possible.
- [x] Supply original reviewable mockups/renders and map every change to a product acceptance item; no renderer rewrite or new GUI framework without measured need.

## Results

Claimed 2026-10-26 after ticket 89 completed at 10a1ebe. Scope is an implementable design contract and original compact/full mockups, with deterministic DPI inspection where possible; no product GUI implementation, framework replacement or heavy build is included in this design pass.

Completed 2026-10-26 on `codex/plugin-ui-contract-90`, based on development `ac2bdda3135eb60345cad45ed98f67d6d29426e1`. The [design contract](../../../docs/design/plugin-ui-contract.md) supplies shared source-backed tokens/actions, separate Podcast P1–P7/Gate G1–G6 layouts, current-versus-planned desktop parity, focus/keyboard/tooltips, state/recovery and motion/reduced-motion rules. It retains 84's exact draw/tessellation and peak-query observations, with a reproducible measurement plan and bounded visible geometry/audio-thread ownership contract.

Four original editable SVGs and twelve PNGs at compact/full × 100/150/200% are retained under `docs/design/plugin-ui/`, with generator, renderer, dimension and SHA256 manifests. Actual 84 compact/full captures were inspected first. All twelve corrected outputs were visually inspected; the final targeted Podcast correction was re-inspected at all three scales to ensure the complete human-label span retains illustrated unity. No native performance/FPS/vendor-equivalence claim is made.

Checks passed: generator control bounds/non-overlap, text contrast and artifact bounds; twelve exact physical dimensions/hashes; 28 document-local links; four self-contained parsed SVGs; renderer PowerShell syntax; `git diff --check`. Impeccable context ran once, mechanical detector once (`[]`), with inline craft/documentation review within the sequential task. Graph search returned Transport closed; narrow source reads were used. Root DESIGN.md was absent, so existing source/product/capture identity was retained without creating a replacement system.

Rendering used 88's guard sequentially with an explicit lightweight 0.5 GiB cap, 2 GiB start/1 GiB reserve and 90-second timeout, leaving heavy defaults intact. Final receipt `C:/Users/User/Documents/Codex/Investigations/plugin-ui-contract-90-render-03.json`: 4.22 s, kernel peak commit 126.76 MiB, maximum sampled working set 163.57 MiB, cleanup verified. Earlier bounded receipts retain renderer opacity/font fixes. No new dependency, framework, GUI/DSP implementation, DAW, vendor assets, installed-link change or build was introduced.

Remaining implementation acceptance: native focus/keyboard/assistive technology, host DPI negotiation, pinch/held modifiers, motion feel, per-track sound, waveform overlays and actual DSP/presentation measurements. Gate's minimum dimensions/timing values are proposed design fixtures, not shipping support/defaults. These explicit gaps proceed under 94/96 and detector/DSP 92/93; they do not block this design ticket's scoped acceptance.

