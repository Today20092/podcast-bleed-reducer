# Smart Gate activity is speech blended with level

The single-microphone gate keeps Silero VAD v6.2.3 as a speech probability and blends it with a smoothed, noise-floor-relative level. Level Bias is the only new user control. Threshold, hold, attack, release, and range then operate on that one activity number. A second model, a desktop ownership mix, and a separate tolerance control were rejected: this plugin has one input, and those alternatives either need enrollment and latency or soften the same edge as the open/close gap.
