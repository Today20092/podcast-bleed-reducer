# Rust ML frameworks in CleanSlate

Research date: 2026-10-18. Scope: current use and practical benefit of Burn, Candle, tch-rs, Linfa and SmartCore. This is research, not an accepted dependency change or model-quality claim. Official documentation was retrieved through Context7 for Linfa, Candle and Burn; first-party project documentation supplies the remaining comparisons.

## Current implementation

**Yes: CleanSlate already uses Linfa.** The manifest pins `linfa = 0.8.1` and `linfa-logistic = 0.8.1`, with `ndarray = 0.16.1`. Burn, Candle, `tch`/`torch-sys` and SmartCore are absent from the checked manifest and lockfile. [Cargo.toml](../../Cargo.toml), [Cargo.lock](../../Cargo.lock).

This is active training code, not an unused dependency: the session ownership path calls `OwnershipModel::train` for each source. Training constructs an ndarray dataset, fits Linfa logistic regression with regularization and convergence settings, normalizes class orientation, and stores coefficients and normalization data. Its diagnostics compare the application's probability calculation with Linfa's prediction probabilities; subsequent estimates use the stored normalization, coefficients and sigmoid. [session_ownership.rs](../../src/session_ownership.rs), [ownership_model.rs](../../src/ownership_model.rs).

The app also already runs a pretrained neural model: its VAD module embeds `silero_vad.onnx` and imports ONNX Runtime bindings from `ort`, pinned to `2.0.0-rc.10`. Thus, adding a framework is not necessary simply to run AI locally. [Cargo.toml](../../Cargo.toml), [vad.rs](../../src/vad.rs).

The roadmap's 2026-10-09 acoustic-only override removed Nemotron; historical Nemotron passages in the specification must not be read as current implementation. The accepted ownership model remains independent binary logistic regression per assigned track, trained from human references; more complex classifier families are outside this release. [Active map](../../.scratch/offline-supervised-migration/map.md), [accepted specification](../../.scratch/offline-supervised-migration/spec.md).

## Framework comparison

The final column is an engineering recommendation for this repository, not a measured framework ranking.

| Framework | Current use | Verified capability | Benefit for CleanSlate |
| --- | --- | --- | --- |
| Linfa | Yes | `linfa-logistic` implements two-class and multinomial logistic regression in Rust. [Official implementation overview](https://github.com/rust-ml/linfa/blob/master/algorithms/linfa-logistic/README.md). | Keep it. It already supplies the requested small supervised classifier without implementing a new optimizer. |
| Burn | No | Training and inference with interchangeable backends; ONNX import converts supported models to Rust using Burn APIs. Its documentation explicitly limits supported ONNX operators. [Official README](https://github.com/tracel-ai/burn), [ONNX import](https://burn.dev/books/burn/onnx-import.html). | Consider only if a future approved feature requires custom neural-network training in Rust or a measured deployment advantage. It does not improve existing features or labels merely by replacing the training library. |
| Candle | No | Rust tensor framework with CPU/GPU operation and pretrained-model examples, including Whisper transcription. [Official README](https://github.com/huggingface/candle), [Whisper example](https://github.com/huggingface/candle/blob/main/candle-examples/examples/whisper/README.md). | A plausible future candidate for a specific supported pretrained audio model. Compare that model's implementation with the existing ONNX route before adding a second runtime. Whisper transcription is not itself a solution to microphone ownership or bleed rejection. |
| tch-rs | No | Bindings to PyTorch's C++ API, with gradient-based training. Requires a matching LibTorch installation; its Windows instructions cover runtime paths, MSVC and debug/release ABI compatibility. [Official README](https://github.com/LaurentMazare/tch-rs). | Useful if a chosen model specifically needs the PyTorch engine. Otherwise the additional native deployment requirements need a demonstrated benefit for this portable desktop app. Python is one installation option, not an inherent runtime requirement. |
| SmartCore | No | Classical algorithms include logistic regression, random forests, decision trees, SVMs and nearest neighbors. [Official guide](https://smartcorelib.github.io/user_guide/quick_start.html). | Potential future comparison candidate for a specific alternative classifier. Replacing the current logistic-regression library alone has no demonstrated quality benefit; a new classifier family needs a separately approved experiment. |

## Recommendation

**Keep Linfa and the existing ONNX inference path. Add none of the other four now.** This follows the implemented model and accepted scope rather than claiming that Linfa is universally fastest or most accurate. The application already combines pretrained speech evidence with a small classifier fitted to the recording. [Ownership training](../../src/ownership_model.rs), [VAD inference](../../src/vad.rs), [specification](../../.scratch/offline-supervised-migration/spec.md).

If independent evaluation reveals a specific weakness, propose a narrow experiment before changing dependencies: freeze the current baseline, separate training examples from held-out recordings, and compare wanted-speech preservation, bleed reduction, probability reliability, analysis time, peak memory and packaged deployment. A different runtime running equivalent model computations should be evaluated for numerical parity and deployment/performance; a different model or feature recipe needs fresh quality evidence. This is a recommended evaluation method, not a completed experiment. Existing quality and long-session release gates remain authoritative. [Implementation tracker](../../.scratch/offline-supervised-migration/implementation.md).

No model training, audio listening, runtime benchmarks or package builds were performed for this note. Framework capability does not establish a quality or speed improvement on CleanSlate recordings, and no dependency or implementation was changed.
