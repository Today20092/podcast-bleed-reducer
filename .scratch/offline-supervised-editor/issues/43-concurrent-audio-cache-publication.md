# Accept a verified concurrent audio-cache publisher

Status: done
Type: bug
Assignee: Codex
Blocked by: none

## Scope and reproduction

Alpha.4 Linux run 38362754135 fails prepared_analysis cancellation regression
with Directory not empty during its first successful preparation. Identical
audio fixtures share a fingerprint-derived cache destination. Publication
checks existence before renaming staging, leaving a race for another publisher.
Preserve validation, cancellation, source identity and recoverable invalid contents.

## Acceptance

- Deterministic regression at the cache publication seam.
- If a concurrent publisher wins, accept only fully verified matching contents.
- Propagate errors when no matching valid artifact exists.
- Original parallel native suites and all-platform packages pass.

## Local evidence

The deterministic publish_staging regression failed before the fix in 0.02 seconds
with Windows os error 145 (directory not empty), matching Linux's os error 39.
After validating a winning artifact on rename failure, the same test passes in
0.02 seconds. It also corrupts the winner and verifies publication still fails.
Full native CI and candidate alpha.5 packaging remain pending.

Final verification: run 38366852428 passed native tests, package integrity checks
and artifact upload on Windows x64, Linux x64 and both Mac architectures. All four
downloaded packages matched their SHA-256 files. Published-source candidate is
d5fe272f82c06d43c7121c23a3c13fe1a574cb57; stable acoustic/device qualification is unchanged.
