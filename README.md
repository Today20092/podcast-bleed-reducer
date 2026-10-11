# CleanSlate

CleanSlate is an offline microphone bleed-reduction tool for multitrack podcasts. It uses a supervised algorithm to learn from a few examples of who is speaking, then attenuate each microphone when its assigned person is silent. Listen to the result and adjust the gain where needed.

<p align="center">
  <a href=".scratch/offline-supervised-migration/implementation.md"><img alt="Status: in development" src="https://img.shields.io/badge/Status-In%20development-C4A7E7?style=for-the-badge&amp;logo=github&amp;labelColor=493267"></a>
  <a href="#download"><img alt="Release targets: Windows, macOS and Linux" src="https://img.shields.io/badge/Targets-Windows%20%7C%20macOS%20%7C%20Linux-B5DFE8?style=for-the-badge&amp;labelColor=29626B"></a>
  <a href="docs/ownership-explained.md"><img alt="Analysis runs offline" src="https://img.shields.io/badge/Analysis-Offline-A2FFB0?style=for-the-badge&amp;labelColor=246732"></a>
  <br>
  <a href="Cargo.toml"><img alt="Built with Rust" src="https://img.shields.io/badge/Built%20with-Rust-F4C98B?style=for-the-badge&amp;logo=rust&amp;logoColor=F4C98B&amp;labelColor=75512B"></a>
  <a href="Cargo.toml"><img alt="Desktop GUI: Iced 0.14" src="https://img.shields.io/badge/GUI-Iced%200.14-B5DFE8?style=for-the-badge&amp;labelColor=29626B"></a>
</p>

[Download](#download) · [Usage guide](docs/editor-workflow.md) · [How it works](docs/ownership-explained.md) · [Roadmap](ROADMAP.md) · [Contribute](CONTRIBUTING.md) · [Report an issue](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose)

## Download

Choose your platform below. CleanSlate runs locally without Python, a model download or an internet connection. Links appear here when packages are published, including alpha prereleases. Draft releases are not public downloads.

<!-- downloads:start -->
| Platform | Download | How to run |
| --- | --- | --- |
| Windows 11, 64-bit | [Download v0.1.0-alpha.2](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-windows-x64.zip) · [SHA-256](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-windows-x64.zip.sha256) | Extract the entire ZIP, then open `CleanSlate.exe`. Keep the support files together. |
| Mac, Apple Silicon, macOS 13+ | [Download v0.1.0-alpha.2](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-macos-arm64.dmg) · [SHA-256](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-macos-arm64.dmg.sha256) | Open the DMG and drag `CleanSlate.app` to Applications. |
| Mac, Intel, macOS 13+ | [Download v0.1.0-alpha.2](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-macos-x64.dmg) · [SHA-256](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-macos-x64.dmg.sha256) | Open the DMG and drag `CleanSlate.app` to Applications. |
| Linux, 64-bit x86, glibc 2.35+ | [Download v0.1.0-alpha.2](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-linux-x64.AppImage) · [SHA-256](https://github.com/Today20092/podcast-bleed-reducer/releases/download/v0.1.0-alpha.2/CleanSlate-0.1.0-alpha.2-linux-x64.AppImage.sha256) | Allow the AppImage to run as a program in its file properties, then open it. |
<!-- downloads:end -->

On a Mac, **Apple menu → About This Mac** shows either an Apple chip or an Intel processor. Linux packages target Ubuntu 22.04 or newer and require a working desktop and audio system. If your system cannot mount an AppImage, run it with `--appimage-extract-and-run`.

These are experimental alpha packages. Windows builds are unsigned. Mac builds use ad-hoc signing and are not notarized, so the operating system may require approval to open them. Listening quality and long-session qualification remain pending. See the [release guide](docs/releasing.md) for checksums and package checks.

## Try it

Download the experimental application for your platform [above](#download), then open it using the steps in the table. To build from source, follow the [build guide](docs/development.md).

[![From aligned tracks to processed WAVs.](docs/diagrams/workflow.svg)](docs/diagrams/workflow.svg)

[Accessible diagram and text equivalent](docs/diagrams/workflow.html).

The steps below describe the same workflow. Tracks must already be synchronized; CleanSlate does not align them for you.

1. Import synchronized mono tracks or selected channels from a multichannel WAV, one dedicated microphone per speaker.
2. Listen in Original mode. Drag across a waveform and select everyone speaking; select nobody for room tone. Supply a solo example for each speaker and a room-tone example.
3. Run Analyze. The learned models evaluate the whole recording, including sections you have not labeled.
4. Compare Original, Processed and Removed. Adjust attenuation, gap duration and fades. Add labels or manual gain edits where needed.
5. Save your editable project and export processed WAVs when the result sounds right. Choose separate mono files or one WAV with separate channels when the source rates and timing are compatible.

See the [usage guide](docs/editor-workflow.md) for editing, reanalysis and export details.

![CleanSlate development editor with three microphone tracks, gain-reduction regions and fades, per-track speaker estimates, sound controls and Original, Processed and Removed listening buttons.](docs/images/editor-overview.png)

The development editor after Analyze. Each lane is a microphone track. The gain curve and shaded regions show volume reduction; the percentages estimate whether each track's speaker is active at the playhead. Use **Speaker examples** to teach who is talking, **Sound** to adjust attenuation and fades, and **Original / Processed / Removed** to compare by listening. Recording filenames are redacted. This screenshot illustrates the controls, not an accepted quality result.

## What it does

CleanSlate was created to make podcast microphone bleed reduction practical offline, using the separate microphone tracks and human examples to guide the algorithm. The goal is to reduce unwanted leakage while preserving each speaker's direct speech and natural overlap.

- Learns each recording's speech, room-noise and directed microphone-bleed relationships from human examples.
- Estimates ownership independently for each track, allowing multiple people to speak at once.
- Provides synchronized waveform navigation, gain controls and protected manual edits that survive reanalysis.
- Keeps source files unchanged and uses the same effective gain envelope for display, audition and export.

Automatic attenuation uses S-shaped fades: gain starts changing gently, moves fastest in the middle, then settles gently at the exact target. Open and close controls set the maximum fade durations. The fades fit inside non-speaking gaps and reach full volume before preserved speech. Short automatic gaps can be ignored with the gap control.

Manual gain points use straight lines between points and remain authoritative after reanalysis. See the [gain-envelope contract](docs/effective-envelopes.md) for how these layers combine. The aim is transparent attenuation without audible clicks, pumping or obvious ducking; confirm the result by listening. Existing renders made with the previous exponential fades need to be regenerated.

## How it works

Speech detection asks whether a microphone contains speech. Ownership asks whether its assigned person is speaking, rather than another person's voice entering that microphone as bleed.

CleanSlate prepares 16 kHz analysis audio, measures speech and microphone relationships in 32 ms frames, and trains one logistic regression model per track from your reference labels. Each model combines those measurements with learned weights to estimate local ownership. Thresholds turn estimates into decisions over time; your labels specify the speaker set inside their exact ranges. Gain settings and manual edits then produce the curve used for audition and export.

Read [How CleanSlate reduces microphone bleed](docs/ownership-explained.md) for the operation order, model inputs, how weights are learned, overlap and practical limits.

## Defaults and limits

Analysis uses acoustic measurements and your speaker examples. It runs locally; there is no Nemotron runtime, download or analysis option.

Supply aligned tracks; CleanSlate does not automatically synchronize them. Ownership scores are model estimates, not guaranteed accuracy. Review the result by listening, especially to Removed audio. Independent quality, listening and eight-speaker/two-hour Windows release qualification remain pending.

The alpha remains experimental while independent quality, listening and long-session checks are pending. See the [evaluation tracker](.scratch/offline-supervised-editor/issues/14-independent-release-evaluation.md). The earlier Nemotron experiment is retained as a [research idea](docs/research/nemotron.md).

## Documentation and development

### Why I made CleanSlate

I make many podcasts and wanted a local tool I could use without paying a recurring automixing fee. After recording, I wanted to import my synchronized microphone tracks, check them by listening, and reduce room bleed when each person wasn't speaking.

I used FabFilter Pro-G, but on my recordings the level difference between a person's voice and bleed from other speakers was sometimes too small to set a reliable gate threshold. I needed something smarter about who was speaking, so it could attenuate inactive microphones while preserving wanted speech. CleanSlate is my attempt to make that first pass useful, with listening and manual gain edits before export.

The focus is podcast bleed reduction. Finish other production work in your usual audio editor. See the [roadmap](ROADMAP.md) for current priorities and future possibilities.

### How this project is built

All coding for this project so far has been done with AI agents using GPT Astra and GPT 6.1 Sol. Development follows Matt Pocock's workflow, with specifications, scoped tickets, implementation and review. Human feedback guides the GUI and checks whether the attenuation sounds right. The [contributing guide](CONTRIBUTING.md) explains our workflow and links to [Matt's agent skills](https://github.com/mattpocock/skills). The skills are recommended for agent-assisted work; human-written contributions are welcome.

### Help shape the project

Report bugs, confusing controls, user-flow problems, listening results or feature ideas through [GitHub Issues](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose). The forms help describe the problem consistently. You do not need to write code to contribute. Review screenshots and logs for private information before posting; share recordings only with permission from everyone recorded.

### Technology

| Component | Technology |
| --- | --- |
| Application and desktop GUI | Rust and Iced |
| Session ownership learning | Linfa logistic regression |
| Fixed speech detection | Silero VAD through ONNX Runtime |
| Audio decoding and analysis resampling | Symphonia and Rubato |
| Playback | Rodio |

### Guides

- [Contributing](CONTRIBUTING.md): feedback, development workflow, optional AI skills and pull requests.
- [Roadmap](ROADMAP.md): current priorities and ideas that need investigation.

- [How it works](docs/ownership-explained.md): speech detection, operation order, model inputs, learned weights and gain decisions, with an illustrated workflow.
- [Developer guide](docs/development.md): builds and checks.
- [Release guide](docs/releasing.md): Windows ZIPs, Mac application bundles, Linux AppImages, package checks and draft releases.
- [GUI development](docs/gui-development.md): repeatable build-and-launch workflow using a separate project copy.
- [Technical model recipe](docs/supervised-ownership-models.md) and [acoustic evidence](docs/acoustic-evidence.md).
- [Domain glossary](CONTEXT.md), [architecture decision](docs/adr/0006-offline-supervised-editable-envelopes.md) and [active roadmap](.scratch/offline-supervised-migration/map.md).

For development tasks and known acceptance gaps, consult the [implementation tracker](.scratch/offline-supervised-migration/implementation.md). Read [AGENTS.md](AGENTS.md) before changing the project.
