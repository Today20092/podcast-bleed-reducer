# Full-session preview performance

Measured the optimized integrated production code with all three original `chopshow podcast 155_A01.wav`, `_A02.wav`, and `_A03.wav` files, 239,660,221 samples per track, using `full-calibration.json`. The helper was built after `ef60ce3`; the later quality delta adds tests and evidence without changing classification. These are calculation timings, not end-to-end GUI repaint or audio-device latency.

Cold preparation: **96.442 seconds**, including **6.625 seconds** to prepare cross-microphone features. One preparation and 468,087 inference frames total; none repeated during warm edits. Cached joint features retain **4,593,736 bytes** for three pairs. Restoring the initial settings reproduces the initial preview exactly.

Twenty edits per control, grouped by control so restoring calibration cannot be charged to the next attenuation edit:

| Control | Median ms | p95 ms |
|---|---:|---:|
| Attenuation | 1.235 | 1.609 |
| Open time | 1.205 | 1.359 |
| Close time | 1.270 | 1.471 |
| Speech sensitivity | 46.597 | 48.288 |
| Bleed discrimination | 46.694 | 48.879 |
| Speech padding | 46.727 | 48.304 |
| Protection | 1.481 | 1.654 |
| Calibration | 42.759 | 45.238 |

The previous responsiveness build's joint-free sensitivity/bleed/padding p95 was about 26–27 ms on these sources. The additional ownership comparison increases that work to about 49 ms here, while retaining cached inference and inexpensive gain edits. Cross-run machine load differs; these are measured observations, not a controlled hardware benchmark.

Observed benchmark-process peak working set: **2,929,762,304 bytes** (about 2.73 GiB); sampled private memory peak: **2,915,491,840 bytes**. Poll interval 100 ms. This includes full-source preparation but excludes the GUI, waveform cache, and audition PCM, so it is not a whole-application memory claim.

Raw results: `full-preview-performance.json`. The earlier `full-preview-performance-mixed-controls.json` is retained for provenance: its interleaved calibration changes contaminated attenuation timings, so do not use it for per-control comparisons.

Reproduce from the integration worktree, substituting the release binary path:

```powershell
& .scratch/guided-ownership/evidence/benchmark_memory.ps1 -Binary '<release>/preview_benchmark.exe' -Settings .scratch/guided-ownership/evidence/full-calibration.json -Sources @('C:/Users/User/Downloads/chopshow podcast 155_A01.wav', 'C:/Users/User/Downloads/chopshow podcast 155_A02.wav', 'C:/Users/User/Downloads/chopshow podcast 155_A03.wav') -Iterations 20
```
