# Resolve VST2 and intended-host integration paths

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 88
Related: 85, 86

## Scope and acceptance

Create a verified format/platform/host matrix and a bounded adapter plan for both products. Use current official documentation through Context7 where available and primary vendor/SDK sources where not. Do not infer host compatibility from a build.

Acceptance:
- [ ] Record installed OBS, FL Studio and Resolve/Fairlight versions where present, supported formats and actual integration/routing/bitness requirements; unknown/uninstalled hosts stay unknown.
- [ ] Determine a legal supported VST2 build/distribution route and current adapter capability before adding dependencies or SDKs. If unavailable, document the blocker and an explicit alternative without claiming VST2 implemented.
- [ ] Clarify whether OBS requires VST2, an additional host component or a native OBS integration; prototype only a reversible, minimal path under the guarded budget if prerequisites exist.
- [ ] Define Windows-first native tests, then macOS/Linux cross/native package gates, minimum versions, mono/stereo and bridge/process-isolation constraints. Keep AAX/AU deferred.
- [ ] Produce docs/plugin-host-matrix.md with supported/tested/planned/blocked columns, evidence dates/links and separate product status; add narrowly scoped adapter tickets if this research resolves their SDK/license questions.
- [ ] No installation/SDK acceptance/system reconfiguration or vendor activation is needed merely to complete this research; do not download unofficial SDKs.

## Results

Not started. Readiness describes specification only; all listed dependencies must be done before claiming.

