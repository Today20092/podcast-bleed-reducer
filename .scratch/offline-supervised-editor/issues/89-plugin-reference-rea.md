# FabFilter and sonible UI/performance reference investigation

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 88
Related: 85, 86

## Scope and acceptance

Use reverse-engineer-anything SKILL.md and the installed REA tools. Shared research, no product implementation. Start with docs/research/plugin-waveform-ui-2026-10-24.md, sonible-static-pass-2026-10-24.md and smartgate-followup-2026-10-25.md so earlier analysis is reused.

Targets: installed FabFilter Pro-Q 4, Pro-R 2 and Pro-C 3 (primary UI/motion references; Pro-L 2 secondary). sonible smart:gate is the primary gating/workflow reference; smart:EQ 4, smart:comp 3, smart:deess, smart:limit, smart:reverb 2 and smart:chain are secondary UI references in the seven owner-supplied Downloads/To Review folders. See docs/plugin-overnight-plan.md for exact paths. Hash/version each exact artifact and never substitute a similarly named version silently.

Answer a bounded matrix: hierarchy/spacing/type/color; resize/DPI; hover/focus/drag/zoom feedback; meters/waveforms and interaction priorities; motion transitions; paint/invalidation/caching/threading/graphics evidence; inter-instance overview/control (especially Pro-Q 4); smart:gate learning/readiness/confidence/attenuation control separation. Map every useful finding to an original CleanSlate design recommendation.

Static-first: manuals, shipped resources and focused strings/imports; extracted payloads may be inspected without executing installers. No sonible installation/activation is needed or authorized by this ticket. Process one target at a time, export evidence and close it before the next. Do not run broad recursive decompilation or all plugins in parallel. Use focused deep analysis only under 88's enforceable resource contract; otherwise checkpoint that question as unknown. The initial bounded Pro-C 3 resource evidence is recorded in the runbook.

Runtime UI/motion claims require actual observation. Installed FabFilter runtime observation is within the owner's computer-use authorization, but is excluded from the default headless overnight lane; prepare a separate single-host native session after other heavy work has stopped. If static data cannot establish frame pacing, animations, GPU usage or internal scheduler behavior, label it unknown rather than infer from appearance/import names.

Acceptance:
- [x] Deliver docs/research/plugin-reference-ledger.md with source hash/version, REA Evidence IDs or source ranges, observed/inferred/unknown labels and bounded search coverage.
- [x] Produce separate reusable UI lessons, performance hypotheses with validation experiments, and gate behavior lessons; do not claim recovered proprietary algorithms/models.
- [x] Mark all target questions answered/partially answered/unresolved, with concrete next actions; unavailable runtime evidence is a recorded limitation, not an invented result.
- [x] Do not copy vendor assets, branding, code or model weights into CleanSlate. No licensing/activation bypass.
- [x] Record memory/time and close owned REA sessions; additional payload expansion or source changes must be justified by a specific unanswered question.

## Results

Claimed 2026-10-26 after ticket 88 completion. Reuse prior reports and bound fresh static inspection to the supplied artifacts. Shared-provider deep analysis, native hosts, vendor installation and activation remain excluded from this low-memory run. Record unanswered runtime/motion questions explicitly.

Completed 2026-10-26 on `codex/plugin-reference-89`, from development `9d8ce6afee67698fe8ddee695db47f6c043830e7`. The [reference ledger](../../../docs/research/plugin-reference-ledger.md) supplies exact artifact hashes/versions, a ten-target/eight-question status matrix, source ranges and REA IDs, original UI lessons, separate performance experiments and gate behavior lessons. Ticket 90 can proceed from these bounded findings; native runtime gaps remain explicit.

Fresh streamed hashing reconciles 29 files, including all 14 supplied sonible originals and seven extracted VST3 subjects. Sequential import-table inspection covers ten VST3 files (2.77 seconds; PowerShell peak 292.02 MiB). Hash pass took 3.17 seconds with 102.05 MiB PowerShell peak. Seven manuals were text-extracted (1.97 seconds) and overview pages rendered/inspected (2.97 seconds); selected installed CHM topics were read. Shared REA/parser peaks were not instrumented. Available physical RAM was about 4.7–5.2 GiB, below the heavy-start threshold; no deep provider, full build or native host was launched.

Fresh portable C3 resource inspection matches runbook Evidence ID `ev_80be1286dedcbf184051bc4f21a3dec472304011d7c7d7377bbde1fe0eb5d962`; complete record exported to `C:/Users/User/Documents/Codex/Investigations/plugin-reference-89-2026-10-26-c3.json` (218,068 bytes). Earlier same-hash resource/native records were reused rather than regenerated. REA reported no open target and no pending cleanup at the beginning/end; no owned native session was opened. No vendor assets or payloads entered the repo, and no installed target/link/configuration was changed.

Acceptance checks: `C:/Users/User/Documents/Codex/Investigations/plugin-reference-89-check.ps1` checks every one of the 29 report hashes against the fresh inventory, all 20 cited retained Evidence IDs and subject identities, ten import records and local Markdown links; passed. `git diff --check` passed. Documentation-only scope requires no product rebuild/test run. Runtime frame pacing/GPU utilization, focus/DPI behavior, scheduler/model internals and detector quality remain unknown, with concrete bounded follow-ups N1–N4 in the report; none is claimed as passed.

