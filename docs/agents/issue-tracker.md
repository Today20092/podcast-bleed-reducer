# Issue tracker: Local Markdown

Issues and specs for this repo live as markdown files in `.scratch/`.

Public bug reports, design feedback and feature proposals enter through [GitHub Issues](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose). GitHub is the public intake and discussion channel; accepted specifications and implementation tickets remain in this local Markdown tracker. Link the public issue from the accepted ticket and the resulting PR. See [contributing](../../CONTRIBUTING.md) for the newcomer workflow.

The sole active roadmap is `.scratch/offline-supervised-migration/map.md`. Its resolved architecture decisions lead to implementation issues in `.scratch/offline-supervised-editor/issues/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The spec is `.scratch/<feature-slug>/spec.md`
- Implementation issues are one file per ticket at `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from `01` — never a single combined tickets file
- Triage state is recorded as a `Status:` line near the top of each issue file (see `triage-labels.md` for the role strings)
- Comments and conversation history append to the bottom of the file under a `## Comments` heading
- Implementation readiness uses the canonical triage labels. Claim an unblocked ticket with `Status: claimed` and an assignee before work; use `Status: done` only after its scoped acceptance checks pass. `Blocked by: NN, NN` refers to sibling implementation tickets and requires all to be `done`. Readiness alone does not clear dependencies.
- Consolidation closes a duplicate with `Status: superseded` and `Consolidated into: NN` (or a relative path to the continuing checklist). Preserve its specification/results and transfer every unmet check to the named open ticket or recurring procedure. Superseded does not mean its checks passed; update active dependencies explicitly rather than treating superseded as done.
- `Status: deferred` parks an unapproved future proposal outside the active queue. Retain its scope and reopening conditions; deferred does not mean implemented or rejected.

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<feature-slug>/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or the issue number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` — the Notes / Decisions-so-far / Fog body.
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every file it lists is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append a context pointer (gist + link) to the map's Decisions-so-far in `map.md`.
