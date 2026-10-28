# Premium plugin visual and motion study

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: prototype
Blocked by: 90, 101
Related: 93, 94, 96, 103

## Owner direction

2026-10-27: the owner wants CleanSlate's plugins to have the visual quality, animations and performance of FabFilter or sonible. Apply this to Smart Gate and Podcast, retaining their separate tasks and existing CleanSlate identity. This authorizes original visual refinement and purposeful animation; do not equate a framework name or static screenshot with vendor performance.

## Scoped acceptance

- [x] Deliver one self-contained, original interactive visual study for both products, with synthetic data clearly identified. Use existing charcoal/teal/purple roles, readable typography, large signal plots, clear numeric controls and restrained chrome.
- [x] Demonstrate purposeful signal/readout motion, direct control feedback, pause and reduced-motion behavior. Bound demo geometry, stop animation when hidden and avoid external assets/dependencies/network activity.
- [x] Inspect representative frozen compact/full renderings and validate control/state/geometry behavior with a small self-check. Record any browser or native verification unavailable in this environment.
- [x] Amend the implementation contract and product tickets with explicit visual/motion and measured performance acceptance. Preserve DSP, parameter/state identities, source-time labels, safety and user settings.

## Boundaries

Design prototype only. Native implementation remains 94 and 103, after their dependencies. Final 96 review-fix validation remains outstanding. A browser study is not a new shipping GUI stack or a benchmark of egui/DAW presentation. Heavy compiler defaults remain 14/8/6 GiB; this turn measured 13.93 GiB physical availability, so no compiler or DAW is started.

## Results

Completed at the design-study boundary on 2026-10-27. [Interactive HTML](../../../docs/design/plugin-ui/visual-study.html), [four visually inspected compact/full SVG/PNG frames and verification notes](../../../docs/design/plugin-ui/visual-study.md), and a source/frame hash manifest are retained. The final self-check passes deterministic geometry, exact human-label gain, control feedback, one scheduled animation callback, pause/hidden/reduced-motion lifecycle and frozen-source parity. Impeccable's final mechanical check returned `[]`. Final lightweight check/render jobs each used less than 31 MiB peak owned commit, with cleanup verified; no native performance claim.

Chrome was absent and CUA exposed no browser provider, so browser layout/input, actual motion feel and presentation timing remain unobserved. This limitation is recorded, not treated as passing native animation. The updated shared contract, 94 and new 103 carry the real implementation/measurement obligations. No production source, framework, parameter/state identity, installed artifact, dependency or release changed. 94 remains blocked by 93; 103 remains blocked by 96 until its corrected final verification/integration passes.
