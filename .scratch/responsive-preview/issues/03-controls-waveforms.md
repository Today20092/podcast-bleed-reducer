# 03: Make controls and processed waveforms honest

**Working branch:** codex/responsive-controls
**Blocked by:** None
**Status:** ready-for-agent
**What to build:** Correct sensitivity presentation, actual padding increments, accurate bleed explanation and gain-aware waveform summaries.

- [ ] Higher displayed sensitivity admits weaker speech; old stored thresholds retain their meaning and sound.
- [ ] Padding control uses 32 ms effective increments and handles legacy values deliberately.
- [ ] Bleed helper text explains margin above each track's noise floor, not direct microphone competition.
- [ ] Processed waveform accounts for changing gain within a bucket rather than blindly using gain at the start; document conservative approximation if used.
- [ ] Behavioral checks cover transitions and parameter round-trip; retain accessibility and compact layouts.
