# Check a desktop release run

Run from this repository in PowerShell 7 with authenticated GitHub CLI:

```powershell
./scripts/release-status.ps1 -RunId 38366852428
./scripts/release-status.ps1 -RunId 38366852428 -Repository Today20092/podcast-bleed-reducer -Json
./scripts/test-release-status.ps1
```

The helper only reads GitHub. Repository defaults to `gh repo view`; an explicit
repository must use `OWNER/REPO` on github.com. It resolves `gh` once from PATH,
then the standard Windows installation path.

Each of the four platforms requires successful **Check and test**, **Build and
verify application package**, and **Save checked package** steps, a completed
successful job, and a nonexpired artifact with the platform's name. Missing,
skipped, cancelled and pending steps never count as successful. Cache saving is
not evidence that tests or packaging passed. Latest jobs are selected by name;
successful jobs retained from earlier attempts remain valid after a failed-job
rerun, while newer pending retries replace their earlier results.

Workflow conclusion, package readiness and draft preparation are separate.
A successful build-only run can have all packages ready without a prepared draft.
Release readiness requires all packages, successful workflow completion and the
successful **Prepare draft release** step (or the historical **Prepare draft prerelease** name). This reports workflow evidence,
not public release publication or a subsequent download check.

For failed jobs, available failed-step logs supply up to 12 diagnostic lines
(test summaries, native exit codes, failures and panic locations). Logs that are
expired, inaccessible or not yet available are marked unavailable. Passing step
status is reported for successful tests; individual test totals are not fetched
from successful-job logs. No full logs, environment dumps, downloads, reruns,
cancellation or release mutations are performed.

Artifact availability means the Actions API lists a nonexpired named artifact;
its contents and checksum are not independently downloaded by this helper.
Package verification is established by the existing package verification step.
Workflow step/platform renames must update this helper and its fixtures together.
