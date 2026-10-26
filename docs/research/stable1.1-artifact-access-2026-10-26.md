# Stable 1.1 artifact access checkpoint

Date: 2026-10-26. [Ticket 74](../../.scratch/offline-supervised-editor/issues/74-stable-1.1-release.md) remains in progress. This bounded follow-up to the [desktop readiness audit](desktop-next-work.md) attempted access to the existing checked release artifacts; it did not build, sign or publish anything. Source baseline: `development` at `ed6393e`, isolated branch `codex/release-artifacts-74`.

## Remote provenance

Connected GitHub artifact metadata was refreshed for [run 39983829792](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39983829792). All four containers remained unexpired, with tag branch `v1.1.0` / source `88dda154daa9f442018302e19f1c792513fd0874` and the same IDs, sizes and SHA256 digests as recorded in the readiness audit. These are Actions ZIP-container digests, not hashes of inner distributables or signed metadata. No new source or package was substituted.

The GitHub download connector accepted the **Windows artifact 12514913854** request and returned reusable file reference `file_00000000eb1881f5945e15c435c264c8`, MIME `application/zip`, filename `cleanslate-1.1.0-run39983829792-windows-x64-container.zip`. The intended container is 18,756,007 bytes with expected SHA256 `ea7c367112bfa6a57bca3bcff1eda886c2b01cb50fad75739711b4fda7995bb3`. Successful reference creation is not successful local delivery or fresh byte verification.

## Bounded local attempt and access barrier

Local work initially paused at the coordinator's low-memory snapshot. After the coordinator reported recovered headroom and reserved the global guarded slot, one sequential Windows transfer ran through `scripts/resource-guard.ps1` with an explicit **0.5 GiB owned-job commit cap, 2 GiB start threshold, 1 GiB reserve and 300-second timeout**. HttpClient requested headers first and would stream into a create-new retention file with fixed buffers, checking container size/hash before extraction. No archive bytes were read into an in-memory ZIP buffer.

The connector-generated temporary HTTPS file reference returned **HTTP 403** to that local request. The response was rejected before creating a container file. No hash, archive path/link/duplicate/size validation, extraction, package-manifest inspection, inner-checksum comparison, exact Windows descriptor verification or candidate execution could take place. The prepared extraction/verification script therefore has no passing archive-verification evidence from this attempt.

Per the scoped access policy, transfer attempts stopped at that barrier: no retries/reference refresh, other-platform downloads, alternate credentials/configuration, unofficial source, account setup or endpoint workaround. No temporary signed URL or credential is copied into this report, guard arguments or retained logs. The underlying reason for the 403 is unknown; this is an access/delivery barrier, not evidence of a corrupt release package.

Retained local evidence directory:

`C:/Users/User/AppData/Local/CleanSlate/ReleaseChecks/1.1.0/run39983829792-audit74/`

It contains `verify-artifact.ps1` (attempt scaffolding), `windows-transfer-guard.json` and `windows-transfer-guard.log`; no artifact container, extracted package or self-check output was created. Existing release/development artifacts remain untouched.

| Guard result | Observed value |
| --- | --- |
| Reason / status / exit | `command-failed` / `failed` / `1` (HTTP 403) |
| Owned-process cleanup | `cleanup_verified: true`; guarded slot released |
| Elapsed | 1.1501828 seconds |
| Sampled peak private / working set | 1,724,416 / 10,129,408 bytes |
| Kernel peak job commit | 35,172,352 bytes |
| Minimum available physical / system commit | 5,981,757,440 / 5,632,053,248 bytes |

Sampling can miss short-lived peaks; kernel job commit and resident working set are separate measurements. This was a command failure, not a resource-limit rejection.

## Next action and unchanged barriers

Resume only when a supported GitHub/file-reference delivery route actually supplies the unchanged official container into the local retention lane. Retain the authoritative container identity, then sequentially verify container bytes, safe archive inventory, inner checksums/source/version and exact Windows descriptor. No signing may substitute for those checks. Other platforms remain checksum/source-verification targets on Windows; never mount/run their DMGs/AppImage here. Native CI success is separate from fresh physical/native acceptance.

After trusted Windows package verification, its CLI `--package-self-check` remains an agent-ready lightweight task using [88's measured budget](../../.scratch/offline-supervised-editor/issues/88-memory-bounded-execution.md#integration-and-representative-checks--2026-10-26): explicit 1 GiB cap, 2 GiB start threshold, 1 GiB reserve and 90-second timeout, with isolated disposable user data and safe refusal. It was not authorized or attempted before provenance checks passed.

All three incomplete acceptance items in 74 remain unchecked. The exact private descriptor signature, draft/publication/download reconciliation and any unavailable native checks remain unresolved. Public 1.0.0 still requires a trusted manual full-package bootstrap to acquire the updater. No production/private key, password, installed link, application setting, owner project/media, remote asset, release/tag or cache was touched.
