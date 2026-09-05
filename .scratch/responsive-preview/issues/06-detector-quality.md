# 06: Evaluate safer detector preprocessing and bleed evidence

**Working branch:** codex/responsive-detector-quality
**Blocked by:** 01, 02, 03
**Status:** ready-for-agent
**Completion:** implemented, tested, and reviewed; ready for parent integration
**What to build:** Tested anti-alias preprocessing and reproducible quality/bleed comparisons with conservative default decisions.

- [x] Implement proper 48→16 kHz anti-alias preprocessing with signal-level checks and versioned analysis identity.
- [x] Compare preservation, overlap, and bleed evidence with existing preprocessing on available labeled fixtures/reference sources.
- [x] Evaluate cross-microphone evidence on concrete cases; document why a default change is accepted or rejected.
- [x] Preserve legacy behavior by default if quality evidence is insufficient; make any experimental opt-in explicit and persisted, never claim listening performed.
- [x] Keep comparison runnable and record findings; no unvalidated aggressive owner-selection change.

## Evidence

See [comparison and reproduction commands](../evidence/06-detector-quality.md) and [per-label JSON](../evidence/06-detector-quality.json).

Filtering failed promotion: worse labeled preservation at threshold 0.20, tied at 0.50, worse bleed reduction at both. Legacy default retained. The filter exists only in the read-only quality runner, so there is no app opt-in or project migration. Strict baseline preservation already fails from small non-unity recovery tails; this is documented without changing the release gate. Known overlap and five quieter direct-speech examples reject a loudest-microphone default. No listening claim.
