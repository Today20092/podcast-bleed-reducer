# Promote the Rust guidelines correction batch

Status: claimed
Type: task
Assignee: Codex
Blocked by: none (104–107 done)

## Scope

The owner authorizes pushing the completed work, creating and shipping a PR, explaining its benefits and documenting how to reuse the Google Rust Guidelines skill. Promote development to main through passing four-platform checks and a merge commit. The promotion includes the existing development backlog; describe that scope explicitly. Preserve separate plugin branches and existing release tags/assets.

Add a short agent instruction pointing to the canonical personal skill and contributor installation/use instructions. Do not vendor the skill or make it a build prerequisite. Public package publication retains ticket 74's final-package and private-signing acceptance; source promotion does not close those checks or ticket 100's remaining native acceptance.

## Acceptance

- [x] Document the canonical skill, optional installation and project-specific precedence; verify links and command discovery.
- [ ] Push the reviewed documentation and create a development → main PR with benefits, complete promotion scope and verification limits.
- [ ] Confirm all four PR platform checks, merge with ancestry preserved and synchronize development.
- [ ] Record PR/source identities and any remaining public-release gates truthfully.

## Verification

AGENTS.md contains the short contextual pointer; CONTRIBUTING.md owns project/global installation and invocation. Canonical revision b775ecd is still the skill repository's main tip and its SKILL.md is accessible through GitHub. `npx --yes skills add Today20092/skills --list` discovers all four personal skills including google-rust-guidelines without installing. CLI help and the official Vercel Skills README confirm named skill, agent and global options; Context7 returned its monthly quota error, so those primary sources were used. `git diff --check` passes. No skill was vendored or installed into this checkout.
