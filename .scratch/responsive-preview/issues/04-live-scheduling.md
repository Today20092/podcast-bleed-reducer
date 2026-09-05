# 04: Coalesce live preview updates while dragging

**Working branch:** codex/responsive-scheduling
**Blocked by:** 01, 02, 03
**Status:** ready-for-agent
**What to build:** Responsive live adjustments with bounded work and current-generation installation.

- [ ] One active computation and newest pending request; superseded work cooperatively stops.
- [ ] Coalesced drag changes update preview, initially targeting 30–50 ms cadence; release persists final state.
- [ ] Current result arrives promptly without 100 ms polling penalty; previous completed preview remains usable.
- [ ] A→B→C, source relink/import, protection updates, failures, and close/reopen never install obsolete work.
- [ ] No full detector restart on warm parameters; benchmark actual session warm median/p95.
