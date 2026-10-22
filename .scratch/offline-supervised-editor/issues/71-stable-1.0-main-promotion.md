# Prepare stable 1.0.0 and promote development to main

Status: claimed
Type: release
Assignee: Codex
Blocked by: none
Related: 60, 63, 68, 69

## Owner request — 2026-10-22

The owner accepts development and requests promotion into main as the first stable release under the documented versioning policy. Update version metadata, README/help/release documentation and release automation to agree on 1.0.0. Include the completed menu/action/toolbar batch and accepted release policy. Ongoing research 70 remains outside the release.

## Acceptance

- [ ] Align Cargo metadata, package defaults and stable-aware draft release automation; retain prerelease support and package safety checks.
- [ ] Update current status/help/README/release notes truthfully, preserving historical alpha records, current public downloads and known limitations.
- [ ] Run local checks and standard GUI build/self-check; record exact source and executable identity. The owner approved the appearance and requested no more computer control.
- [ ] Promote development through a reviewed/check-passing PR to main using a merge commit; synchronize main back into development and retain main as GitHub's default branch.
- [ ] Record native package verification/publication status accurately. A main merge does not itself publish packages; retain existing downloads until verified replacements are public.

## Authorization and limits

This request authorizes stable release preparation and main promotion. No new algorithm, plugin, source alignment or unrelated research implementation is included. The compatibility promise and physical-platform/quality limits in docs/releasing.md remain authoritative. No new listening or hardware evidence is implied by the stable label.

## Publication authorization — 2026-10-22

The owner explicitly requested publishing stable 1.0.0 with all download packages and merging the generated README download update after checks pass. The annotated `v1.0.0` tag identifies reviewed main `68198b3703a6fb4ec0aef53b9712acf44a87cacd`. The tagged package workflow is run 39819315222. The earlier build-only preparation run 39816454872 passed all four native packages; its application source tree is identical to the tagged merge commit. Publication will use the final tagged-run assets, with checksums and artifact identity verified before making them public.
