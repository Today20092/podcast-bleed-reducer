# 04: Coalesce live preview updates while dragging

**Working branch:** codex/responsive-scheduling
**Blocked by:** 01, 02, 03
**Status:** resolved
**What to build:** Responsive live adjustments with bounded work and current-generation installation.

- [x] One active computation and newest pending request; superseded work cooperatively stops.
- [x] Coalesced drag changes update preview at a 40 ms request boundary; release persists final state.
- [x] Active preview polling uses 10 ms; previous completed preview remains usable.
- [x] A→B→C, source relink/import, protection updates, failures, and close/reopen reject obsolete work.
- [x] No full detector restart on warm parameters; parent measured actual session warm median/p95.

## Evidence

See [scheduling.md](../evidence/scheduling.md). App lifecycle tests cover initial preparation retention, controlled sustained input, source replacement, protections, and failure recovery. Parent owns integrated UI cadence and full-session acceptance.
