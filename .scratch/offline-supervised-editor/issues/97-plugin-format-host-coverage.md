# Resolve VST2 and intended-host integration paths

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 88
Related: 85, 86

## Scope and acceptance

Create a verified format/platform/host matrix and a bounded adapter plan for both products. Use current official documentation through Context7 where available and primary vendor/SDK sources where not. Do not infer host compatibility from a build.

Acceptance:
- [x] Record installed OBS, FL Studio and Resolve/Fairlight versions where present, supported formats and actual integration/routing/bitness requirements; unknown/uninstalled hosts stay unknown.
- [x] Determine a legal supported VST2 build/distribution route and current adapter capability before adding dependencies or SDKs. If unavailable, document the blocker and an explicit alternative without claiming VST2 implemented.
- [x] Clarify whether OBS requires VST2, an additional host component or a native OBS integration; prototype only a reversible, minimal path under the guarded budget if prerequisites exist.
- [x] Define Windows-first native tests, then macOS/Linux cross/native package gates, minimum versions, mono/stereo and bridge/process-isolation constraints. Keep AAX/AU deferred.
- [x] Produce docs/plugin-host-matrix.md with supported/tested/planned/blocked columns, evidence dates/links and separate product status; add narrowly scoped adapter tickets if this research resolves their SDK/license questions.
- [x] No installation/SDK acceptance/system reconfiguration or vendor activation is needed merely to complete this research; do not download unofficial SDKs.

## Results

Claimed 2026-10-26 after ticket 88 completion. Research uses current primary documentation and read-only installed-version evidence; no host launch, SDK/license acceptance, installation or system configuration change is part of this pass. Unknown compatibility and unavailable VST2 rights remain explicit blockers.

Completed 2026-10-26: [matrix and adapter plan](../../../docs/plugin-host-matrix.md) records OBS 32.2.2, FL 26.1.1.5547 and Resolve 21.1.1.10 from exact installed executables plus registry metadata; edition/configuration remains unknown. Context7 official OBS/Steinberg/Image-Line documentation and primary vendor/SDK sources distinguish host support, old probe evidence and separate product readiness. Exact retained-source/lockfile receipts confirm nice-plug 0.4.2 exports VST3/CLAP mono/stereo, no VST2/native OBS adapter, and a same-process PCM bus.

No supported legal VST2 distribution route is established; current VST3 MIT licensing does not establish VST2 rights. Native OBS filter is the explicit direct-host alternative, with module/dependency licensing and causal Gate prerequisites still open under 85. No new ready adapter ticket is justified. External routing/additional host components remain unverified alternatives. Resolve Linux third-party formats and all proposed minimum versions remain unqualified.

Scoped checks: reviewed supported/tested/planned/blocked matrix for both products, local Markdown link existence and git diff whitespace. Documentation-only changes require no Cargo build/full matrix. Read-only memory snapshot: 31.92 GiB total / 6.28 GiB available, below 88's 14 GiB heavy start requirement; prototype skipped for memory and missing causal DSP/adapter prerequisites. No runtime/peak-memory/latency claim, SDK download, installation, native host launch, activation, link change, push or release. Native/listening/package checks are specified future gates, not passed acceptance results.

