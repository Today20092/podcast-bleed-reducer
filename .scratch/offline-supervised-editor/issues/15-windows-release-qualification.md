# Qualify the Windows offline release

Status: ready-for-agent
Blocked by: 05, 13, 14
Spec: ../spec.md

## Scope

Package the Windows native Iced app and pinned runtime dependencies. Verify a clean installation, explicit first-use model download, then network-disabled processing without Python/GPU requirements. Include actual dependency/model notices and supported codec/rate/speaker scope. Do not claim macOS/Linux release qualification from build documentation.

Run final end-to-end save/reload/relink, label correction, protected envelope edit, audition and validated individual WAV export using release binaries. Complete the existing automated release checks and owner listening/performance approval. Record failures honestly; do not release merely because the implementation tickets compile.

## Reach

Release build/installer assets and native DLL search/loading, model acquisition, docs/README supported scope, project source/recipe compatibility, automated release-gate evidence and listening sign-off.

## Acceptance

- Clean Windows install launches the editor without a terminal window and locates all required native dependencies.
- First download failures are recoverable; successful installed-model runs need no network or Python.
- Advertised common mono codecs/mixed rates and eight-speaker/two-hour scope match tested release behavior; unsupported inputs fail clearly.
- No detected wanted/overlap loss, measurable accepted bleed improvement and owner listening approval are present.
- The owner accepts measured cold-analysis wait and interactive performance. Missing approval or failed gates leave release incomplete.
- Source recordings, manual decisions, old recoverable records and prior valid deliverables remain intact.
