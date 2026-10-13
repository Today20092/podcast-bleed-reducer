# Stabilize native render failure-injection synchronization

Status: claimed
Type: bug
Assignee: Codex
Blocked by: none

## Evidence and scope

Intel Mac job 115187023552 in run 38364559921 timed out at tests/render.rs:29
waiting five seconds for a private staged WAV. The fault thread exited without
injecting its failure; run then succeeded and unwrap_err failed. The full render
suite took 9.43 seconds. This is a harness scheduling/preparation timeout, not an
observed export cleanup failure.

Keep all error, cancellation and no-published-output assertions. Increase the
bounded synchronization wait to 30 seconds and sleep one millisecond per poll
instead of continuously consuming CPU with yield_now.

## Acceptance

- Original render suite passes locally and on native CI.
- Every existing failure/cancellation assertion remains.
- Complete all four native package checks before creating the release candidate.
