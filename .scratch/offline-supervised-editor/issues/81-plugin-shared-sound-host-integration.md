# Session sound overrides and host integration

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 80

Complete the accepted shared-control contract after the Session candidate's native acceptance. Current Session defaults are shared persisted fields; legacy host sound parameters are intentionally not mutated remotely.

- [ ] Provide per-track reduction/fade overrides editable from any participant, retaining group defaults.
- [ ] Preserve each target's actual host parameter gestures, automation and state/save notifications; never pretend direct parameter mutation notified another component's host.
- [ ] Make shared label/settings edits mark project state dirty through supported host APIs where available.
- [ ] Rebuild effective envelopes from retained analysis when sound changes without unnecessary detector retraining, with invalidation/cancellation and original fallback.
- [ ] Test conflicting state loads, target removal and actual host save/automation behavior.

No new product interview is required for these already-confirmed defaults/overrides. Native API limitations must be reported truthfully.
