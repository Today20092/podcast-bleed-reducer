# 02: Reuse source analysis for warm previews

**Working branch:** codex/responsive-analysis
**Blocked by:** None
**Status:** ready-for-agent
**What to build:** Retained detector facts, validated source identity and cache-aware preview derivation wired into the editor.

- [ ] Cache probabilities, energies and noise floors before thresholding; sensitivity/bleed/padding use those facts.
- [ ] Attenuation/Open/Close/protections reuse ownership when appropriate and rebuild only curves.
- [ ] Invalidate changed/relinked/ordered sources and preprocessing identity; preserve cold APIs and render checks.
- [ ] Avoid retained duplicate full PCM for analysis; demonstrate cold/warm equivalence and inference reuse.
- [ ] Stage reporting is meaningful; preserve existing async behavior for later scheduling ticket.
- [ ] Commit tests and real-session-capable benchmark entry point.
