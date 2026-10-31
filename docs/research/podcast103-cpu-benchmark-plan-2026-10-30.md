# Podcast 103 comparable CPU benchmark preparation

Executed at benchmark source `b5052b2`; see
[the measured results](podcast103-cpu-benchmark-results-2026-10-30.md).
No native presentation or 60 Hz qualification.
Managed child worktrees, with setup hooks skipped:

- `podcast103-cpu-baseline`, branch `codex/podcast103-cpu-baseline`, exact
  historical `d2b30c91fa5a7f078ca75cdbf6999defc69ed513`; source untouched.
- `podcast103-cpu-current`, branch `codex/podcast103-cpu-current`, production base
  `b41d51e89a3bc5d413f83c0193b859e2d1a27f27`; test-only benchmark extension.

Both are siblings of the stable `podcast-seek-resume` worktree. Production source
and native build outputs there remain untouched. The existing
`session_editor::tests::waveform_render_work_measurement` remains unchanged in
the current worktree. Its historical source uses the same algorithm before the
optional gain argument existed. Both query/render the same eight-track fixture,
1000 source peak columns, 720×560/1000×680 logical sizes, fixed 564px lanes and
clipped vertical scroll height. One cold plus 40 warm samples; nearest index
`ceil((n-1)*p)` over warm samples only, matching the existing historical summary.

## Additional current cases

`session_editor::cpu_benchmark::gain_projection_cpu_measurement` logs separately
labelled refresh, cached, gap-refresh, dense-refresh and stale scenarios. Actual
waveform code derives visible member masks from clip rectangles. Its draw span
includes the actual gain projection; no duplicate pre-query or estimated mask.
Refresh sets the existing cache timestamp to None. Cached warm trials set a fresh
timestamp and assert it was retained, so an expired/changed query cannot silently
be reported as a hit. Stale withdraws validity after the cold frame. Gap omits the
16000–24000 range on every member. Dense uses the existing real EffectiveEnvelope
manual-ramp fixture at a two-hour view and asserts detail refusal. Its first-second
envelope is explicitly distinct from the 64000-frame constant-envelope scenarios.
All eight member slots are mapped; actual geometry remains clipped to visible lanes.

`session_editor::cpu_benchmark::egui_dispatch_to_observed_state_cpu_measurement`
uses the same captured eight-track view and actual wheel events. An untimed first
layout establishes hover; each measured dispatch resets only the viewport and
asserts that the source range changed while selection did not. Elapsed time begins
immediately before `run_ui` and ends after reading the changed viewport. This is a
CPU dispatch-to-observed-state upper bound including frame construction, not an
OS input latency or tween duration. It intentionally excludes tessellation.

Raw rows retain query, draw including gain, tessellation, correlated total spans
and mesh vertices. Additional summaries report p50/p95/p99 and CPU-budget
exceedances above 16,666.667 µs; `presentation_misses` is null. Postprocess each
unchanged waveform raw row's correlated query+draw+tessellation total the same
way. Do not sum independent stage percentiles. Counts remain observations from
40 warm frames; tails and maxima are noisy at this sample size.

## Matched identities and future commands

SHA256 (both worktrees match for lock and profile):

| Input | SHA256 |
| --- | --- |
| `plugins/podcast/Cargo.lock` | `8961aaf03b11c3f70053192c7b57b3fa93a0ffebed0b0daf2e6f8ec5e2c2cdaa` |
| `scripts/gui-dev.toml` | `1c44515fee5e1411fcce849c93e9ec641734bf3b7814c477e57d777c4bca85b7` |
| `visual_capture` source with whitespace removed | `c1f76fbe086c8640fc8248c8b58b048e5204aceec5392514256bcc98805bd442` |
| pinned 1.99 Cargo executable | `c9b44417dc8228b81923a1068a3c09e32d2793f2cced2a816789cf433c1f0d46` |
| pinned 1.99 Rustc executable | `fc40f1c8b65095a22a48c111aadc279c26a049b7e4161e8b99ffe3eb23873e86` |
| pinned LLD executable | `b4b3a7e91505e6ab40f97493bebe398908f63582d686b72111e8ca8b08351bf5` |

The current gain harness prints actual native PCM SHA256s before mutation or
measurement. Fixture setup, filesystem hashes, model/effective-envelope setup,
JSON formatting and result assertions are outside measured spans. Final artifact
hashes and compiler metadata must accompany the eventual run receipts.

After compiler-slot grant, serialize builds/runs. Use the CURRENT timeout-only
`podcast-seek-resume/scripts/resource-guard.ps1` and current
`local-build-settings.ps1`, with their absolute paths, even when cwd is baseline.
Never invoke the historical wrapper or its old RAM policy. Use pinned actual
Cargo/Rustc 1.99 and LLD, existing locked/offline single-job shared target settings:

```text
cargo rustc --manifest-path plugins/podcast/Cargo.toml --lib --profile test
  --message-format=json [current shared CargoArguments]
  -- -C linker=<pinned rust-lld.exe> -C linker-flavor=lld-link
```

Preserve the exact JSON compiler-artifact test executable outside the shared cache
immediately after each build. Run that preserved executable through the current
wrapper with a fresh receipt, `--exact --nocapture --test-threads=1` and each test
name above. Run the unchanged waveform test in both checkouts. Run the two new
tests only in current. Keep `CLEANSLATE_PLUGIN_CAPTURES` unset; CPU raster export
is unrelated observer work. Run current strict library/test Clippy after checks.

Historical raw log and preserved old executables are absent at their documented
machine-local paths. The fresh matched baseline rerun is separate from the old
published summary table, which cannot supply missing raw correlations/miss counts.

Allocation-call counts are unavailable. Mesh capacity is geometry storage, not
allocator calls. Process CPU/working-set totals include fixture/worker/test costs
and do not isolate plugin or per-instance GUI costs. No sustained idle/playback,
simultaneous native instances, audio callback underruns, native DPI, compositor
presentation or hardware input-to-photon evidence comes from these CPU spans.
Last-Arc destruction latency remains a separately documented potential cost.
