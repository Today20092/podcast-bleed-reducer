# Fix the Windows alpha 3 native test crash

Status: done
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

## Diagnosis and fix

Sequential full-suite run 38359840556 reproduced 0xc0000005 in
app::tests::fresh_original_playback_and_transient_gain_feedback_do_not_require_analysis.
Serialization therefore did not cure the crash.

ready_sources queried CPAL before checking whether cached audio existed, bypassing
launch's headless test rate seam. This made an ordinary cold GUI unit test access
the Windows native enumerator from a short-lived test thread. The lookup now
returns before hardware probing on cache miss and shares preparation_output_rate
with launch. Production still probes the device on the GUI thread; tests use the
existing 48 kHz seam. Source stamps and rate mismatch guards remain.

The cache regression checks empty, valid shared, wrong-rate and changed-source
states. Temporary serial execution was reverted. Fixed run 38361304187 passed
Windows Check and test with the original parallel command; package checks and
remaining platforms are pending at this checkpoint.

Later verification: all four platforms passed Check and test in 38361304187.
Windows, macOS arm64 and Linux package verification passed. Intel Mac packaging
and the final tagged alpha.4 release remain pending.

The failed alpha.3 tag remains unchanged. The corrected candidate is alpha.4.
No physical disconnect/recovery, independent acoustic or long-session qualification
is claimed by this native CI fix.

Final verification: run 38366852428 completed successfully with all four native
test suites and package checks green. Windows passed 264 tests, 18 ignored.
The exact packaged source is d5fe272f82c06d43c7121c23a3c13fe1a574cb57.
Candidate alpha.5 includes the follow-up cache-publication and render harness fixes.
