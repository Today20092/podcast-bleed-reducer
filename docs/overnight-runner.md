# Sequential queue runner

`scripts/overnight-runner.ps1` is a Windows PowerShell 7 coordinator for explicit,
reviewed local commands. It defaults to a read-only dry-run. It does not implement
tickets by opening background chats, schedule itself, create worktrees, merge
branches, publish releases or mark tickets done after tests pass.

The canonical queue remains `.scratch/plugin-products/overnight-queue.json`.
Canonical Markdown tickets supply current readiness, dependencies and claims;
the runner checks these against the queue rather than assuming its historical
statuses are current. All prerequisites, including 88 and 99, must be done before
`-Start`. Coordination umbrellas are excluded. Missing commands produce an
`agent-action-required` handoff so the active agent can do the implementation.

## Select a run

Save an execution-plan JSON outside source workspaces. This example deliberately
selects no executable work:

```json
{
  "schema_version": 1,
  "run_id": "review-20261026",
  "workspaces": {
    "desktop": {
      "path": "C:/Users/User/Documents/podcast-automix-rust-gui",
      "branch": "development",
      "commit": "<full 40-character commit from git rev-parse HEAD>"
    }
  },
  "steps": []
}
```

Product lanes use their existing Podcast/Smart Gate workspaces and branches from
[the runbook](plugin-overnight-plan.md). Select each lane's exact checked commit;
no checkout, reset, broad merge or workspace creation happens implicitly.

```powershell
# Inspect order, blockers, ownership, workspace identity and selected commands.
& ./scripts/overnight-runner.ps1 -PlanPath 'C:/logs/overnight-plan.json'

# An explicit start runs only the selected, validated commands.
& ./scripts/overnight-runner.ps1 -PlanPath 'C:/logs/overnight-plan.json' -Start
```

Each `steps` entry has `ticket_id`, `commands` and optional `risks`. A command has
`kind` (`build`, `check` or `lightweight-check`), `file_path`, reviewed executable
`sha256`, an explicit string array `arguments`, optional `inputs` containing
`path`/`sha256`, and optional `artifacts` containing output file paths. Pin input
scripts, binaries and models used by a check; declared outputs are backed up and
hashed so the last good artifact survives a failed attempt. Existing output files
are preserved in the run's `last-good` directory, without automatic restoration
over newer files.

Supported commands are deliberately narrow:

- Cargo `build`, `test`, `clippy` and `fmt` retain serialized execution and timeouts.
  Compilation/test commands require locked/offline dependencies, one job and
  the existing shared target/config. Later/duplicate job or profile/config
  overrides are rejected.
- PowerShell scripts must use `-NoProfile -File` and be pinned in `inputs`:
  `scripts/check.ps1 -Local`, `scripts/gui-dev.ps1 -BuildOnly`, or the existing
  `scripts/test-plugin-hosts.ps1`. The latter preserves its isolated child cache,
  deadlines and PASS contract. The publishing probe build script is excluded;
  use explicit guarded Cargo commands instead.
- An existing, hash-pinned `rust-app.exe --package-self-check` is supported.
  Direct plugin-host executables are excluded in favor of their maintained runner.
- The fixed shipped `fixture` commands (`pass`, `fail`, `wait`)
  are disposable test seams. Additional source-mutation fixtures require an
  explicitly marked temporary repository. Fixtures accept no arbitrary shell
  code and cannot represent product acceptance.

No command-string shell execution, caller-selected execution policy, DAW/vendor
installation, shared-server deep REA, private signing, push/tag/release or installed
link retargeting is provided. Unknown execution paths remain for agent/owner
review. Source and selected command/input identities are checked again before
execution; the runner is a coordinator for trusted reviewed local code, not an
untrusted-code sandbox.

## Ownership, resources and restart

One machine-wide coordinator mutex covers canonical claims and all steps. A new
claim is written atomically as `Status: claimed` with `Assignee: overnight/<run_id>`
only after dependencies, workspace and all selected commands pass validation.
Other owners and stale claims are never stolen automatically. Preparing a
checkpoint before the claim keeps interrupted claim acquisition reviewable.

Every command runs through [the execution wrapper](resource-guard.md), including
scripts that also use it. The owner removed all memory thresholds, reserves and
caps after upgrading to 40 GB RAM. Builds retain a 30-minute deadline; existing
package self-check/plugin-host commands use 90 seconds and disposable fixtures
use five seconds. No label can bypass compiler serialization or change the
reviewed timeout policy.

State defaults to `%LOCALAPPDATA%/CleanSlate/OvernightRuns/<run_id>`, or an explicit
`-StateRoot` outside all source workspaces. Identity, per-ticket checkpoints and
the final `run.json`/`handoff.md` are written by atomic same-directory replacement.
Before and after commands, checkpoints retain source commit/branch, tracked and
untracked source fingerprints/edit paths, claim/dependency hashes, selected
inputs/policy, artifact hashes/backups, timestamps, receipts/logs, memory/time,
risks and next action. Canonical ticket files are excluded from source diffs
because the runner owns their claims; the relevant ticket/dependency contents
are checked separately. Other `.scratch` source/config remains fingerprinted.

Run the same explicit command and unchanged plan to resume. Passed commands are
skipped only after verifying source and declared output hashes. Changed plans,
branches, commits, source, inputs, claims or artifacts require review. A command
left running after interruption has an unknown outcome: the whole run stops for
review rather than repeating a possibly completed build. A failed test also stops
the entire run, including later invocations of the same run. No ticket becomes
`done` automatically; `checks-passed` means its acceptance still needs review.

The source snapshot selected at claim time remains the reviewed baseline. Before
every command and after it returns, the runner checks branch, commit, tracked and
untracked source fingerprint, and pinned plan/queue hashes against that baseline.
Post-command observations are retained separately in the command record. Even a
successful command that changes these identities stops the entire queue for
review; its changes never become a new resume baseline.

Executable/input hashes are also revalidated after preparation, immediately
before launch. A rejected identity retains an explicit unlaunched
`review-required` command checkpoint, so a later invocation cannot skip over
the rejected check and continue independent work.

Missing prerequisites/SDKs or owner/agent work skip to independent ready tickets.
Busy-task refusals and timeouts preserve outputs and may skip to independent
work. A later explicit resume gets at most one retry. There is no memory-recovery
preflight or wait. Two blocked attempts exhaust the retry. No cache is wiped,
unrelated process killed or endless recovery loop started.

The host chat cannot restart itself after cancellation/context/process loss.
Checkpoints support a new supported session resuming the explicit runner command;
use supported app automation only after a separate explicit scheduling request.
Neither a morning completion promise nor native listening/signing acceptance is
implied.

## Disposable verification

Run `scripts/test-overnight-runner.ps1` separately from a guarded build. It creates
an isolated temporary Git repository, tickets/plans and small fixed commands.
The suite covers dry-run order/dependencies/ownership, branch/commit mismatch,
stale claims, verified restart, tracked/untracked edits, failed-test stop/resume,
timeout retry, maximum retry count, last-good backups, forbidden Cargo
policy/override paths and competing coordinator exclusion. Evidence and
failure receipts remain in the printed temporary directory. No real product is
built or ticket claimed by these fixtures.

`scripts/test-runner-source-identity.ps1` is the focused source-review regression:
fixed temporary fixtures change tracked source, untracked source, branch or HEAD
during a successful first command. Each case verifies that the second command
and later tickets never run, the reviewed/observed identities remain separate,
and restart cannot accept the changed baseline.
The external-input case also verifies that an already-rejected pinned-input
change leaves a durable review checkpoint before the second command launches.
