# Integrated calculation benchmark

Commit: fdfde4d. This includes tickets 01–03, before live scheduling, audition and detector-quality integration. Release executable, 20 runs per control on the user's full three-stem session, no WAV outputs.

| Control | Median ms | p95 ms |
| --- | ---: | ---: |
| Attenuation | 1.701333 | 2.237365 |
| Open | 1.657822 | 2.121010 |
| Close | 1.722077 | 2.133232 |
| Sensitivity | 29.349959 | 31.924588 |
| Bleed | 30.292816 | 32.720430 |
| Padding | 30.025534 | 32.324360 |
| Protection | 2.148876 | 2.749791 |

Preparation count: 1. Inference frames: 468087. Ownership derivations: 61. Restored default preview equals the initial cold preview exactly. Each source contains 239660221 samples.

Cold preparation: 129747.464901 ms. Concurrent task builds were active and hit compiler memory pressure during this run, so this cold number is not comparable to the earlier isolated 91.7-second baseline. Warm values are computation only, excluding editor scheduling and drawing. Repeat at final integration for final-build evidence.

Command:

```powershell
preview_benchmark.exe 20 'C:/Users/User/Downloads/chopshow podcast 155_A03.wav' 'C:/Users/User/Downloads/chopshow podcast 155_A02.wav' 'C:/Users/User/Downloads/chopshow podcast 155_A01.wav'
```

The executable was built with `cargo build --release --locked --bin preview_benchmark` from this integration worktree. No source audio was copied or modified. No listening or GUI-latency claim is made.
