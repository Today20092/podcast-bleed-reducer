# Fix the Windows alpha 3 native test crash

Status: claimed
Type: bug
Assignee: Codex
Blocked by: none

## Scope

Owner authorized fixing the repeated Windows release test ACCESS_VIOLATION and
completing the previously authorized all-platform build. Preserve every test and
existing release qualification limits. Leave the original workspace edits untouched.

## Reproduction

Release run 38341277070 attempts 1 and 2 crash rust_app library tests with
0xc0000005. macOS arm64/x64 and Linux pass. Concurrent output does not identify
the crashing test. Diagnostic run 38359840556 runs the full suite sequentially
to obtain the active test name; serialization is a probe, not a root-cause claim.

## Acceptance

- Identify the failing test/path or record concrete native-crash evidence.
- Fix the cause without ignoring tests or weakening assertions.
- Run the original Windows release checks successfully.
- Complete native tests and package self-checks for all four platforms.
- Record remaining physical/acoustic qualification limits.
