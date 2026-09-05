# 03: Validate and tune guided ownership on the supplied recordings

**Working branch:** codex/ownership-quality
**What to build:** Reproducible evaluation of the integrated guided algorithm on existing labels and the screenshot failure, with calibration and scored audio kept separate.
**Blocked by:** 01, 02 and parent editor/API integration
**Status:** ready-for-agent

- [ ] Read original sources without saving audio copies; preserve label provenance.
- [ ] Compare legacy, automatic joint, and guided joint behavior; do not train on scored spans.
- [ ] Report bleed reduction, exact preservation and gain-depth/duration changes, overlap, and remaining failures.
- [ ] Make evidence-driven conservative fixes if wanted speech is harmed or actual bleed remains incorrectly owned.
- [ ] Provide a runnable comparison command and tested diagnostic support without unnecessary production controls.
- [ ] Review findings resolved and measured limits documented honestly.
