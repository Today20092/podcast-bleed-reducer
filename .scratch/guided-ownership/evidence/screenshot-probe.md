# Screenshot failure: independent short-window probe

Read-only measurements from the original full stems, ordered A03/A02/A01, September5 2026. The user labels upper/middle circles as bleed and the bottom as direct speech. The inner828–831s interval and track mapping are inferred from the screenshot ruler and prior full screenshot, not sample-exact new human labels.

The independent Python probe reads only selected WAV ranges. It applies a127-tap Blackman-windowed300–3400Hz bandpass, decimates to8kHz, measures96ms windows every32ms, and searches±12ms for maximum absolute normalized correlation. This is exploratory measurement, not the production classifier. It generates no WAV output.

| Pair, left/right | Median absolute correlation | Median left-minus-right dB | Median correlation lag ms |
|---|---:|---:|---:|
| A03/A01 | 0.6750 | -15.715 | -3.125 |
| A02/A01 | 0.5850 | -18.446 | -2.375 |
| A03/A02 | 0.5638 | 2.983 | 0.625 |

There are91 windows in the inner interval. The correlation10th percentiles are0.2508 and0.2378 for the two microphones versusA01, so assuming nearly perfect correlation would miss real examples. Band-limited RMS over the entire interval is0.0060372,0.0039274,0.0404284 for A03,A02,A01 respectively.

The wider819–847s window contains872 measurement frames and speaker/transient changes. Its corresponding median correlations are0.5530 and0.4310, median level differences -13.808 and -16.495dB. Do not label that whole wider interval as single-speaker bleed based on the cropped screenshot.

Reproduce with the adjacent probe_copy.py using NumPy. On this host: uv run --no-project --python C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe python <absolute-path-to-probe_copy.py>. The bundled Python already includes NumPy. Keep training/calibration spans outside this scored interval.
