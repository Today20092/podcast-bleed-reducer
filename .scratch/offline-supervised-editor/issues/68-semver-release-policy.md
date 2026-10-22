# Document stable release and semantic versioning policy

Status: done
Type: task
Assignee: Codex
Blocked by: None
Labels: ready-for-agent

## Scope

The owner accepted the recommendation to target CleanSlate 1.0.0 as the first stable release and requested durable versioning guidance using writing-for-agents and agents-md-field-study. Document the policy in the existing release guide, add a conditional AGENTS.md pointer, and reconcile the roadmap and branch workflow. This ticket covers documentation, not version metadata, workflow implementation, tagging or publication.

## Acceptance checks

- [x] Define patch/minor/major selection and the desktop compatibility promise.
- [x] Distinguish the accepted 1.0 target from the currently published alpha and from publication authorization.
- [x] Preserve package/source/safety checks and truthful platform and quality limits; identify current workflow preparation needed for stable publication.
- [x] Keep policy authoritative in the release guide; verify added links, whitespace and the scoped diff.

## Results

Claimed before edits. Existing menu-integration branch and unrelated local changes are retained; no branch switch or integration is part of this task.

Implemented the owner-selected release guidance in docs/releasing.md, with one conditional AGENTS.md pointer and routing/decision updates in the branch workflow and active roadmap/spec/implementation index. Only the root AGENTS.md exists in this checkout. The field-study review found useful existing build/test, architecture, Git/CI and deeper-document pointers; no unrelated instruction expansion or nested file was needed. Generic checklist additions and a whole-repository instruction rewrite were outside the selected scope.

Verified the current workflow's draft/prerelease flag, alpha notes input, build-only distinction and publication-triggered download update against the YAML. Reviewed the scoped diff and confirmed local Markdown file targets (167 references across touched files), the new policy heading/anchor and clean whitespace with git diff --check. Version examples describe the future candidate; manifests, workflow code, public download links and published releases are unchanged. No application tests, builds, package checks or new hardware qualification were run for this documentation-only task. Unrelated ticket 67/map discovery edits and the existing research note are preserved.
