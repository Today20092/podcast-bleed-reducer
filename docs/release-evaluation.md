# Release evaluation

The existing `ownership_evaluate` and `release_gate` binaries accept the same project command:

```text
ownership_evaluate --project session.cleanslate frozen-manifest.json new-evidence.json cache-directory
release_gate --project session.cleanslate frozen-manifest.json new-evidence.json cache-directory
```

These commands verify the installed required model offline and use the editor's supervised worker. They require usable development solo and room references already saved in the project. Evaluation labels are read only for scoring and must not intersect any development reference interval. The project is not rewritten. Source changes, saved project changes, installed runtime changes and an existing report path reject publication.

The version 1 frozen manifest records `claim` (`controlled`, `known_case` or `independent`), nonempty `provenance`, `prior_use`, the project's `reference_identity`, each source's stable `track`, `fingerprint` and native `grid`, and labels containing `track`, common-clock `start`/`end`, `kind` (`preserve` or `bleed`) and `case`. Unknown fields, source/grid changes, contradictory physical-sample labels and false independent claims for declared prior use fail closed. Current independent declarations must have empty prior use; a later typed provenance contract will distinguish integrity-only inspection from training and tuning.

Reports separate automatic gain from effective gain after manual edits and protections. A manual override cannot hide automatic wanted-speech or overlap loss. Gain scoring counts attenuation on zero-valued samples too; PCM equality alone cannot establish preservation. Per-track sample denominators retain native rates, while simultaneous preservation uses common-clock ticks. Ownership probabilities and Owned/Absent/UncertainUnity sample counts are reported separately.

Automatic preservation loss gives `release_status: failed`, even while other gates are pending. Otherwise current commands report `incomplete`: baseline comparison, independent real truth, listening and measured timing approval are not yet integrated. Neither command currently grants release approval.

Ticket 14 remains claimed. Frozen baseline receipt verification/comparison, frozen parameter/manual/model declarations, listening-region delivery and the complete long-session benchmark are still being implemented. Controlled test fixtures establish mechanics only.
