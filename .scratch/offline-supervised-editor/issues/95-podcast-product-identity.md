# Separate Podcast package and probe-compatible baseline

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 87, 88
Related: 85, 86

## Scope and acceptance

In codex/plugin-podcast, introduce the CleanSlate Podcast package, product-specific editor entry point, IDs/state and build/test/dist/link destinations around the existing Session implementation. Preserve maintained offline analysis and persistent source-time caches. Do not add the live gate as a product mode.

Acceptance:
- [ ] Unique package/name/CLAP/VST3 identities and documented state/cache namespace; probe identities unchanged so old FL projects still open.
- [ ] Old state is not silently treated as the new product's state; any import/migration is explicit, source-bound and reversible.
- [ ] Existing Session unit/event tests and actual-format host checks run against the new artifact; baseline waveform/label/history behavior survives.
- [ ] No detector copy or dependency on a running desktop GUI. Document remaining UI-bearing root library dependencies for later focused extraction.
- [ ] Product-specific build scripts cannot overwrite Gate/probe output or change the existing Common Files links. Check with 88; installer/development-link activation is a separate deliberate step.

## Results

Claimed 2026-10-26 after 87/88 completion. Prepare the separate Podcast identity in its isolated product lane while Gate source preparation proceeds; local process jobs remain serialized under the shared guard. Preserve the legacy probe and maintained Session engine. Build/event/format acceptance remains pending whenever the default 14 GiB heavy-work start requirement refuses execution; static checks alone cannot close this ticket.

