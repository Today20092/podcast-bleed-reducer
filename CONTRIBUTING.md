# Contributing to CleanSlate

CleanSlate helps podcasters reduce bleed between dedicated microphone tracks. Contributions can be code, documentation, bug reports, listening feedback, UI/UX feedback, or ideas about what the project needs next. You can contribute without using an AI agent.

## Report a problem or suggest a change

Search [existing issues](https://github.com/Today20092/podcast-bleed-reducer/issues) first, then [choose an issue form](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose):

- Bug report: steps to reproduce, expected and actual results, app version and platform.
- Design and user-flow feedback: what you were trying to do, where you got stuck, and what would make the task clearer.
- Feature proposal: the recording problem, desired result and current workaround. Ideas beyond the current focus are welcome, but submission does not mean acceptance.

Listening feedback is especially useful when it identifies wanted speech being reduced, remaining bleed, unnatural fades, or overlap that sounds wrong. Include approximate timestamps, the number of microphones, relevant settings and whether you listened in Original, Processed or Removed mode.

GitHub issues are public. Remove personal paths, recording names and private information from screenshots or logs. Do not post recordings without permission from everyone recorded. Describe the problem without uploading audio first; if a sample is needed, prefer a short recording made specifically to reproduce it. Never post tokens or credentials. There is no in-app diagnostic export yet.

## Choose and agree on work

Small fixes can come directly as pull requests. Features and changes to audio behavior should begin with an issue and an agreed specification before implementation. The [public roadmap](ROADMAP.md) explains the direction; the [accepted specification](.scratch/offline-supervised-migration/spec.md) defines the current behavior.

GitHub Issues collect public reports and discussion. Maintainers turn accepted work into scoped local Markdown tickets in [.scratch/offline-supervised-editor/issues/](.scratch/offline-supervised-editor/issues/). These files are tracked in Git despite the directory name. Link the public issue from its implementation ticket when one exists.

Read [AGENTS.md](AGENTS.md), the [active map](.scratch/offline-supervised-migration/map.md), and the [implementation tracker](.scratch/offline-supervised-migration/implementation.md). Choose an unclaimed ticket whose `Blocked by` dependencies are all `done`. A `ready-for-agent` label alone does not make a blocked ticket available. Claim it with `Status: claimed` and an `Assignee:` before editing. Coordinate in the GitHub issue or PR so another contributor does not start the same work.

Follow the [tracker conventions](docs/agents/issue-tracker.md) and [triage vocabulary](docs/agents/triage-labels.md). Record acceptance results in the ticket; mark it `done` only after its scoped checks pass. Tiny fixes without an existing ticket should record their scope and validation in the PR; agent implementation follows the ticket requirements in AGENTS.md.

## Build and submit a pull request

1. Fork the repository, clone your fork and start feature work from the current `development` branch. Use a scoped branch such as `codex/fix-waveform-help` for agent work. See the [branch and release workflow](docs/branch-workflow.md).
2. Follow the [developer guide](docs/development.md) for builds and checks. GUI work uses the [GUI development guide](docs/gui-development.md) and `scripts/gui-dev.ps1`.
3. Keep the change focused. Before changing domain behavior, read [CONTEXT.md](CONTEXT.md) and [ADR-0006](docs/adr/0006-offline-supervised-editable-envelopes.md). Preserve source validation, cancellation, safe exports and the shared gain-envelope evaluation.
4. Run the ticket's acceptance checks and the relevant development checks. For documentation-only work, check links and formatting. Audio behavior needs meaningful automated checks and listening evidence where the ticket requires it; passing tests alone does not establish audio quality.
5. External contributors open a feature PR against `development`, linking its issue and local ticket. The owner may integrate solo feature branches locally, revise and test the combined development build, then push development as a backup; per-feature PRs are optional in that workflow. Promote a tested batch through one `development` → `main` PR, then deliberately tag a release. Describe the resulting behavior, checks and anything unverified; include screenshots/listening evidence where relevant.

Keep recordings, personal projects, caches and generated builds out of commits. Contributions use the project's [MIT license](LICENSE).

## Optional AI agent skills

### Google Rust Guidelines

For Rust implementation and review, use the personal [Google Rust Guidelines skill](https://github.com/Today20092/skills/tree/main/skills/engineering/google-rust-guidelines), adapted from [Google's Rust style guide](https://google.github.io/styleguide/rustguide.html). The skill includes the complete rule inventory, platform-specific references, source revision and attribution. It is maintained independently and is not endorsed by Google.

Install it for a project using the [Skills CLI](https://github.com/vercel-labs/skills):

```sh
npx skills add Today20092/skills --skill google-rust-guidelines
```

For a personal Codex installation shared across projects:

```sh
npx skills add Today20092/skills --skill google-rust-guidelines --global -a codex
```

Select your agent when prompted. For agents supporting named skill invocation, use `$google-rust-guidelines`; otherwise ask the agent to use Google Rust Guidelines. For example: “Audit this ticket's changed Rust code using Google Rust Guidelines; report applicable findings, exceptions and verification.”

The canonical skill stays in the skills repository; do not copy it into CleanSlate. Installation is optional and not a build dependency. The short pointer in AGENTS.md makes the policy discoverable even without an installed copy. Follow this project's accepted specification and architecture, apply Google platform rules only where relevant, and scope fixes to the ticket. Avoid unrelated style rewrites. The skill's source revision can change independently; record the skill commit in an audit or PR when reproducibility matters.

The initial audit and tickets 104–107 used [skill revision b775ecd](https://github.com/Today20092/skills/tree/b775ecd77d076212b65d0f6c14396760cf74c4e4/skills/engineering/google-rust-guidelines). See the [correction receipt](docs/research/rust-guidelines-corrections-2026-10-28.md) for the actual fixes and checks.

### Specification and review workflow

This project uses [Matt Pocock's agent skills](https://github.com/mattpocock/skills) to clarify requirements, write specifications, split work into tickets, implement and review. The project's configuration lives in [AGENTS.md](AGENTS.md) and [docs/agents/](docs/agents/); the skill files themselves are not bundled in this repository.

For agent-assisted work, install the skills for your agent using the [upstream installation instructions](https://github.com/mattpocock/skills#installation-30-second-setup). An editable-file installation can use:

```sh
npx skills@latest add mattpocock/skills
```

Choose your agent and relevant skills in the installer. Upstream also documents plugin installation for supported agents. Choose one installation method per agent to avoid duplicate skills. Installation is optional and is not part of building CleanSlate.

Use the interview and specification tools before substantial changes, then work from scoped tickets and review the result. Skill names and installation options can change, so use the upstream guide for the current commands. Configure the agent to follow this repository's existing local-ticket conventions rather than replacing the tracker or creating a second roadmap. You remain responsible for understanding and validating any generated changes.
