# Resource-conscious overnight development plan

2026-10-30 owner override: all RAM/commit thresholds, running reserves, memory
caps and memory-recovery waiting are removed after the upgrade to 40 GB RAM.
See [111](../.scratch/offline-supervised-editor/issues/111-remove-memory-limits.md)
and the current [execution wrapper](resource-guard.md). Older observations and
refusal records below are historical, not current launch requirements.

2026-10-26 implementation update: [88](../.scratch/offline-supervised-editor/issues/88-memory-bounded-execution.md) is complete, including desktop/plugin entrypoint integration, disposable process tests and representative guarded checks of preserved binaries. [99](../.scratch/offline-supervised-editor/issues/99-overnight-checkpoint-runner.md) supplies the [explicit sequential command coordinator](overnight-runner.md), atomic claims/checkpoints and bounded disposable restart/failure tests. Heavy builds refused safely below 14 GiB available; none were launched. No real overnight queue or schedule was started. Historical planning statements below do not describe a running coordinator.

Prepared 2026-10-25; no overnight run is scheduled or launched by this document. Product split: [85 Smart Gate](../.scratch/offline-supervised-editor/issues/85-smart-gate-plugin.md), [86 Podcast](../.scratch/offline-supervised-editor/issues/86-podcast-plugin.md), [87 workspace organization](../.scratch/offline-supervised-editor/issues/87-separate-plugin-products-workspaces.md). The [active map](../.scratch/offline-supervised-migration/map.md) remains authoritative. This is a dependency queue, not a promise that every product will be complete in one night.

## Workspaces and ownership

| Lane | Branch | Local workspace |
| --- | --- | --- |
| Desktop / shared infrastructure / canonical tickets | development | C:/Users/User/Documents/podcast-automix-rust-gui |
| Podcast | codex/plugin-podcast | C:/Users/User/.codex/worktrees/plugin-podcast/podcast-automix-rust-gui |
| Smart Gate | codex/plugin-smart-gate | C:/Users/User/.codex/worktrees/plugin-smart-gate/podcast-automix-rust-gui |

One ticket at a time is the default even though the workspaces are separate. Worktree isolation does not isolate RAM or the shared Cargo cache. Canonical tracker changes are made in the desktop checkout and copied/integrated deliberately; each product workspace must checkpoint a matching plan revision before executing it. Product-specific implementation commits stay on their branches. Preserve unrelated dirty primary files; do not stage/commit them as a side effect.

## Resource contract before unattended execution

[88](../.scratch/offline-supervised-editor/issues/88-memory-bounded-execution.md) implements the resource guard; [99](../.scratch/offline-supervised-editor/issues/99-overnight-checkpoint-runner.md) implements the resumable command runner. Their scoped disposable tests pass. A reviewed execution plan and explicit start select actual work; tickets requiring implementation/review remain with the active agent. Neither tool resumes a lost host chat or schedules itself.

- One global heavy process tree: Rust compile/link/test, deep REA, model profiling or a native host. No overlapping heavy tasks across branches. Default agent concurrency is one.
- Current build scripts already use Cargo -j1, the shared application-data target, incremental builds and 16 root codegen units. Reuse them; do not wipe caches, run release/LTO by default or run redundant full matrices.
- Tasks keep a 30-minute timeout and one-second accounting. There is no memory launch threshold, running reserve or memory cap.
- A bounded input file size does not cap tool RAM. A shared REA MCP session is not automatically contained by a shell watchdog. If exact provider ownership/limits/cancellation cannot be enforced, skip deep analysis overnight; use manuals/resources/focused static evidence and checkpoint unknowns.
- On timeout, retain logs/last-good outputs and stop only owned work. At most one retry, then blocked. Never close unrelated applications to reclaim memory.
- Native FabFilter/FL/OBS/Resolve interaction, subjective listening and private release signing are separate attended/integration lanes. Do not leave a DAW running overnight as the default test harness.

Historical planning-time observation: 31.9 GiB visible RAM and 18.0 GiB available. The owner reports 40 GB installed after the upgrade; neither value is a launch gate.

## Dependency queue

[Machine-readable queue](../.scratch/plugin-products/overnight-queue.json) preserves leaf IDs, workspace lanes and dependencies. Never execute coordination umbrellas 85/86 as a single unlimited ticket.

| Ticket | Deliverable | Depends on |
| --- | --- | --- |
| 87 | Preserve baseline and separate workspaces | none |
| 88 | Execution coordination / owned-process cleanup; memory policy superseded by 111 | 87 |
| 99 | Resumable coordinator and dry run | 88 |
| 89 | REA UI/performance/gate reference ledger | 88 |
| 90 | Separate original UI designs and interaction contract | 89 |
| 97 | Format/host matrix and VST2/OBS feasibility | 88 |
| 98 | Desktop/updater residual-work audit | 88 |
| 95 | Podcast identity/package/state/build split | 87, 88 |
| 91 | Smart Gate identity/package/state/build split | 87, 88 |
| 92 | Causal single-input detector decision and harness | 91 |
| 93 | Gate DSP and actual-format processing checks | 92 |
| 94 | Focused Gate UI | 90, 93 |
| 96 | Podcast workflow and machine acceptance evidence | 90, 95 |

Use existing 80 for Podcast listening/native acceptance, then 81 shared sound, 82 cache reuse and 83 recovery/metadata after 80 is actually done. Research 89 may finish with explicitly unresolved runtime questions; UI design can proceed from established findings with stated assumptions. Detector 92 cannot unlock DSP by merely reporting that no viable detector was found.

A blocked chain does not stop independent tickets. Do not clear a dependency by changing its label or calling it historical. Existing 75/76/78 obligations must be reconciled rather than lost; 98 records their disposition.

## Reference sources

Primary installed FabFilter targets, confirmed present:
- C:/Program Files/Common Files/VST3/FabFilter Pro-Q 4.vst3
- C:/Program Files/Common Files/VST3/FabFilter Pro-R 2.vst3
- C:/Program Files/Common Files/VST3/FabFilter Pro-C 3.vst3
- C:/Program Files/FabFilter contains corresponding installed CLAP/DLL/help files; Pro-L 2 is a secondary reference already investigated.

Confirmed sonible folders, each containing the named installer and manual:
- C:/Users/User/Downloads/To Review/sonible_smarteq4_win_1.1.1
- C:/Users/User/Downloads/To Review/sonible smartcomp3 win 1.0.2
- C:/Users/User/Downloads/To Review/sonible smartdeess win 1.0.2
- C:/Users/User/Downloads/To Review/sonible smartgate win 1.0.1
- C:/Users/User/Downloads/To Review/sonible smartlimit win 1.2.0
- C:/Users/User/Downloads/To Review/sonible smartreverb2 win 1.0.3
- C:/Users/User/Downloads/To Review/sonible smartchain win 1.1.0

Use [reverse-engineer-anything](C:/Users/User/.agents/skills/reverse-engineer-anything/SKILL.md). Reuse the existing FabFilter/sonible reports before reanalysis. Read-only extraction/static inspection is permitted; no sonible installation/activation is part of the overnight plan. New native runtime work needs the actual installed/available target and recorded evidence.

Planning-time bounded REA Pro-C 3 preflight: version 3.0.1.53, 4141056 bytes, SHA256 `134890ede5a2f3f9786ec2009011ac02d6e3d2247a08c286692ad4157bfac42e`. `inspect_pe_resources` used a 64 MiB file bound and 2048-entry bound; complete resource-tree coverage examined 566 entries / 281 resources. Evidence ID `ev_80be1286dedcbf184051bc4f21a3dec472304011d7c7d7377bbde1fe0eb5d962`. No target execution or deep disassembler was started. Static resource payloads are opaque except icon relationships; this establishes neither actual appearance, animation/frame pacing, rendering backend performance nor vendor algorithm behavior. REA session reported no open target.

## Desktop and release barriers

Desktop keeps its existing scope. Updater 73 is implemented; do not build a second updater. Release 74 is still in progress with package verification and the owner's exact private descriptor signature/publication outstanding. 98 can prepare evidence, not access private signing material or publish. Savings-stage 72 remains needs-info; no made-up efficiency claims. Cleanup 22 retains historical automatic-approval denial; do not delete that recovery target or active build caches overnight.

No unattended push/tag/release, installed-link retargeting, vendor installation, destructive cleanup or media/project overwrite. Preserve source identities, cancellation, safe output publication and checked binaries.

## Morning handoff

Every attempted ticket leaves: branch/commit, changed files, acceptance checks and failures, log paths, artifact paths/hashes, measured peak/time, resource-stop reason, incomplete native/listening checks and next action. Successful compile is not product acceptance. Report finished, blocked and pending separately. If any run cannot resume safely after agent/tool interruption, retain its checkpoint and ask for a supported restart rather than claim it kept working.

