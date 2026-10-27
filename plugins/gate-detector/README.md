# Gate detector feasibility only

Ticket 92 experiment, independently locked and not linked into Smart Gate.
See the [measured decision and historical checkpoint](../../docs/research/smart-gate-detector-92.md).
Tests, strict Clippy and the 165-run inference harness pass. Results support a
conservative worker/gain prototype, not callback inference or sound qualification.
Ticket 92 remains claimed pending review; Gate audio is still pass-through.

The optional recorded-category extension also passes **225 runs** with actual
whispering, a short spoken affirmative, human laughter and saved human overlap
references. Low whisper/laughter probabilities remain strong preservation risks;
category execution is not passed audio-quality acceptance.

From repository root, use shared Cargo settings and the standard resource guard:

```powershell
$s = & ./scripts/local-build-settings.ps1
& ./scripts/resource-guard.ps1 -FilePath (Get-Command cargo).Source -ArgumentList (@('test', '--all-targets', '--manifest-path', 'plugins/gate-detector/Cargo.toml') + $s.CargoArguments)
& ./scripts/resource-guard.ps1 -FilePath (Get-Command cargo).Source -ArgumentList (@('clippy', '--all-targets', '--manifest-path', 'plugins/gate-detector/Cargo.toml') + $s.CargoArguments + @('--', '-D', 'warnings'))
& ./scripts/resource-guard.ps1 -FilePath (Get-Command cargo).Source -ArgumentList (@('build', '--example', 'feasibility', '--manifest-path', 'plugins/gate-detector/Cargo.toml') + $s.CargoArguments)
```

Then guard `$s.TargetDir/debug/examples/feasibility.exe` with two arguments: the
canonical local authorized `test-audio` root and a new private report JSON path.
Never commit audio or infer missing labels. The harness verifies source hashes,
pins the exact reviewed reference-project hash (changed projects require explicit
binding/label review and a new fixture identity),
reads at most three seconds per human reference, retains all probabilities and
compares exact evidence across five blocks and three rates. It creates its report
without overwriting an existing file. The harness is synchronous and allocates;
no audio callback safety, gain processing or DAW qualification is implied.

For recorded-category smoke coverage, add a third argument containing the three
already decoded public PCM files described in
[`fixtures/public-categories-v1.json`](fixtures/public-categories-v1.json):

```text
feasibility.exe AUTHORIZED_TEST_AUDIO NEW_REPORT_JSON PUBLIC_FIXTURE_DIRECTORY
```

The compiled metadata manifest supplies exact PCM bytes/hashes, attribution,
licenses, decoding and fixed excerpt selection. It does not download or ship
audio. Same-size changed PCM, nonfinite input or wrong source/project bindings
are rejected before model preparation. This mode also verifies the pinned
`quick-test-10min` project and all three complete source hashes, then reads the
full saved overlap reference 7 from track 1. The source-bound human active set is
[1,2]; original reference 13 is solo and is never called natural overlap.
