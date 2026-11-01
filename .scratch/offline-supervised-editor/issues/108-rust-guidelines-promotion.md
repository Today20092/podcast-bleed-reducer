# Promote the Rust guidelines correction batch

Status: done
Type: task
Assignee: Codex
Blocked by: none (104–107 done)

## Scope

The owner authorizes pushing the completed work, creating and shipping a PR, explaining its benefits and documenting how to reuse the Google Rust Guidelines skill. Promote development to main through passing four-platform checks and a merge commit. The promotion includes the existing development backlog; describe that scope explicitly. Preserve separate plugin branches and existing release tags/assets.

Add a short agent instruction pointing to the canonical personal skill and contributor installation/use instructions. Do not vendor the skill or make it a build prerequisite. Public package publication retains ticket 74's final-package and private-signing acceptance; source promotion does not close those checks or ticket 100's remaining native acceptance.

## Acceptance

- [x] Document the canonical skill, optional installation and project-specific precedence; verify links and command discovery.
- [x] Push the reviewed documentation and create a development → main PR with benefits, complete promotion scope and verification limits.
- [x] Confirm all four PR platform checks, merge with ancestry preserved and synchronize development.
- [x] Record PR/source identities and any remaining public-release gates truthfully.

## Verification

AGENTS.md contains the short contextual pointer; CONTRIBUTING.md owns project/global installation and invocation. Canonical revision b775ecd is still the skill repository's main tip and its SKILL.md is accessible through GitHub. `npx --yes skills add Today20092/skills --list` discovers all four personal skills including google-rust-guidelines without installing. CLI help and the official Vercel Skills README confirm named skill, agent and global options; Context7 returned its monthly quota error, so those primary sources were used. `git diff --check` passes. No skill was vendored or installed into this checkout.

## Promotion result — 2026-10-28

[PR 24](https://github.com/Today20092/podcast-bleed-reducer/pull/24) merged the complete existing development backlog and Rust corrections into main with merge commit `f0ea4f02e092d3e8f0c674a5489051f2474a4ba8`. Its exact head `e1919e9a76af5fe50a3e3e60df8e9ed6d0805e1d` passed [run 40635018559](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/40635018559): Windows x64, Linux x64 and both macOS architectures all completed successfully. Windows reports 386 passed / zero failed / 20 existing ignored tests; macOS ARM64 reports 380 / zero / 20. Formatting, strict Clippy and script fixtures pass through the same full check command.

Main and the reviewed head have identical production, test, Cargo, script and CI files. The primary checkout fast-forwarded to the merge, and development was pushed at that merge (recreated after GitHub's branch auto-deletion). This closure record is a subsequent documentation-only development commit.

The PR documents before/after benefits, its wider development scope, the pinned skill and NPX installation/invocation. No tag or release asset was moved or published. The existing v1.1.0 draft still identifies its earlier source; ticket 74 retains actual final-package verification and owner-only private signing. Ticket 100's native GUI acceptance and the fresh local GUI/device/listening limits in the Rust receipt remain open. These gates prevent claiming that this source merge updated the installed app or public downloads.
