# FabFilter and sonible UI/performance reference investigation

Status: open
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
- [ ] Deliver docs/research/plugin-reference-ledger.md with source hash/version, REA Evidence IDs or source ranges, observed/inferred/unknown labels and bounded search coverage.
- [ ] Produce separate reusable UI lessons, performance hypotheses with validation experiments, and gate behavior lessons; do not claim recovered proprietary algorithms/models.
- [ ] Mark all target questions answered/partially answered/unresolved, with concrete next actions; unavailable runtime evidence is a recorded limitation, not an invented result.
- [ ] Do not copy vendor assets, branding, code or model weights into CleanSlate. No licensing/activation bypass.
- [ ] Record memory/time and close owned REA sessions; additional payload expansion or source changes must be justified by a specific unanswered question.

## Results

Not started. Readiness describes specification only; all listed dependencies must be done before claiming.

