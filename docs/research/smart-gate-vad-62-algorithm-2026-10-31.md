# Smart Gate algorithm: stay on Silero v6.2, do not retune yet

2026-10-31. Decision note for the live single-microphone gate. It records the
model check, the sourced comparison of later Silero releases and other
published detectors, and the algorithm choice that follows. No threshold was
changed. No second model was added. The GUI was not started.

Companion notes: [bundled Silero provenance](silero-model-provenance-2026-10-31.md),
[gain policy](smart-gate-gain-policy-93.md),
[detector feasibility](smart-gate-detector-92.md).

## Decision

Keep the current gate.

1. The streaming model already in `assets/silero_vad.onnx` is Silero VAD
   v6.2 / v6.2.3, not v5. Leave that file in place. Do not switch to the
   ifless export or the offline sequence ONNX.
2. Keep the gain rule: open at 0.10, close at one tenth of that and never
   above 0.01, hold 300 ms, release about 5 ms, attack about 250 ms, range
   floor at most −6 dB, evidence expiry 100 ms, fault and expiry return exact
   unity.
3. Before any GUI work, rescore the existing whisper, laughter, and noise
   fixtures with this model. Move the open and close lines only if that pass
   says the v6.2 probabilities no longer match the reason those lines exist.
4. Do not add RNNoise, DeepFilterNet, or a personal-VAD model in this step.
   Each one changes the product from a gate into enhancement or enrollment,
   and each one adds latency or a reference recording.

## What is already in the tree

`assets/silero_vad.onnx` is 2,327,524 bytes. SHA-256
`1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3`.
`git hash-object` of the local file is
`80c5592ef1f4c9ede3e357bbd02eb863358a6a9d`.

That git blob is the streaming file
`src/silero_vad/data/silero_vad.onnx` at both
[v6.2](https://github.com/snakers4/silero-vad/releases/tag/v6.2)
(`be95df9`) and
[v6.2.3](https://github.com/snakers4/silero-vad/releases/tag/v6.2.3)
(`5cd7945`, latest 6.2 release as of this note). The v5.1 blob is different
(`b3e3a900c0d70e67b5e2b90a33ad856ee7947930`) even though the file size is the
same. A download of the v6.2.3 contents-API file hashed to the same SHA-256
as the local asset.

The gate recipe string had still said `silero-v5`. It now says
`silero-v6.2.3/causal-blackman127-fc7000-delay64/16k512-context64-v1`
in `plugins/gate-detector`. The desktop report label `silero-vad-v5` is the
same stale name on the same bytes. This note does not change that desktop
string.

v6.2.3’s own release text only makes torchaudio optional for file read and
write. It does not publish a new streaming model. v6.2.2 added a separate
offline file, `silero_vad_16k_sequence.onnx`, for whole recordings. That file
is not embedded.

## Session contract

The official ONNX wrapper is the same shape from v5.1.2 through v6.2 and
current master. Each step takes 512 new samples at 16 kHz (32 ms), prepends
64 samples of carried context, passes LSTM state shaped `[2, batch, 128]` and
an int64 sample rate, and returns one speech probability plus the next state.
Inputs are named `input`, `state`, and `sr`. Outputs are `output` and
`stateN`. Silero’s README says one 30+ ms chunk takes under 1 ms on one CPU
thread. That figure is not a measured difference between v5 and v6.2.

`plugins/gate-detector` already calls the session that way. The host rates
44.1, 48, and 96 kHz are low-passed with the existing 127-tap causal Blackman
before the 16 kHz frame. Invalid evidence, a worker fault, bypass, and
evidence older than 100 ms return gain 1. That fail-open rule is ours. The
papers below do not require it.

## What v6 changed, and what it did not

[v6.0](https://github.com/snakers4/silero-vad/releases/tag/v6.0) kept the v5
feature set, changed training, and claimed 16% fewer errors on noisy
real-life data and 11% fewer on a multi-domain set. Music with voice-like
instruments, and very high artificial, cartoon, and small-child voices,
stayed known failures.

[v6.2](https://github.com/snakers4/silero-vad/releases/tag/v6.2) retrained
again. The note says a slight overall improvement with no metrics update,
higher stability on rare data, and better behavior on unusual, child,
cartoon, and muted voices, muted speech, and lower-quality phone calls.
Those claims are qualitative. The new probabilities are not a numerical
stand-in for scores measured on v5.

The ifless ONNX added in v6.2 is an opset-18 export. The default session
still takes `sr`. Do not load the ifless file in place of
`silero_vad.onnx`.

## Why the thresholds stay

The open line 0.10 and the close line 0.01 were chosen so a whisper whose
v5-era mean was about 0.14 would reach unity, while laughter whose mean was
about 0.009 could still be attenuated. That is recorded in
[the gain-policy checkpoint](smart-gate-gain-policy-93.md). It was not a
claim that whisper or laughter is preserved.

Silero’s own streaming iterator opens at 0.5 and closes about 0.15 below
that, never under 0.01. Copying 0.5 onto v6.2.3 without a new fixture pass
can cut the quiet speech the 0.10 line was written to keep. The rescore is
the next algorithm change. The GUI is not.

## Rejected for this step

These are real options. They are not the next edit.

| Option | What it is | Why it waits |
|---|---|---|
| RNNoise | Hybrid DSP/GRU noise suppressor, look-ahead limited to 10 ms | Replaces noise in the signal. Early C timing was about 60× real time on x86 and about 7× on a Raspberry Pi 3. [demo](https://raw.githubusercontent.com/jmvalin/rnnoise_demo/master/index.html) |
| DeepFilterNet | 48 kHz enhancement | Published real-time setup is a 20 ms window, 10 ms hop, and 2 frames of look-ahead: 40 ms latency. Single-thread real-time factor 0.19 on an i5-8250U. [paper](https://arxiv.org/html/2305.08227) |
| Personal VAD | Target speech vs other speech vs silence, about 130k parameters, per frame | Needs an enrollment recording of the person at the microphone. [paper](https://ar5iv.labs.arxiv.org/html/1908.04284) |
| Personal VAD 2.0 | Streaming, no right context | 32 ms window, 10 ms hop, 4 stacked frames subsampled by 3, causal convolution, 31 frames of left context. [paper](https://arxiv.org/pdf/2204.03793) |
| TEN VAD | Another speech probability, hop 160 or 256 samples at 16 kHz (10 ms or 16 ms) | Still a threshold on a score. A shorter hop is a later latency experiment, not a smarter decision by itself. |

A level check beside the speech score (open only when the probability clears
the line and the signal is above a recent noise floor) is the smallest
extra rule if the rescore shows false opens on quiet room tone. It stays
causal. It is not part of this decision.

## Coverage

The comparison above is the written result of the `deep-research` run on
this question. The run’s status was partial.

- v6.2 edge-case gains have no new published metric.
- The README’s “under 1 ms” figure is not version-compared.
- Input names were taken from the official wrappers and from this session,
  not from a fresh dump of both ONNX graphs.
- The ifless file’s input list was not read from the graph.
- Personal VAD 2.0 does not publish one end-to-end latency number. The 30 ms
  output step is inferred from a 10 ms shift subsampled by 3.
- Third-party DeepFilterNet builds advertise less look-ahead. The paper
  cited here specifies the 40 ms setup.
