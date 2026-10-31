# CleanSlate Smart Gate workspace

Branch: `codex/plugin-smart-gate`
Product ticket: [85](.scratch/offline-supervised-editor/issues/85-smart-gate-plugin.md)
First implementation leaf: [91](.scratch/offline-supervised-editor/issues/91-smart-gate-product-identity.md), after 87/88 are done.

Read AGENTS.md, the active map, [product boundaries](docs/plugin-products.md) and [overnight resource plan](docs/plugin-overnight-plan.md) before work. The canonical tracker remains the primary desktop checkout; synchronize its latest planning records before claiming. This workspace contains the preserved 0.0.6 probe as reusable infrastructure, not a finished single-input gate. Do not ship the old paired ownership mode as proof of live-gate quality. Volume leveling is outside the initial gate.

Product-specific code stays on this branch. Shared engine changes need focused integration through development and desktop checks. Do not alter installed probe links, write into the primary checkout's dist, overwrite user projects, or merge the entire plugin branch into desktop. Default to one guarded heavy task globally and no unattended DAW.

Before executing checks, follow the current owner memory override in AGENTS.md:
historical memory thresholds/caps no longer apply. Shared timeout, serialization,
cancellation and cleanup requirements remain; memory accounting is informational.

## Ticket 91 source checkpoint

`plugins/smart-gate` is a separate 0.0.1 pass-through candidate on the temporary
`codex/smart-gate-identity-91` branch, based on Gate integration `3bbda73`.
Planning/design/host-matrix records were selectively synchronized from canonical
development `bb356cb`; shared guard sources already matched that revision.
No whole-development merge was used. Canonical ticket ownership stays in the
primary checkout; copied planning documents are context, not another coordinator.

Use `scripts/build-smart-gate.ps1` and `scripts/test-smart-gate-hosts.ps1` for this
product. They never activate installed links. Locked offline metadata and
noncompiling formatting passed. After the authorized reboot, standard guarded
compilation, tests, strict Clippy and actual CLAP/VST3 state/bypass/coexistence
checks passed on clean source 10180cb. Ticket 91 is done; its candidate and hashes
are recorded in the ticket. Continue with unblocked detector feasibility 92.
