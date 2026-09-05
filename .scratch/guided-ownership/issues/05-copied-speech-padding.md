# 05: Resolve copied-speech gaps without damaging short overlap

**Status:** needs-triage
**What remains:** Quality acceptance after the guided-ownership test build. The supplied screenshot's inferred 828–831 second interval still retains A02/A03 bleed while A01 is direct speech. Unprotected wanted speech also retains small existing recovery tails rather than exact unity.

The implementation and evaluation are delivered; these quality requirements remain open. See `../evidence/quality-results.md` for measured copy runs, padding behavior, sample-level preservation metrics, and rejected experiments.

- [ ] Confirm precise screenshot bounds and track mapping through listening; obtain distinct validation examples before further tuning.
- [ ] Reduce copied speech across ambiguous gaps while preserving 32/64 ms quiet independent words, filtered/reverberant overlap, laughter, and cyclic/isolated ambiguity.
- [ ] Avoid duration-only seed suppression and unsafe source-disqualification cascades already rejected by tests/review.
- [ ] Reconcile wanted-speech onset/recovery with exact-unity acceptance without changing its definition after measuring results.
- [ ] Keep cached preparation and measure actual envelopes, not only classifier labels; report bleed and wanted-speech changes separately.

## Comments

More aggressive experimental changes were discarded before packaging. This ticket records remaining work; no background task is scheduled and no listening sign-off is claimed.
