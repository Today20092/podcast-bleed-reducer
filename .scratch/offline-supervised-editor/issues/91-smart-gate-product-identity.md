# Separate Smart Gate package and state identity

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 87, 88
Related: 85, 86

## Scope and acceptance

In codex/plugin-smart-gate, establish the separate CleanSlate Smart Gate package/artifact identity and focused editor entry point using the existing adapter patterns. Retain the probe source/state for reopening old projects. Do not enable the legacy paired ownership DSP under a new single-mic name.

Acceptance:
- [ ] Separate package/crate, product name, unique CLAP ID and VST3 class identity, state version and product-specific build/test/dist/link paths; deterministic IDs documented.
- [ ] Gate and probe coexist in the actual-format test hosts; probe IDs/state remain unchanged.
- [ ] Product output is pass-through until 93 is qualified, visibly marked as such; no Session group/capture prerequisite.
- [ ] Extract/reuse only demonstrated common host/editor helpers without copying the ownership trainer or moving unrelated desktop UI.
- [ ] Build/check with 88, preserve old artifacts, and verify locked dependency graph, format enumeration, state round trip and bypass.
- [ ] Package names/signing/version maturity follow docs/releasing.md; no release or installed-link change in this ticket.

## Results

Claimed 2026-10-26 after 87/88 completion. Implement only the separate pass-through product identity in the Smart Gate lane, preserving probe source/state and installed artifacts. Current available memory remains below the default 14 GiB heavy-work requirement; code and locked metadata may progress, but build/format/state acceptance stays pending until guarded checks can actually run. Do not mark done on static checks alone or enable legacy paired DSP.

