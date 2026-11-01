# Verify and deliver the working Smart Gate VST3/CLAP build

Status: open
Readiness: needs-triage
Type: task
Blocked by: 115
Related: 85, 93, 94, 114

## Scope and acceptance

Turn the completed 115 backend into an identifiable local development candidate
that actually processes audio in both formats. This is plugin verification and
delivery, not final GUI polish, public release or a new cross-platform campaign.

- [ ] Preserve matching VST3/CLAP candidates and verified runtime prerequisites
  and notices. Record the source commit, artifact hashes and exact executable
  paths; do not infer tested source from a mutable shared Cargo cache.
- [ ] Update the lightweight hosts to discover current SDK parameter metadata
  and validate the contracted controls. Remove stale parameter-count and fixed
  6 dB gain assumptions; read back real Bypass/Depth/activation/readiness when
  diagnosing unity output. A changed consent counter alone cannot prove gating.
- [ ] Demonstrate actual finite attenuation, unity for bypass/zero depth and
  safe unavailable-evidence behavior through each format. Exercise meaningful
  parameter endpoints/automation, state save/reload and the chosen latency/dry
  alignment. A visible editor or successful DLL load is not an audio pass.
- [ ] Use a short representative podcast excerpt to check quiet word starts,
  endings, pauses, breaths/laughter and control-change clicks. Record what was
  actually listened to; do not infer a quality win from numerical tests.
- [ ] Verify editor open/close and basic control wiring against this same
  candidate, then coordinate installation/shortcuts with the GUI owner so an
  actively edited preview is not replaced unexpectedly. Preserve rollback.
- [ ] Publish a concise development handoff: exact installed candidate, how to
  enable/use it, passing checks and known limitations. Probe/Podcast installs
  and public release state remain unchanged.

Keep checks narrow and relevant to changed behavior. Reuse 93's still-valid
foundation evidence; broaden only for a failure or a demonstrated new risk.

## Comments

2026-10-31: Draft only. This ticket verifies new controls after 115; it does not
replace 93's existing foundational audio/lifecycle acceptance. No build, host,
installation change or release publication was started while planning.
