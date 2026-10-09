# CleanSlate

CleanSlate is an offline microphone bleed-reduction tool for multitrack podcasts. It uses a supervised algorithm to learn from a few examples of who is speaking, then attenuate each microphone when its assigned person is silent. Listen to the result and adjust the gain where needed.

<p align="center">
  <a href=".scratch/offline-supervised-migration/implementation.md"><img alt="Status: in development" src="https://img.shields.io/badge/Status-In%20development-C4A7E7?style=for-the-badge&amp;logo=github&amp;labelColor=493267"></a>
  <a href="docs/development.md"><img alt="First release target: Windows" src="https://img.shields.io/badge/Target-Windows-B5DFE8?style=for-the-badge&amp;labelColor=29626B"></a>
  <a href="docs/ownership-explained.md"><img alt="Analysis runs offline" src="https://img.shields.io/badge/Analysis-Offline-A2FFB0?style=for-the-badge&amp;labelColor=246732"></a>
  <br>
  <a href="Cargo.toml"><img alt="Built with Rust" src="https://img.shields.io/badge/Built%20with-Rust-F4C98B?style=for-the-badge&amp;logo=rust&amp;logoColor=F4C98B&amp;labelColor=75512B"></a>
  <a href="Cargo.toml"><img alt="Desktop GUI: Iced 0.14" src="https://img.shields.io/badge/GUI-Iced%200.14-B5DFE8?style=for-the-badge&amp;labelColor=29626B"></a>
</p>

[Usage guide](docs/editor-workflow.md) · [How ownership and bleed work](docs/ownership-explained.md) · [Build from source](docs/development.md) · [Development status](.scratch/offline-supervised-migration/implementation.md)

## Try it

CleanSlate is in development, with Windows as the first release target. Start with the [build guide](docs/development.md) to build and open the editor from this checkout.

```mermaid
flowchart LR
    tracks["Import<br/>Aligned tracks"]
    examples["Label<br/>Solo speech + room tone"]
    analyze["Analyze"]
    listen["Listen<br/>Adjust gain"]
    save["Save project<br/>Render WAVs"]
    tracks --> examples --> analyze --> listen --> save
    listen -->|"Relabel"| analyze
```

The steps below describe the same workflow. Tracks must already be synchronized; CleanSlate does not align them for you.

1. Import synchronized mono tracks, one dedicated microphone per speaker.
2. Listen in Original mode. Drag across a waveform and select everyone speaking; select nobody for room tone. Supply a solo example for each speaker and a room-tone example.
3. Run Analyze. The learned models evaluate the whole recording, including sections you have not labeled.
4. Compare Original, Processed and Removed. Adjust attenuation, gap duration and fades. Add labels or manual gain edits where needed.
5. Save your editable project and create individual processed WAVs when the result sounds right.

See the [usage guide](docs/editor-workflow.md) for editing, reanalysis and export details.

## What it does

CleanSlate was created to make podcast microphone bleed reduction practical offline, using the separate microphone tracks and human examples to guide the algorithm. The goal is to reduce unwanted leakage while preserving each speaker's direct speech and natural overlap.

- Learns each recording's speech, room-noise and directed microphone-bleed relationships from human examples.
- Estimates ownership independently for each track, allowing multiple people to speak at once.
- Provides synchronized waveform navigation, gain controls and protected manual edits that survive reanalysis.
- Keeps source files unchanged and uses the same effective gain envelope for display, audition and export.

Automatic attenuation uses S-shaped fades: gain starts changing gently, moves fastest in the middle, then settles gently at the exact target. Open and close controls set the maximum fade durations. The fades fit inside non-speaking gaps and reach full volume before preserved speech. Short automatic gaps can be ignored with the gap control.

Manual gain points use straight lines between points and remain authoritative after reanalysis. See the [gain-envelope contract](docs/effective-envelopes.md) for how these layers combine. The aim is transparent attenuation without audible clicks, pumping or obvious ducking; confirm the result by listening. Existing renders made with the previous exponential fades need to be regenerated.

## Defaults and limits

Acoustic-first analysis is the default and the current development focus. It runs locally and needs no Nemotron download.

Supply aligned tracks; CleanSlate does not automatically synchronize them. Ownership scores are model estimates, not guaranteed accuracy. Review the result by listening, especially to Removed audio. Independent quality, listening and eight-speaker/two-hour Windows release qualification remain pending.

## Work in progress

Nemotron-assisted analysis is experimental and disabled by default. An implementation exists, but its benefit over acoustic-only analysis has not been established. Comparative evaluation must check wanted-speech preservation, bleed reduction, listening quality and processing cost before deciding whether to promote it as a supported feature. See the [evaluation tracker](.scratch/offline-supervised-editor/issues/14-independent-release-evaluation.md).

Developer setup remains documented in the [experimental runtime guide](docs/development.md#experimental-nemotron-runtime), with [model and runtime notices](docs/runtime-notices.md).

## Documentation and development

### How this project is built

All coding for this project is done with AI agents using GPT Astra and GPT 6.1 Sol. Development follows Matt Pocock's workflow, with specifications, scoped tickets, implementation and review. Human feedback guides the GUI and checks whether the attenuation sounds right.

### Technology

| Component | Technology |
| --- | --- |
| Application and desktop GUI | Rust and Iced |
| Session ownership learning | Linfa logistic regression |
| Fixed speech detection | Silero VAD through ONNX Runtime |
| Audio decoding and analysis resampling | Symphonia and Rubato |
| Playback | Rodio |
| Experimental speaker activity | Nemotron through the native NeMo-Speech.cpp runtime |

### Guides

- [Ownership architecture](docs/ownership-explained.md): labels, acoustic inputs, bleed calibration, weights, logistic regression and gain decisions, with a Mermaid diagram.
- [Developer guide](docs/development.md): builds, checks, optional native runtime and packaging.
- [GUI development](docs/gui-development.md): repeatable build-and-launch workflow using a separate project copy.
- [Technical model recipe](docs/supervised-ownership-models.md) and [acoustic evidence](docs/acoustic-evidence.md).
- [Domain glossary](CONTEXT.md), [architecture decision](docs/adr/0006-offline-supervised-editable-envelopes.md) and [active roadmap](.scratch/offline-supervised-migration/map.md).

For development tasks and known acceptance gaps, consult the [implementation tracker](.scratch/offline-supervised-migration/implementation.md). Read [AGENTS.md](AGENTS.md) before changing the project.
