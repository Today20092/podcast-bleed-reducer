# Refresh alpha.7 release notes and measured performance summary

Status: done
Type: task
Assignee: Codex release documentation agent
Blocked by: None
Labels: ready-for-agent

## Scope and acceptance

The owner requested a release of the accepted development application with current documentation and a summary of new functionality and performance changes. Prepare the alpha.7 release notes and a concise evidence-based performance summary. Review the complete development history since the published alpha.6 source, distinguish current behavior from future tickets, and retain experimental/platform limits.

- [x] Cover delivered user-visible behavior since alpha.6, with links to authoritative usage and performance documentation.
- [x] Tie numerical performance claims to source, workload, measured before/after and limitations; distinguish software geometry and observation from physical latency or whole-app FPS.
- [x] Record conditional no-change decisions, cache reuse, audio lifetime/restart fixes and shared streaming waveform work without inventing throughput gains.
- [x] Verify local links and whitespace; apply the unslop wording pass. Do not claim release/package checks that the parent release task has not completed.

## Results

Claimed before edits in isolated codex/release-notes, based on development 6160da9. Parent ticket 64 owns version metadata, release/package checks, promotion and publication. This documentation work does not authorize or perform application changes.

Prepared alpha.7 notes covering the complete development batch since published alpha.6, retaining alpha.6's original notes under a clearly historical heading. Added a concise performance evidence summary with retained before/after values, workload/source identities, conditional no-change decisions and explicit physical/wide-canvas limits. The alpha.7 package-verification paragraph remains intentionally pending parent release evidence; no hosted/native package result is invented.

Verification passed: eight local Markdown links resolve; all numerical claim literals were checked against cited reports; git diff --check is clean. Applied the unslop wording pass to new prose. No application, benchmark, release package, network link rendering or hardware test was run by this documentation task. Parent 64 performs release integration and final evidence updates.
