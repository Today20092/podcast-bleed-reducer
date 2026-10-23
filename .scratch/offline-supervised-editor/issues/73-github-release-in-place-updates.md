# Update the complete portable application from GitHub releases

Status: done
Assignee: Codex
Implementation readiness: ready-for-agent
Type: feature
Blocked by: none
Related: 26, 64, 68, 71

## Owner request — 2026-10-22

Provide an application update feature for new GitHub releases, replacing the complete ZIP/portable package in place rather than asking users to manually replace individual files. Preserve the installed application and user work if an update cannot complete.

The owner requested work on this ticket on 2026-10-23. Discovery is claimed; record the recovery/trust design before implementation. It does not change release publication policy or depend on sponsorship ticket 72.

## Discovery deliverable

### Accepted first delivery — 2026-10-23

The owner explicitly selected **Windows installation first, with a safe manual fallback elsewhere**: install only into clean Windows portable package folders, verify signed complete packages and retain rollback. Mixed folders containing recordings/projects and the advertised macOS/Linux packages use a manual-download fallback initially. Preserve user files rather than reorganizing them to qualify an installation. This accepts the first-delivery platform boundary; it does not claim signed metadata, restart safety or rollback is already implemented.

Inspect the actual package layouts, GitHub release workflow and current application/preferences storage before selecting an implementation. Fetch current platform/library documentation before choosing dependencies. Record the supported update matrix: prioritize the owner's Windows portable ZIP workflow, then explicitly state support or manual-update behavior for each advertised macOS/Linux package. Account for Windows executable locks and macOS bundle/signing constraints rather than claiming one ZIP replacement strategy works everywhere.

Record the smallest viable design, package identity/version manifest, authenticity mechanism, transaction/recovery strategy and any required release-workflow changes here before coding. A checksum downloaded beside an archive can detect corruption but does not independently authenticate a publisher; document the trust model and key/distribution requirements. Existing release-preparation work may be developed separately; coordinate shared packaging changes with ticket 71 before integration.

## Required behavior

- Provide a registered Check for updates action with current/new version, release notes and a deliberate Download and install choice. Manual checking is the initial scope; any startup/background checks require an explicit setting and must preserve offline startup. Do not silently install or restart.
- Query the canonical project GitHub releases. Compare semantic versions correctly, choose the exact OS/architecture/package, exclude drafts and default to stable releases without opting stable users into prereleases. Handle an existing alpha installation and same/older versions explicitly; define prerelease opt-in if needed.
- Download and stage the complete application package, including its executable, required libraries, assets and notices. Verify version/platform/package identity and integrity/authenticity before replacement. Reject mismatched, corrupt, truncated and untrusted content; constrain redirects/download origins to the documented release delivery path.
- Validate archive extraction paths and entries before writing: prevent traversal, absolute paths and link escapes. Bound extraction/resource use. Stage outside the live installation and never extract directly over a running package.
- Before applying, require a safe close of the application through its existing unsaved-project flow and wait for active analysis/export work to finish or be explicitly cancelled. A helper, if needed for executable locks, operates only on the known installation after the parent exits and reports failure recoverably.
- Replace the whole owned package as a recoverable transaction. Remove obsolete package-owned files while preserving unrelated user files, projects, source recordings, exports, preferences and reusable caches. Define how legacy mixed portable folders are handled; refuse an unsafe layout rather than treating every file as disposable.
- Preserve a runnable previous installation and restore it after failed replacement or startup validation. Define startup health checks, crash/power-loss recovery and the point at which backup cleanup is safe. Retain project compatibility: do not automatically migrate or overwrite a user's project while testing an updated binary.
- Handle cancellation, offline/network errors, API rate limits, missing assets, insufficient disk space, read-only/removable installations, antivirus/file locks and permission failures with actionable feedback. Avoid automatic elevation. Keep a usable manual-download route for unsupported platforms/layouts or failed self-update.
- Update release packaging/metadata and user documentation together with the GUI. Preserve ordinary deliberate tag/draft/publication controls; a merged updater feature does not publish a release.

## Acceptance

- [x] Record the package/platform matrix, trust model and chosen recovery design; replace discovery gaps with concrete implementation steps. The linked report defines four stages; use the GUI guide and standard build/check scripts for delivery. Runtime/package checks below remain outstanding.
- [x] A packaged Windows portable installation detects a newer eligible GitHub release and updates every package-owned component through a user-confirmed restart; obsolete owned files are handled correctly. Newer release detection uses synthetic GitHub API metadata; no future release was published for testing.
- [x] Version selection tests cover stable/prerelease ordering, same/older versions, wrong architecture, missing/draft assets and malformed release metadata.
- [x] Archive and verification tests reject traversal/link escapes, wrong identity, corrupt/truncated downloads and authenticity failures before modifying the working installation.
- [x] Transaction tests cover cancellation, download/extraction failure, disk/permission/lock errors, interrupted replacement and failed new-version startup; the previous app remains runnable or recovers deterministically. Capacity refusal is tested without filling the physical drive; forced helper exit represents interruption at the rename boundary.
- [x] An unsaved session and active export cannot be lost by updating. Fixture projects, source audio, exports, preferences, caches and unrelated portable-folder files survive; no destructive test uses owner recordings. Software close/worker guards and synthetic preservation fixtures are the evidence; no owner unsaved desktop session was manipulated.
- [x] Perform a packaged old-to-new update and forced-failure rollback on Windows; record versions, package hashes, executable paths and results. Verify other supported update paths on their platforms; document manual update where unimplemented. Nine Windows packaged cases passed; macOS/Linux automatic installation is unimplemented/manual-only.
- [x] Release metadata, GUI progress/errors, help and manual-update instructions agree. Check registration, keyboard access and offline startup; record remaining platform limitations honestly. Registered action controls reuse focus/shortcut handling; native physical keyboard/appearance and other-platform hardware matrices remain unperformed.

## Results

2026-10-23 completed Windows-first delivery: [receipt](../../../docs/research/portable-updater-delivery-2026-10-23.md). Final local checks passed 370 tests (20 ignored), strict Clippy and script fixtures. Nine disposable packaged cases passed whole-package update, explicit cleanup, self-check/GUI failure rollback, helper-interruption recovery, mixed/corrupt refusal, native file locks and permission denial. Final normal GUI BuildOnly and offline self-check passed with the production public key. The owner generated their encrypted release key privately; only its matching public key was read/pinned. The two-axis review's dangling-index and missing-cleanup findings were fixed and regression-tested; reviewers reported no unresolved actionable source findings. Base source is `2e4c3ad331cbd4aed34e5ecdcf035377ddfc5a8f` plus ticket 73's scoped development commit. Temporary 1.0.1 fixture builds are not a release/version bump; Cargo remains 1.0.0. Existing public downloads are unchanged. No main promotion, tag, dispatch or publication occurred. The receipt records package/executable hashes, preserved paths, synthetic/live-network evidence and explicitly unperformed physical/platform checks. Historical preparation-only notes below are superseded.

2026-10-23 implementation-session handoff (not delivery): Owner confirmed TDD seams at release selection/package verification, installer/helper transaction on disposable roots, and application shutdown/update actions. Owner does not have an established signing-key procedure; explain/provision an externally held release key before enabling production installation. No production key was generated or pinned. A test-only key may exercise verification but must never enable distributed installation. Implementation remains outstanding, including every unchecked acceptance item. Initial dependency/test scaffolding was removed rather than retained unverified; no updater code, completed-work commit, push, release or package replacement resulted. Cargo initially failed on memory allocation and standard-library metadata; a later GUI BuildOnly retry progressed past those failures, so they are not an established persistent blocker. Logs for attempted checks are `%TEMP%/cleanslate-ticket73-check.log`, `%TEMP%/cleanslate-ticket73-gui.log` and `%TEMP%/cleanslate-ticket73-baseline.log`. Do not count these attempts as updater acceptance. Baseline source remains `2e4c3ad331cbd4aed34e5ecdcf035377ddfc5a8f`; unrelated dirty research/tracker files are preserved.

2026-10-23: Completed the [package/trust/recovery discovery](../../../docs/research/portable-updater-feasibility-2026-10-23.md). Existing Windows packages have a complete manifest; Mac packages are ad-hoc signed DMGs and Linux uses AppImage. The proposed first install path accepts only an exact clean Windows package tree on supported local storage, with a manual fallback for mixed/modified folders and other platforms. Signed release descriptors require a pinned verification key and a bootstrap release; adjacent checksums alone do not authenticate the publisher. Existing app close handling is insufficient for update safety: add a shared save/discard/cancel gate and wait for worker acknowledgment before helper replacement. A journaled same-volume directory swap retains the previous app and exposes recovery for the interruption between its two renames. The report defines four implementation stages and fault-injection/package acceptance checks. This is a concrete proposal, not completed runtime code or a verified update. The owner confirmed full-app free use in ticket 72; no purchase entitlement is required for updates.

2026-10-22: Created on owner request and linked from the active roadmap/index. Specification only; no update library selected, live release queried, package replaced or release published.
