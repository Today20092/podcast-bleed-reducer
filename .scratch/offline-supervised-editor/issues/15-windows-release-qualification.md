# Qualify the Windows offline release

Status: superseded
Consolidated into: ../../../docs/branch-workflow.md#release-checklist
Blocked by: 05, 13, 14
Spec: ../../offline-supervised-migration/spec.md

## Retirement into release procedure — 2026-10-20

After the owner accepted current audio/performance and requested reducing redundant tickets, retire this standing qualification ticket. Packages already exist; no packaging implementation remains here. Required mechanical checks belong to the [per-release checklist](../../../docs/branch-workflow.md#release-checklist), including the unfinished downloaded-artifact/interactive checks previously inherited from 25. The older scope/checklists below are historical; none of their unperformed checks is newly asserted as passing. Ticket 14 is closed on owner acceptance rather than independent statistical qualification. No stable designation, tag, publication or main promotion is authorized by this retirement.

## Carried-forward package checks — 2026-10-20

This remains the final Windows release-qualification ticket, blocked by independent quality/performance gate 14. Acoustic-only packaging is implemented and published alpha versions supersede 25's original alpha-1 publication scope. Remaining acceptance from 25 is consolidated here:

- [ ] Test the downloaded final release artifact in clean offline Windows, retaining its version, source commit, checksums and results. Distinguish hosted/ZIP-integrity checks and synthetic package self-checks from interactive application use.
- [ ] Exercise interactive import, audition and export, including real-device listening, on that artifact. Preserve the existing save/reopen/relink, labels, protected manual edits and output-validation acceptance below.
- [ ] Record physical limitations and owner timing/listening approval; do not infer acceptance from publication or ordinary passing tests. Coordinate the workflow/device scenarios in 23 and independent quality evidence in 14 without duplicating their reports.

The old required Nemotron acquisition contract is retired. Silero is embedded; no first-use model download is required. This audit neither publishes a release nor passes any outstanding hardware/audio gate.

## Scope

Package the Windows native Iced app and pinned runtime dependencies. Verify a clean installation and network-disabled processing with embedded Silero, without Python/GPU requirements. Include actual dependency/model notices and supported codec/rate/speaker scope. Do not claim macOS/Linux release qualification from build documentation.

Run final end-to-end save/reload/relink, label correction, protected envelope edit, audition and validated individual WAV export using release binaries. Complete the existing automated release checks and owner listening/performance approval. Record failures honestly; do not release merely because the implementation tickets compile.

## Reach

Release build/installer assets and native DLL search/loading, embedded-model availability, docs/README supported scope, project source/recipe compatibility, automated release-gate evidence and listening sign-off.

## Acceptance

- Clean Windows install launches the editor without a terminal window and locates all required native dependencies.
- Embedded-model inference and complete processing work without a model download, network or Python; missing/corrupt package dependencies fail clearly.
- Advertised common mono codecs/mixed rates and eight-speaker/two-hour scope match tested release behavior; unsupported inputs fail clearly.
- No detected wanted/overlap loss, measurable accepted bleed improvement and owner listening approval are present.
- The owner accepts measured cold-analysis wait and interactive performance. Missing approval or failed gates leave release incomplete.
- Source recordings, manual decisions, old recoverable records and prior valid deliverables remain intact.
