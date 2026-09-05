# 03: Validate and tune guided ownership on the supplied recordings

**Working branch:** codex/ownership-quality
**What to build:** Reproducible evaluation of the integrated guided algorithm on existing labels and the screenshot failure, with calibration and scored audio kept separate.
**Blocked by:** 01, 02 and parent editor/API integration
**Status:** ready-for-agent

- [x] Read original sources without saving audio copies; preserve label provenance.
- [ ] Compare legacy, automatic joint, and guided joint behavior; do not train on scored spans.
- [x] Report bleed reduction, exact preservation and gain-depth/duration changes, overlap, and remaining failures.
- [x] Make evidence-driven conservative fixes if wanted speech is harmed or actual bleed remains incorrectly owned.
- [x] Provide a runnable comparison command and tested diagnostic support without unnecessary production controls.
- [x] Review findings resolved and measured limits documented honestly.

## Completion

Evaluation completed with known limitations; see ../evidence/quality-results.md for exact results and reproduction. All four requested modes were measured through the actual Rust preview path. The unchecked item is intentional: calibration intersects early scored wanted labels, and continued experiments inspected late scores. All observed real audio is now development evidence; no independent holdout acceptance is claimed.

Reviewed conservative corrections improve combined labeled bleed energy attenuation from −1.660171 dB to −2.023057 dB. No wanted samples are below −0.1 dB, but exact unity is imperfect. Screenshot bleed at 828–831 seconds remains fully open. Experimental stronger suppression was discarded because brief speech and ambiguous source handling were unsafe. The evaluation task is complete; the screenshot quality requirement remains unmet.

Final automated tests: 82 passed, five explicitly ignored, zero failed. Independent Standards and Spec final delta reviews: no code findings. Parent handles integrated performance, packaging and listening.
