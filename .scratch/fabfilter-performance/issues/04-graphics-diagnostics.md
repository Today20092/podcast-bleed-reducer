# 04: Report the actual graphics environment safely

Status: ready-for-agent
Type: task
Blocked by: None (can start immediately)
Source proposal: P7

**What to build:** Expose useful graphics backend/adapter and available display scaling/timing information through the established diagnostic or version-information flow so machine-specific rendering issues can be reproduced.

## Acceptance criteria

- [ ] Fetch current documentation for the installed GUI stack and report observed active backend/adapter rather than assuming the configured default is active.
- [ ] Reuse the established diagnostic flow; add no new renderer, fallback setting, dependency upgrade, network collection or automatic publication.
- [ ] Handle unsupported/unavailable graphics/display fields explicitly and keep copied reports free of project/audio paths and recording contents.
- [ ] Verify the public diagnostic/report boundary and perform a current-machine native capture when available; document which fields cannot be obtained.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

