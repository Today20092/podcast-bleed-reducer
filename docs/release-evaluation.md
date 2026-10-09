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

Current frozen comparison requires identical mono 48 kHz native grids and unambiguous CSV-safe basenames. Mixed-rate native gain scoring already works; verified common-grid derivatives for historical comparison remain pending. Reports reject differing input-energy/sample denominators before comparing bleed. A missing baseline is explicitly reported as pending.

Ticket 14 remains claimed. Listening-region delivery, typed prior-use provenance, mixed-rate frozen comparison derivatives and the complete long-session benchmark are still being implemented. Controlled test fixtures establish mechanics only.
