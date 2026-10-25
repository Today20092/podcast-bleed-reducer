# Waveform and live-gate UI design contracts

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 89
Related: 85, 86

## Scope and acceptance

Turn the reference ledger into implementable CleanSlate interfaces, reusing the application's design tokens/actions. Work with existing UI skills and inspect current compact/full plugin renders. Deliver one shared interaction/performance contract and separate Podcast and Gate layouts. Record static-only inspiration separately from observed motion. Research incompleteness does not prevent original designs with stated assumptions.

Podcast: waveform-first shared session view, source-time history, capture coverage/readiness, drag selection/labels, explicit Analyze, per-track sound, audition, recoverable errors. Document desktop control parity including wheel/pinch, pan, range zoom, fit, navigator, Center, Catch up/Lock, focus and editable actions; distinguish exact parity from planned gaps. Gate: speech confidence, open/attenuated state, reduction, detector readiness and bounded gain controls; omit multi-track capture/labeling.

Acceptance:
- [ ] Save docs/design/plugin-ui-contract.md with compact/full layouts, type/spacing/color tokens, focus/keyboard rules, tooltips and empty/loading/error/stale states.
- [ ] Specify motion purpose, interruption behavior and reduced-motion behavior; do not couple gain timing to visual smoothing.
- [ ] Use captured baseline redraw/tessellation and peak-query timings from 84 as the performance baseline. Define reproducible interaction timing/frame-work/allocation measurements, bounded visible geometry and no audio-thread UI locks; do not invent presented-FPS or vendor equivalence.
- [ ] Review layouts at 100%/150%/200% DPI and smallest supported editor size using deterministic rendering where possible.
- [ ] Supply original reviewable mockups/renders and map every change to a product acceptance item; no renderer rewrite or new GUI framework without measured need.

## Results

Not started. Readiness describes specification only; all listed dependencies must be done before claiming.

