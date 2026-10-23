# Optional purchase delivery — ticket 72

2026-10-23. Optional purchase stage implemented on `development`; the complete application remains free. This is a development build, not a publication or completion of ticket 72's later savings stage.

## Delivered behavior

- Purchase CleanSlate appears on first launch with Continue focused, optional-payment disclosure and Don't show again selected. Continue/Escape saves the choice through the existing atomic preference writer. Explicitly unchecking the option requests the message on later launches. Invalid/malformed preferences are preserved; saving failures leave the editor usable with actionable feedback.
- Menu and About reopen the dialog. Registered, optionally remappable commands own the support and provider/roadmap actions. Keyboard selection and button dispatch use the same actions; command dispatch now returns asynchronous tasks so browser completion/errors reach the UI.
- Ko-fi `https://ko-fi.com/ayoubab1` is primary. GitHub Sponsors `https://github.com/sponsors/Today20092` is secondary, with one-time/monthly wording. Owner-supplied Ko-fi, existing `.github/FUNDING.yml`, GUI and README destinations agree. The public Sponsors page was checked; Ko-fi could not be retrieved by the web tool and checkout was not exercised.
- Copy uses the owner's requested purchase wording, explains intended maintenance/usability/testing/package funding and links the public roadmap. Full use remains free; no payment processing, activation, telemetry, price, future-feature promise or numeric savings claim was added. README, workflow help, GUI guide and public/local roadmaps distinguish development work from published 1.0.0.
- Preference loading now treats an absent appearance entry as its default, preventing a purchase-only preference file from producing a false theme warning. Existing unrelated JSON entries and malformed files remain preserved.

See the [FUTO research and implications](futo-payment-model-2026-10-23.md) for the evidence and its limitations.

## Verification and build identity

`scripts/check.ps1 -Local` passed on the final source: **359 tests passed, 20 ignored**, strict all-target/all-feature Clippy, formatting and local build/workflow/package/release/action-contract fixtures. Ignored hardware/benchmark checks were not run. Three added tests cover preference defaults/persistence/corruption/save failure, dialog work preservation/browser errors and keyboard navigation/dismissal. Existing UI contrast and command regression checks also passed. Tests construct both theme views; this is not a rendered visual inspection.

`scripts/gui-dev.ps1 -BuildOnly` passed (490.97 seconds reported, including time waiting for the shared Cargo target lock). The resulting executable's `--package-self-check` passed offline inference, analysis, project save/reopen, rendering and cancellation. No owner recording or saved project was used for this check.

- Checkout: `C:/Users/User/Documents/podcast-automix-rust-gui`, branch `development`.
- Source base: `d1c5b358a86c2b514c89f54534953424a69f0796` **plus uncommitted working-tree changes**. The former checkout path remains a junction to this location; final checks/build ran from the new path after tool-session handles were lost during relocation.
- Executable: `C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe`.
- Executable size: 59,007,488 bytes.
- Executable SHA-256: `e878b02aaf9db304c8d371f97cacb2e84a2224ef1cf278354715a3b2a0f873c0`.
- `src/app.rs` SHA-256: `b3c4162cd12b3fa65b8442edc204530d5b82be23c647021fe0a4a87a64aa44d9`.
- `src/appearance.rs` SHA-256: `e3a53bfa60ca20420be2f7bd1d2b72a8418712381854a3f5cf307ccb9a534f03`.
- `src/shortcuts.rs` SHA-256: `a795aff560302fad8fc741d6d04b868aaa51c053cbc5a251857ada49fab24e3e`.

Support destination consistency and local links in all three new research reports passed. `git diff --check` passed. Pre-existing Auto-Align research/ticket files and earlier roadmap edits were retained.

## Remaining acceptance

No native compact/wide or Light/Dark visual inspection, browser checkout transaction, physical keyboard/listening check or rendered README review was performed. The GUI was built without launching it. Do not infer those checks passed from view construction or the headless self-check.

Ticket 72 remains claimed: its savings baseline/formula, counting/deduplication/reset rules and eventual counters/estimates are unfinished. Separate plugin/updater discovery is outside this delivery. No release was published.

## Owner acceptance and development commit

The owner subsequently requested a GUI preview. The standard GUI script rebuilt in 1.11 seconds, launched a separate development project copy, and the application responded under the development window title. The owner reviewed the appearance, said it looks good, and requested committing this purchase stage to development. This accepts the shown appearance without claiming every compact/wide/theme/keyboard/checkout combination was separately exercised. The saved source project was not used as the editable development copy. The source hashes above still match; no application code changed after verification.
