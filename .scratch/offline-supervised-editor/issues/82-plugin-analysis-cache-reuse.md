# Reusable plugin Session analysis cache

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 80

The 0.0.5 candidate persists raw capture, waveform records, label content fingerprints and cache identity, but regenerates immutable analysis snapshots/envelopes for each Analyze and after reopening. Complete reusable analysis caching without weakening source validation or coupling it to streaming epochs.

- [ ] Reuse maintained prepared features/models when native source content is unchanged and only labels or sound controls change.
- [ ] Persist source-bound analysis/envelopes for reopening, rejecting stale/incompatible results and keeping unplayed data unverified until host input confirms it.
- [ ] Keep unknown captured islands/gaps original; extend analysis beyond one common continuous interval only with explicit validity semantics rather than zero-filled training input.
- [ ] Bound disk generations/RAM, support cancellation, preserve references and provide clear invalidation/coverage feedback.
- [ ] Measure representative long-session memory/disk/time and cache reuse; do not generalize the four-second synthetic receipt.
