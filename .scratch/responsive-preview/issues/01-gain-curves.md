# 01: Construct shared gain curves at target changes

**Working branch:** codex/responsive-gain
**Blocked by:** None
**Status:** ready-for-agent
**What to build:** Fast exact envelope construction for preview and final render.

- [ ] Sweep region/protection target changes rather than every PCM sample.
- [ ] Preserve initial conditions, one-pole retargeting, gaps, overlaps, per-track/all-track protections, session-end sentinels.
- [ ] Differential and closed-form checks verify gain behavior; long-session construction benchmark shows reduced work.
- [ ] Existing callers remain compatible; commit tested implementation and evidence.
