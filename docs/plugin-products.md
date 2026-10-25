# CleanSlate products and plugin workspaces

Accepted by the owner on 2026-10-25. This separates the previous combined Session/Live Mic product direction. The [active map](../.scratch/offline-supervised-migration/map.md) remains the authoritative roadmap.

| Product | Workflow | Development branch | Continuing ticket |
| --- | --- | --- | --- |
| CleanSlate Desktop | Standalone Rust GUI, supervised offline bleed reduction | `development` | Existing desktop queue |
| CleanSlate Smart Gate | Single microphone, live speech gating | `codex/plugin-smart-gate` | [85](../.scratch/offline-supervised-editor/issues/85-smart-gate-plugin.md) |
| CleanSlate Podcast | Supervised multitrack post-production inside a host | `codex/plugin-podcast` | [86](../.scratch/offline-supervised-editor/issues/86-podcast-plugin.md) |

Smart Gate means speech-controlled attenuation; automatic voice-volume leveling is not in its initial scope. Podcast does not need live multitrack recording support. Both are separate products, not modes selected inside one shipping plugin. Product names are working names.

## Code and integration boundaries

Each plugin branch has an independent Git worktree with its own source, working state and eventual product artifacts. The initial snapshots retain the full repository because the prototype depends on its maintained offline library by relative path. They do not require running the desktop GUI. This is a source-control/product separation, not a claim that the engine has already been fully extracted into UI-free crates.

The shared `crates/ownership-model` and offline engine remain maintained Rust code. Make common engine fixes once, verify desktop behavior, integrate their focused commits through `development`, then bring those commits into both plugin branches. Keep product-specific host adapters/UI/state/builds on their product branches. Do not merge one plugin's complete branch into the other or into desktop just to obtain an engine fix. Further core extraction should follow demonstrated dependency needs; do not copy or fork the detector implementation.

The shared ownership-model extraction and desktop library feature boundaries already exist in the working baseline. They stay with desktop because it uses them too. Plugin source and plugin-specific build/link/test scripts move out of the primary desktop checkout only after the preserved snapshot is committed and checked. Shared scripts/fixtures and planning/research records remain in the primary checkout. Unrelated pending desktop/release/research edits remain untouched.

## Baseline and compatibility

Both branches initially contain `plugins/fl-probe` 0.0.6 and its test hosts as historical scaffolding. Their root `PLUGIN-WORKSPACE.md` selects the product ticket and next steps. This does not produce two newly branded binaries. Distinct packages, format IDs, state schemas, build destinations and install links are early implementation acceptance items in 85/86. Never reuse a new product identity to silently replace the old probe in a saved host project.

The installed CleanSlate FL Probe remains the existing checked build. Its Common Files links still point to `C:/Users/User/Documents/podcast-automix-rust-gui/dist/fl-probe`, which is retained. Do not retarget these links or rebuild/reinstall that identity automatically from either product workspace. Existing host projects and cached audio remain untouched. Each future product gets its own checked artifacts and deliberate development links.

Use lightweight unit/actual-format host checks for normal iterations; open native hosts for integration evidence when needed. Preserve the old output before updates. Existing navigation tests are not sound/export or complete product acceptance. Ticket 80 and its dependent 81–83 retain their unfinished checks under Podcast.

## Formats, platforms and hosts

VST3 and CLAP are primary targets, with VST2 retained as desired legacy coverage subject to adapter and licensing/distribution verification. Windows is first; macOS/Linux require native packaging and checks. AAX and AU are deferred by owner preference. These are targets, not a claim that the current adapter implements all of them.

FL Studio is the current tested prototype host. Resolve/Fairlight (Podcast) and OBS (Smart Gate) are intended hosts requiring their own capability/integration checks; do not assume a format build proves those hosts support it. Unsupported bridging/process isolation must be visible and safe. No vendor installation, copied vendor model, or further reverse engineering is needed for this split.

## Workspace receipt

[Overnight planning](plugin-overnight-plan.md) adds scoped agent tickets 88–99, renewed static reference investigation and a mandatory tested memory guard/checkpoint runner before unattended heavy work. Desktop/update residuals have their own audit; no repeated updater implementation or new desktop feature scope is implied.

[Reorganization ticket 87](../.scratch/offline-supervised-editor/issues/87-separate-plugin-products-workspaces.md) records exact local paths, source commits and checks. This plan makes no new release, audio-quality or platform claim.
