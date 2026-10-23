# Portable updater delivery — ticket 73

Date: 2026-10-23. Source baseline: `2e4c3ad331cbd4aed34e5ecdcf035377ddfc5a8f`
plus the scoped ticket 73 implementation committed on development. Cargo remains
1.0.0; the temporary 1.0.1 below is a disposable acceptance binary, not a release.
No main promotion, release tag, package workflow dispatch or publication occurred.

## Delivered behavior

Registered manual checking offers stable releases by semantic precedence, with
per-check prerelease opt-in, release notes, current/offered versions and a manual
package name/link for each advertised native platform. No startup network request
or purchase entitlement exists. Download/verification is separate from restart.

Automatic installation accepts exact clean Windows x64 portable trees on writable
local fixed NTFS storage. Signed metadata binds the whole ZIP and inner manifest;
the verifier bounds archive/directory allocation, extraction and each file, and
rejects unsafe paths, links, aliases, duplicates and identity/hash mismatches.
Mixed/modified/legacy folders, unsupported storage and macOS/Linux use manual
download. All package files move together; obsolete owned files leave the live tree.

The trusted old app is copied as the recovery helper. A flushed journal and
installation index identify a same-volume swap; an actual verified parent process
handle is awaited. Ordinary close/restart share Save/Discard/Cancel and block on
unfinished edits and live workers, including after cancellation requests. Candidate
offline self-check and empty GUI startup acknowledgment (nonce, PID, version and
manifest) precede commit. Failure preserves/restores the old package; the retained
helper recovers the gap between directory renames. No real project is reopened or
migrated during health testing. Explicit cleanup ends rollback, verifies identities,
removes only owned files and can retry interrupted acknowledged cleanup.

The owner generated the encrypted release key in their own terminal. Only the
public key was read and pinned in `assets/update-public-key.txt`; the private key
and password were not read or committed. Existing public 1.0.0 needs one deliberate
manual replacement with a trusted bootstrap build. Signed descriptors/signatures
for future releases require a separate owner signing step documented in
[update signing](../update-signing.md). Ordinary CI never receives that private key.

## Validation

- `scripts/check.ps1 -Local`: **370 passed, 0 failed, 20 ignored**, strict all-target/
  all-feature Clippy, formatting and script fixtures passed. This includes release
  ordering/channel/asset checks, signature/exact-byte/duplicate-field failures,
  archive traversal/link/alias/truncation/resource checks, worker-close guards,
  capacity preflight refusal, preparation-write failure before index publication,
  and interrupted cleanup/unrelated-file preservation.
- The two-axis code-review skill found a dangling-index staging bug and missing
  success cleanup. Both were fixed and regression-tested; both reviewers then
  reported no unresolved actionable source findings.
- Nine packaged Windows cases passed: success plus explicit cleanup; failed offline
  self-check; GUI crash; GUI acknowledgment timeout; helper interruption after old
  tree rename with manual recovery; mixed-folder refusal; corrupt/truncated archive;
  native executable lock; native permission denial before installation. Permission
  ACLs were set and restored only inside a unique disposable fixture root.
- Every case preserved byte hashes of synthetic project/source/export/preference/
  cache fixtures. Success checked new executable identity, the full new inventory,
  removal of an obsolete owned file, stable installation path and retained old
  executable before explicit cleanup. Built-in self-check separately exercised
  synthetic inference, analysis, real fixture project save/reopen and WAV rendering.
- Current public v1.0.0 ZIP delivery was checked by a non-following HEAD request:
  HTTP 302 to `release-assets.githubusercontent.com` over HTTPS, matching the closed
  redirect policy. Redirect credentials/query parameters were not printed.
- Resolved Windows dependency-license coverage passed. The final normal GUI build
  and offline self-check use the production public key, with no fixture overrides.

## Preserved acceptance artifacts

Normal production-key development executable:
`%LOCALAPPDATA%/CleanSlate/BuildCache/debug/rust-app.exe`, SHA-256
`c96c2604695b7c3ec60397139f55c3f02d084d20dfd8190fd9878b8b8adbbffe`.
Final BuildOnly completed in 376.28 seconds including Cargo-lock waiting; executable offline self-check passed.
A preserved copy and final source-commit receipt live under
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/ticket-73-development/`.

Disposable executables (debug/test key; **never distribute**):

| Version | Path | SHA-256 |
| --- | --- | --- |
| 1.0.0 | `%LOCALAPPDATA%/CleanSlate/UpdaterFixtures/old/CleanSlate.exe` | `c2e87ce2951fa029a47c1362249dce3aba451bf8f4789e38519f84e8767c92cf` |
| 1.0.1 | `%LOCALAPPDATA%/CleanSlate/UpdaterFixtures/new/CleanSlate.exe` | `94d61a4c68fb49a799920dfb771fdb4da89370fafdcef997eabf59d874a105ef` |

The main eight-case receipt and all synthetic package roots are retained under
`%TEMP%/cleanslate-updater-e2e-41ac2e890d084267beefe205db3344ec/receipt.json`.
Archive SHA-256: `12a08b151e740cc2b857d9dc8f1686d290f647ecfd816d0debebb6bd866e2bb3`.
The additional native permission-denial receipt is under
`%TEMP%/cleanslate-updater-e2e-7ad6877aea7d4cd2bf1c3f5a4cc9a6b9/receipt.json`;
archive SHA-256: `abff69fdd56fdd2326ab07b223ce249814cb69bb68563b0df76237ff21d45b8f`.
Logs: `%TEMP%/cleanslate-73-final-check.log`, `cleanslate-73-e2e.log`,
`cleanslate-73-permission-e2e-final.log`, `cleanslate-73-production-build.log` and
`cleanslate-73-production-self-check.txt`.

## Explicit limits

No macOS/Linux automatic installer is implemented or claimed. Their native hardware,
manual replacement and GUI keyboard/appearance matrix were not exercised here.
Native Windows startup was exercised by the packaged health handshake; physical
keyboard/sound judgment and an owner unsaved desktop project were not manipulated.
The worker/close contract is covered by software guards and existing project/export
tests. Disk-capacity refusal was tested without filling the physical drive. Power
loss was represented by forced helper exit at the write-ahead rename boundary;
hardware corruption and universal rename durability are not promised.

Release detection used synthetic GitHub response metadata; no future release was
published to manufacture a live newer version. The existing public downloads are
unchanged. Failed-attempt diagnostics are deliberately retained for manual inspection;
successful retained packages require explicit cleanup before another installation.
Initial transient resource errors and the initial permission-fixture harness timeout
are not counted as passing checks; the corrected final runs above are the evidence.
