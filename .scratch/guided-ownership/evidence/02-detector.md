# Detector experiment and implementation

Branch starts at shared contract 10f2cd0. Test seams are pre-approved in decisions.md, round 3, item 19. Implementation is separate from parent editor wiring and actual-session quality acceptance.

## Bounded comparison before implementation

A deterministic 4096-sample harmonic/modulated signal was compared with a copy delayed by 13 samples. Search covered signed lags -48 through 48. Squared normalized correlation removes polarity and scalar gain. Least-squares residual fraction is 1 minus squared correlation.

| Case | Recovered lag | Explained power | Residual fraction |
| --- | ---: | ---: | ---: |
| Copy at -0.25 gain | 13 | 1.000 | 0.000 |
| Same copy plus independent signal | 13 | 0.434 | 0.566 |
| Copy at 1.6 gain | 13 | 1.000 | below 0.000001 |

Conclusion: lag-aware evidence distinguishes the constructed overlap, but correlation cannot establish direction under recording gain mismatch. Solo examples must learn directional level relationships. These synthetic results do not establish real-recording quality.

Parent independently probed the provisional 828–831 second screenshot interval in the full originals. Median absolute speech-band correlation was 0.675 for A03/A01 and 0.585 for A02/A01, with level differences -15.72 and -18.45 dB. This excludes requiring near-perfect correlation as the only useful rule. The interval remains provisional and is not a training label here.

## Pending measurements

Implementation checks, timing, tradeoffs and review outcomes will be appended after execution. No listening or release acceptance is claimed.
