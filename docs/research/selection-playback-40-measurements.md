# Ticket 40 playback measurements

Measured on the owner's Windows machine on 2026-10-12, with `261005-T001.WAV` selected as three sources, Rodio 0.21.1, Focusrite USB Audio output at 96 kHz and the Release profile. Output was muted. No device or global OS settings changed.

The software marker is the first valid prepared PCM sample yielded to the output callback for the current command. Silence emitted while priming does not satisfy it. This does not measure hardware audible onset or completion of a whole hardware output buffer.

| Measurement | Before | Retained transport |
| --- | ---: | ---: |
| Repeated warm starts | 23 | 23 |
| Software marker median | 89.42 ms | 18.91 ms |
| Software marker p95, nearest rank | 102.57 ms | 21.57 ms |
| Software marker maximum | 103.03 ms | 21.73 ms |
| Initial device/source start | 130.77 ms | 138.24 ms |
| Full-source preparation entries during warm commands | Cache reuse in baseline | 0 |
| Device opens across 24 starts | 24 | 1 initial, 0 subsequent |
| Playback workers across 24 starts | 24 | 1 initial, 0 subsequent |

The final quiet run's warm dispatch/source-attachment p95 was 3.26 ms. Initial in-memory preparation took 9.81 s with existing on-disk caches; the earlier baseline took 6.13 s under different build/I/O activity. These are not controlled empty-cache conversion comparisons. Conversion, GUI input dispatch and the old GUI result-delivery contribution were not isolated. The old app used a 100 ms preparation polling interval; no guessed polling delay is added to the measured baseline.

The retained run counted 22,080 missing-buffer sample requests while priming repeated starts. After the first valid sample of each start, 350 ms of continued playback produced zero additional underruns across all 24 trials. This exercises refill beyond the bounded queue but does not establish long-session or physical listening acceptance. A separate repeat sampled headless process working set every 50 ms, with a peak of 14.79 MiB; GUI/GPU memory and baseline process memory were not measured. Prepared PCM remains disk-backed and shared; three-track queued/current/in-flight PCM blocks are bounded to approximately 0.28 MiB, excluding allocator overhead and envelopes.

The opt-in `audition::tests::owner_selection_start_timings` test asserts zero full-source preparation entries, one output open and one worker across repeated ranges. It injects the same error flag set by CPAL's callback, verifies failed output refuses reuse without reopening automatically, and verifies explicit retry opens exactly one replacement stream. This is a simulated boundary failure, not a physical unplug test.

Verification: 263 tests passed across 33 suites, 18 ignored; strict all-target/all-feature Clippy, formatting, GUI BuildOnly and Release build passed. Both source reviews have zero remaining findings. Exact range/loop, pending seek, completed-range restart, output-rate cache invalidation and existing app/snapshot regressions passed. Owner checks for perceived delay, clicks/dropouts, monitoring modes, Solo, popup loops and pause/resume remain pending. No third-party source was copied and no dependency changed.

Detailed local logs: `C:/Users/User/.codex/playback-40-quiet-benchmark.log` and `C:/Users/User/.codex/playback-40-memory-benchmark.log`. Source review reports are in the same directory as `playback-40-spec-review.md` and `playback-40-standards-review.md`.
