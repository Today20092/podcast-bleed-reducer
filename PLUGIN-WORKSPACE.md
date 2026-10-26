# CleanSlate Podcast workspace

Branch: `codex/plugin-podcast`
Product ticket: [86](.scratch/offline-supervised-editor/issues/86-podcast-plugin.md)
First implementation leaf: [95](.scratch/offline-supervised-editor/issues/95-podcast-product-identity.md), after 87/88 are done.

Ticket 95 source preparation lives on `codex/podcast-identity-95`, based on
`cf1cc5293c5b65379a5c75db72928eabc16db26d`. The new
[`plugins/podcast`](plugins/podcast/README.md) package and
`scripts/build-podcast.ps1` passed guarded Rust/format-host acceptance after the
owner-authorized reboot: 62 tests, strict Clippy, builds and actual CLAP/VST3
hosts on clean source 70f3340. Ticket 95 is done; continue with 96 while keeping
80's native sound/export/PDC/save/listening checks open. Canonical prerequisites/design/host/resource
records were synchronized from desktop `ed6393e`; desktop source was not merged.

Read AGENTS.md, the active map, [product boundaries](docs/plugin-products.md) and [overnight resource plan](docs/plugin-overnight-plan.md) before work. The canonical tracker remains the primary desktop checkout; synchronize its latest planning records before claiming. This workspace contains the preserved 0.0.6 probe, not a finished Podcast-branded plugin. Existing 80/81/82/83 checks remain binding; 84's scoped navigation evidence is retained.

Product-specific code stays on this branch. Shared engine changes need focused integration through development and desktop checks. Do not alter installed probe links, write into the primary checkout's dist, overwrite user projects, or merge the entire plugin branch into desktop. Default to one guarded heavy task globally and no unattended DAW.
