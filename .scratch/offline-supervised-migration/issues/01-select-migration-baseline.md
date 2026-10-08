# Select and preserve the migration baseline

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Assignee: Codex with repository owner
Parent: ../map.md
Blocked by: none

## Question

Which existing implementation should be the migration baseline, and which experimental ownership changes should be retained, held separately or discarded?

Compare main `730aace`, responsive `135fbb1`, ownership `6a60cc2` and experiments `442b3e9`. Responsive is an ancestor of ownership, but ancestry does not establish quality acceptance. Inspect existing guided-ownership tickets and evidence, preserve root untracked records, and identify source/project compatibility obligations. Record the chosen commit, reusable modules and exclusions before any integration.

## Comments

2026-10-08: The owner has no saved CleanSlate projects. A fresh project format is acceptable; backward compatibility with existing project files is not required. Preserve source recordings and repository records regardless.

Repository investigation recommends ownership `6a60cc2bfe187ea889fd2d8bcc3344311cfd706b` as a development baseline. It contains responsive `135fbb155ef49672e581030d9d92a9c664821056` plus calibration UI/persistence, cached microphone comparisons, render guards and an exact-unity protection fix. The responsive commit remains available as a fallback. Root main `730aace6ea63d508648d2e50a7a38aaa49fadaae` is the scaffold and is not the implemented editor.

Recorded integration checks report 82 passing and five ignored tests, formatting and strict Clippy. These checks were not rerun during planning. Existing quality reports retain an unresolved screenshot bleed failure, an unmet exact-unity criterion for unprotected wanted speech, and no independent unseen holdout or listening sign-off. Historical evaluation data was consulted during later tuning. Reusing the branch does not accept its detector quality.

Recommended exclusions: the rejected spectral-leakage/evidence-aware-padding experiments at `442b3e9556f71957149ae327184ae0fdb156f00b`, discarded short-seed suppression, and discarded source-disqualification attempts. Preserve their reports as provenance. Preserve import validation, source fingerprints, cancellation and obsolete-result guards, exact-unity protections, cached expensive analysis, and agreement between preview, audition and render.

Evidence: `.scratch/guided-ownership/evidence/integration-checks.md`, `quality-results.md`, `performance-results.md`, `.scratch/guided-ownership/spec.md`, `interfaces.md`, and `docs/guided-calibration.md` in the ownership worktree.

## Answer

2026-10-08: The owner confirmed the recommended starting point. Use ownership commit `6a60cc2bfe187ea889fd2d8bcc3344311cfd706b` as the development migration baseline, retain Iced, use a fresh project format, and keep rejected experiments separate. No old saved projects require conversion. Preserve all source audio, repository records and research provenance. Retain responsive `135fbb155ef49672e581030d9d92a9c664821056` as a recoverable fallback.

Reuse validated imports, waveforms/timeline, audition, retained analysis, calibration mechanics, project source identity checks, protections, safe rendering/export and quality tooling. Ownership inference is replaceable development code whose promotion requires new independent acceptance evidence. Do not start from scaffold main or include the rejected experiment branch in the production baseline. Actual checkout/integration is implementation work after planning.
