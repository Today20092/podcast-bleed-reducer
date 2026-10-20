# Make the project understandable to users and contributors

Status: done
Assignee: Codex
Type: task
Labels: ready-for-agent
Blocked by: none
Spec: ../../offline-supervised-migration/spec.md

## Scope

Owner approved on 2026-10-11 after a grilling interview: focus on podcasters, keep the personal origin story, require already synchronized tracks, and describe supervised bleed reduction with listening and manual gain edits. Add a public roadmap distinguishing current work from possibilities, a contributing guide recommending optional Matt Pocock AI skills, and GitHub bug, design-feedback and feature forms. Reconcile contributor entry links and the public-intake/local-ticket distinction. Automatic synchronization and in-app reporting are future ideas only.

## Acceptance

- README links to contributor guidance, roadmap and public issue intake and includes the approved origin story.
- Contributor guidance covers human and AI contributions, skills provenance, setup, ticket dependencies, checks and PRs.
- Roadmap preserves the accepted specification and quality gates; speculative work has no delivery promise.
- Three issue forms ask for actionable evidence without requiring private recordings or diagnostics; blank issues remain available.
- Local links, heading anchors, YAML structure and scoped diff whitespace pass checks. No app code changes.

## Comments

2026-10-11: Claimed before documentation edits. Existing unrelated application and documentation changes will be preserved.

2026-10-11: Completed README origin story and navigation, CONTRIBUTING.md, ROADMAP.md, contributor entry links and public-intake/local-ticket guidance. Added three GitHub YAML forms plus explicit blank-issue availability. Checked 71 local Markdown links, balanced fences, same-document anchors, three parsed YAML forms with 17 unique fields, required metadata and dropdown choices; scoped git diff --check passed. Verified skill installation guidance against mattpocock/skills and form fields against GitHub's official schema. YAML validation used a temporary uv environment with PyYAML; no repository dependency added. No application tests or rendered GitHub form preview were run. No application code changed, commit or push performed. Forms become available after merge to the default branch. Future synchronization and diagnostic export remain exploratory roadmap entries.
