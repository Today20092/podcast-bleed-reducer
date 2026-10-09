# Release evaluation

The existing `ownership_evaluate` and `release_gate` binaries accept the same project command:

```text
ownership_evaluate --project session.cleanslate frozen-manifest.json new-evidence.json cache-directory
release_gate --project session.cleanslate frozen-manifest.json new-evidence.json cache-directory
```

These commands verify the installed required model offline and use the editor's supervised worker. They require usable development solo and room references already saved in the project. Evaluation labels are read only for scoring and must not intersect any development reference interval. The project is not rewritten. Source changes, saved project changes, installed runtime changes and an existing report path reject publication.

The version 1 frozen manifest records `claim` (`controlled`, `known_case` or `independent`), nonempty `provenance`, `prior_use`, the project's `reference_identity`, each source's stable `track`, `fingerprint` and native `grid`, and labels containing `track`, common-clock `start`/`end`, `kind` (`preserve` or `bleed`) and `case`. Unknown fields, source/grid changes, contradictory physical-sample labels and false independent claims for declared prior use fail closed. Current independent declarations must have empty prior use; a later typed provenance contract will distinguish integrity-only inspection from training and tuning.

Reports separate automatic gain from effective gain after manual edits and protections. A manual override cannot hide automatic wanted-speech or overlap loss. Gain scoring counts attenuation on zero-valued samples too; PCM equality alone cannot establish preservation. Per-track sample denominators retain native rates, while simultaneous preservation uses common-clock ticks. Ownership probabilities and Owned/Absent/UncertainUnity sample counts are reported separately.

Automatic preservation loss gives `release_status: failed`, even while other gates are pending. With a frozen baseline selection, the commands also fail a measurable bleed comparison that does not improve automatic output RMS. Manual edits cannot supply that improvement. Otherwise reports remain `incomplete`: independent real truth, listening and measured timing approval require separate evidence. Neither command grants release approval.

Independent manifests additionally require `sound_sha256` (the canonical saved parameters and gain constraints), the exact six-field `analysis_recipe`, `engine_sha256` (the evaluator executable), and a `baseline` selection. The latter contains `receipt`, its pinned `receipt_sha256`, a preselected `variant` (`legacy_default`, `automatic_joint` or `guided_joint`), and `development_settings` with its pinned SHA-256. Pin these before observing evaluation results. Recipe equality is checked against the completed editor result.

The comparison verifies the exact `6a60cc2` checkout, unchanged lock/model asset, copied executable closure and receipt hashes before and after running the original binary. Guidance must be contained in existing source-bound development solo/room labels; scoped room truth cannot calibrate another microphone. Held-out truth cannot share an outward-rounded native sample with development truth, even at a touching tick boundary. The original evaluator runs in a uniquely owned attempt directory with cancellation and child-process cleanup. Its selected variant, actual physical intervals and report bytes are checked. Raw reports remain local for reproduction; alternative variants are not used to select the acceptance baseline.

Current frozen comparison requires identical mono 48 kHz native grids and unambiguous CSV-safe basenames. Mixed-rate native gain scoring already works; verified common-grid derivatives for historical comparison remain pending. Both scorers derive the same frozen manifest digest from the validated source, truth, reference and recipe declarations. Comparison rejects a differing digest as well as differing input-energy/sample denominators. A missing baseline is explicitly reported as pending.

The existing performance binary also accepts the saved-project workflow:

```text
preview_benchmark --project 10 session.cleanslate new-benchmark.json cache-directory
```

It loads the saved project and verifies the installed required runtime, then measures the same editor worker's preparation, references, native history, classifier, temporal and overview stages. A second analysis measures reuse. Transient gain edits use the effective-envelope cache without saving edits or invoking inference. The report records native history endpoints and compaction metadata, classifier counts and cache counters. A cold application-cache claim requires an initially empty cache; OS caches are not flushed.

A separate cancellation namespace reuses immutable prepared audio through hard links and starts with fresh native history. The public job flag is requested 20 seconds after progress reaches 20, which means preparation and reference readiness have completed, before runtime/history work. Actual child-process observation is needed to identify a more precise phase. The report distinguishes active cancellation, completion before a request, completion despite a request and failure. Completion before the timer preserves the first and warm measurements rather than granting cancellation evidence. A pre-canceled job must retain the latest accepted analysis. Logical cache bytes count hard-linked files per path; physical disk usage and sampled process-tree memory require an external measurement receipt.

Ticket 14 remains claimed. Listening-region delivery, typed prior-use provenance, mixed-rate frozen comparison derivatives and the complete long-session benchmark are still being implemented. Controlled test fixtures establish mechanics only.
