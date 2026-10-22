# Alpha.7 performance changes and their evidence

The alpha.7 batch integrates the 2026-10-21 performance work and later editor refinements through development `6160da9`. The measurements below belong to their recorded development builds. They are not a benchmark of the final release package, a comparison with FabFilter, or a promise about all hardware.

## Measured changes

| Change and workload | Before | After | What the observation establishes |
| --- | ---: | ---: | --- |
| Single-track draft, eight synthetic two-hour tracks with dense automatic boundaries, 25 repetitions | 31.024 ms median, 43.496 ms p95 | 19.545 ms median, 22.396 ms p95 | Reusing unaffected track preparation removes work in this component fixture. The files use an 8 Hz source grid; this is not ordinary-rate two-hour audio processing. |
| Attenuation change on the same dense fixture, 25 repetitions | 26.114 ms median, 32.183 ms p95 | 23.190 ms median, 25.690 ms p95 | Dense attenuation changes still rebuild affected automatic envelopes and remain above the 16.7 ms diagnostic review threshold. |
| Complete waveform and gain preparation, three real 600-second 48 kHz tracks, 2176-column locked view, first interleaved run | 14.093 ms median | 13.831 ms median | About 1.9% less software geometry work with exact-flat gain pruning in this run. |
| Same follow workload, repeated interleaved run | 9.665 ms median | 9.535 ms median | About 1.35% less median work; complete p95 was slightly worse. No consistent tail improvement is established. |
| Confirmed minimized playback, short native Windows samples on one 32-logical-processor machine | 105.47% of one core in the baseline sample | 5.39% of one core in the later sample | Optional visual suspension was effective in the checked scenario. These were short sequential observations, not paired controlled trials or total-machine CPU percentages. |

For warm editing, the baseline used `83ba484` plus the measurement driver and unchanged composition behavior; the retained-preparation run used integration base `9446d3e` plus the ticket05 changes, delivered in `c3c447f`. The ordinary three-track 48 kHz fixture changed little: draft median/p95 was 0.412/0.588 ms before and 0.405/0.538 ms after. The optimization is aimed at demonstrated redundant preparation, not a claim that every edit became substantially faster. See [the complete warm-edit report](fabfilter-warm-edit-measurements-2026-10-21.md) for all repetitions, sparse fixtures and source identities.

The follow comparison used `e824482` plus ticket07's change, delivered in `ea2dc4e`, with alternating dense/candidate order in the same process. A constant 0.25 gain curve at 1000 columns retained three necessary vertices instead of 1001. Every changing sample and the outside-source unity boundary remain. The 4096-column probe still exceeded its 16.7 ms diagnostic budget. These timings exclude GPU rasterization, presentation, audio-device output and whole-window layout. See [the follow report](fabfilter-follow-measurements-2026-10-21.md).

The native CPU baseline was source `e520273`. The later integrated observation at `7985942` used the same private three-track 600-second/48 kHz session family; its visible locked-follow sample was 106.14% of one core, so this evidence does not show a visible-playback CPU improvement. Minimization and restore were observed, and playback reached the end while minimized. Working-set samples were not memory peaks. See [the baseline](fabfilter-performance-baseline-2026-10-21.md) and [final native receipt](fabfilter-performance-final-review-2026-10-21.md).

## Work removed without a claimed timing gain

- Unchanged worker polls retain static drawing geometry. Data, source, viewport, edit, appearance and logical-size changes are checked against fresh drawing. The software tests cover Dark/Light and 1x/2x logical bounds; this is not a physical DPI or raster comparison.
- Decoded audio blocks and old envelope snapshots retire outside device-sample iteration. A real ZOOM F8n Pro regression observed an old envelope freed inside sample iteration before the restart fix and outside it afterward. Steady playback, edits and rapid seeks had zero allocation, reallocation or free observations in the scoped public consumer probe. Separate teardown still performs frees. This is ownership/correctness evidence, not a new audible-latency result.
- Shared streaming waveform construction, integrated at `102fee9`, puts import and reopening through one builder. Cold/reused caches, channel separation, cancellation and changed/truncated sources received regression coverage. No new import or cache-throughput speedup was measured for that refactor.
- Dial indicators ease to a new value over 110 ms and request no animation redraw after settling. Audio values apply immediately. This is a presentation change, not faster audio processing.

## Decisions to retain existing behavior

Two ordered contention repeats on three real 600-second/48 kHz tracks measured standalone/concurrent analysis at 22.660/23.258 seconds and 22.321/23.739 seconds. Concurrent foreground completion maxima were below 3.29 ms in the software edit/delivery/sample loop. Automatic and effective envelopes matched exactly and project bytes stayed unchanged.

Those runs did not demonstrate a need for a new scheduler, worker-count control, priority policy, parallel DSP or SIMD rewrite. None was added. The software consumer intentionally rests 20 ms between operations; its starvation observations are not native device underruns, and the fixed standalone-first order is not a randomized causal study. The driver was built from `6ea5818` plus ticket08's changes, delivered in `46c9fae`. See [the contention report](fabfilter-background-contention-2026-10-21.md).

The existing envelope curves already share dense storage through `Arc`. The very small vector-delivery timings did not justify a new playback-vector API. Conditional investigations that found no need for a change are not advertised as optimizations.

## Release interpretation

The completed changes preserve the shared audition/export envelope evaluator, native source grids, source validation, cancellation and safe export publication. They do not add an audio-quality mode or change the supervised ownership model. Current source checks and owner acceptance are recorded separately from these measurements.

There is no claimed whole-application FPS result, physical input-to-photon result, physical audible-latency gain, inference acceleration, ordinary-rate two-hour session qualification or fresh Mac/Linux hardware measurement here. Quiet-overlap limitations and independent listening requirements remain. The [alpha.7 release notes](../alpha-release-notes.md) describe the delivered controls and release-verification boundary.
