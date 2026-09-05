# 06: Evaluate safer detector preprocessing and bleed evidence

**Working branch:** codex/responsive-detector-quality
**Blocked by:** 01, 02, 03
**Status:** ready-for-agent
**What to build:** Tested anti-alias preprocessing and reproducible quality/bleed comparisons with conservative default decisions.

- [ ] Implement proper 48→16 kHz anti-alias preprocessing with signal-level checks and versioned analysis identity.
- [ ] Compare preservation, overlap, and bleed evidence with existing preprocessing on available labeled fixtures/reference sources.
- [ ] Evaluate cross-microphone evidence on concrete cases; document why a default change is accepted or rejected.
- [ ] Preserve legacy behavior by default if quality evidence is insufficient; make any experimental opt-in explicit and persisted, never claim listening performed.
- [ ] Keep comparison runnable and record findings; no unvalidated aggressive owner-selection change.
