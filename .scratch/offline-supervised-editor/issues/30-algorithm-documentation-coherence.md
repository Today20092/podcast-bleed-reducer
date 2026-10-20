# Align algorithm documentation with the acoustic editor

Status: done
Assignee: Codex
Type: task
Blocked by: none
Spec: ../../offline-supervised-migration/spec.md

## Scope

The owner requested a review of the GitHub README, architecture, design and other documentation against the current code, and a readable explanation of speech detection, ownership inputs and logistic regression training.

Use the existing ownership guide as the reader entry point. Correct current technical contracts and label superseded design and measurement records as historical. Preserve unrelated checkout changes and existing historical evidence. No application behavior changes.

## Acceptance

- Review every public Markdown document and the diagram text, comparing the local checkout with GitHub main.
- Trace the current editor through fixed VAD, acoustic features, reference sampling, per-track training, temporal decisions and gain composition.
- Explain the operation order, actual inputs, learned weights, positive/negative labels, overlap, human truth and limits in the ownership guide.
- Remove current requirements for Nemotron and unfinished integration from technical guides; retain clearly labeled historical evidence.
- Check changed Markdown links, code constants and whitespace. Record review scope and remaining limits here.

## Results

Reviewed the 27 public Markdown documents in the local repository and compared them with GitHub main at `b079f5f0ef35221aee947d537ddec75d7430d15f`. Inspected all ten documentation diagram text equivalents and checked matching HTML/SVG labels. GitHub metadata confirmed that this repository has no wiki or configured external homepage. Reviewed the published `v0.1.0-alpha.2` release description and its eight application/checksum assets.

The reader entry point remains `docs/ownership-explained.md`, now linked as **How it works** in the README's top navigation and a short README overview. It explains fixed speech detection versus local ownership, calculation order, all feature inputs, label-to-target conversion, normalization, learned coefficients/intercept, sigmoid, regularization, overlap, exact human truth and gain composition.

Corrected the technical model recipe, acoustic cache identity, temporal branch and architecture decision. Removed current Nemotron requirements and unfinished-integration claims. Clarified that intermediate acoustic estimates retain their prior state, while missing/invalid evidence receives protection. The editor disables the temporal gap guard and applies the live gap control during gain generation. Historical design, owner-candidate and required-native capacity records remain clearly marked as historical. Glossary wording now matches current gain shading and the removed review interface.

The published alpha and current code also exposed stale separate-mono-only export wording. Updated the README, usage guide, public roadmap and implementation status for separate or multichannel delivery, encoding choices and native-grid/BWF compatibility limits. Restored the already-published README alpha download links from GitHub rather than carrying forward the local unpublished placeholders.

## Audit inventory

- Entry points and contracts: `README.md`, `CONTEXT.md`, `ROADMAP.md`, `CONTRIBUTING.md`, `AGENTS.md`.
- User and algorithm guides: `docs/ownership-explained.md`, `docs/editor-workflow.md`, `docs/acoustic-evidence.md`, `docs/supervised-ownership-models.md`, `docs/temporal-ownership.md`, `docs/effective-envelopes.md`.
- Architecture and records: `docs/adr/0006-offline-supervised-editable-envelopes.md`, `docs/owner-verification.md`, `docs/session-retro-2026-10-09.md`, `docs/diagrams/README.md`, and the ten diagram pairs.
- Development and release: `docs/development.md`, `docs/gui-development.md`, `docs/releasing.md`, `docs/release-evaluation.md`, `docs/alpha-release-notes.md`.
- Supporting references: `docs/agents/domain.md`, `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md`, `docs/research/nemotron.md`, `docs/research/rust-agent-tooling.md`, `assets/README.md`, `tests/fixtures/import/README.md`.
- Active target/history: migration map, accepted specification and implementation sequence; relevant implementation tickets, including completed export ticket 28.

## Verification

- Current code traced through editor worker, reference readiness, Silero frames, acoustic feature generation, label sampling, logistic training/validation/prediction, temporal optimization, exact reference constraints, automatic gain and export settings/delivery.
- 11 assertions passed for model/frame constants, feature recipe/order, thresholds, initial state, disabled temporal gap guard, gain gap default, benchmark phase and reader navigation.
- Three export source/documentation assertions passed.
- All ten HTML/SVG diagram text pairs agree.
- 240 local file/heading references across 30 Markdown documents passed. This includes the 27 public documents, this ticket and the modified specification/implementation tracker.
- `git diff --check` passed.

## Remaining limits

Documentation only: no application code changed and no Rust suite, GUI rendering or audio/listening experiment run. Source assertions verify the documented contract, not detection accuracy. Independent release quality and long-session gates remain pending. Changes are local and have not been committed or pushed. Existing unrelated checkout changes were preserved; historical records were not rewritten into new measurement claims.
