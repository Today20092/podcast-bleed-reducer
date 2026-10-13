CleanSlate v0.1.0-alpha.5 — editor workflow, progressive imports and faster playback.

The alpha.3 candidate did not publish because its Windows headless tests crashed
in audio-device enumeration. Alpha.4 corrects that test/preparation path while
preserving production device-rate validation and the complete test suite.
The alpha.4 candidate also exposed a concurrent cache-publication race on Linux.
Alpha.5 accepts a competing publisher only after verifying its complete matching
artifact; invalid cache data and ordinary errors still block preparation.

Changes since the last published desktop release, v0.1.0-alpha.2:

## Added

- Original-audio selection loops while labeling speakers, with pause/resume, Stop,
  and restoration of the previous audition state after labeling.
- Typed sound-setting values, exact mute, and value reset gestures.
- A searchable/scannable keyboard shortcut reference and selection-fit navigation.
- An application menu with Settings, Help and About, including credits.
- Persistent dark and light appearance.
- Immediate provisional timeline rows during imports, overall stages and per-recording
  status, measured progress where available, and whole-recording rough previews that
  refine in the background.
- Queued imports, independent readiness for separate recordings, failed rows with
  retry/remove, individual cancellation and cancellation of all pending work.

## Improved

- Import setup is centered and the inactive gain editor is hidden.
- Multichannel import decoding shares recording work rather than decoding every
  selected channel independently. Validation and waveform work run in the background.
- Completed imports remain usable while other recordings load. Canceling pending work
  preserves completed recordings and edits. New tracks join playback on its next start.
- Ready-session playback reuses validated prepared audio, the output stream and transport.
  Repeated selections avoid full preparation and repeated device initialization.
- Shortcut help separates actions into readable rows. Redundant sound-value hover
  instructions were removed.

## Fixed

- Stop remains accessible in the speaker-label popup; paused playback is shown correctly.
- Shortcut-reference scrolling reserves room for its scrollbar; About credits fit the dialog.
- Playback completion cannot stop a newly requested selection before it is acknowledged.
- Seeking immediately after replacing a range uses the new bounds and preserves the newer seek.
- Short preparation/buffering states are debounced; long waits retain visible status.
- Failed output devices are detected and can be retried explicitly.

## Verification and limits

The Windows build passed 263 automated tests (18 ignored), strict Clippy and formatting.
The owner tested repeated playback, selections, labeling loops, seeking, import during
playback and import feedback and reported that everything was working well.
On the owner's Focusrite device, warm software first-sample readiness measured
18.91 ms median and 21.57 ms p95, compared with 89.42 ms median and 102.57 ms p95
for the previous stream-opening path. This is a software measurement, not physical
audible latency. The measured warm cycles had zero steady-state underruns.

Configurable keyboard shortcuts, S-to-save and zoom/playhead-lock preferences
remain planned in ticket 41 and are not included.
Physical device disconnect recovery, independent speech/overlap preservation and
long-session release qualification remain pending. This is an experimental alpha.

Choose the attached download for your system:

- Windows 11 x64: extract the entire ZIP and open `CleanSlate.exe`.
- macOS 13+, Apple Silicon or Intel: open the matching DMG and drag `CleanSlate.app`
  to Applications.
- Linux x64, glibc 2.35+: allow the AppImage to run as a program, then open it.
  Ubuntu 22.04 or newer is the build baseline. A desktop/audio system is required.

Only attached assets are available for a given version. Windows builds are unsigned;
Mac builds use ad-hoc signing and are not notarized. The application runs acoustic
analysis offline using human reference labels, embedded Silero VAD and learned
ownership models. It includes New project, gain-envelope editing and documentation.
Nemotron is removed from the application.

Your operating system may show a security warning. Keep original recordings and project backups.
Independent listening, wanted-speech/overlap preservation and long-session
qualification remain pending; this alpha makes no completed quality-gate claim.
Quiet overlap without representative labels can receive a low ownership estimate.
Listen carefully, label wanted speech/overlap and protect it before exporting.

See the included README, MIT license and third-party notices. Verify your download against
the matching SHA-256 file. For Linux without FUSE, use `--appimage-extract-and-run`.
Report problems through the repository's issue tracker.
