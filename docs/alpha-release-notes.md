# CleanSlate v0.1.0-alpha.8

This release brings the reviewed editor controls, customizable shortcuts and performance work into one desktop update. It also makes read-only recording media usable without writing beside the originals. It remains an experimental alpha.

Alpha.7 passed source checks on all four platforms but did not produce a draft: packaging correctly stopped when the zune-core/zune-jpeg crate archives omitted license texts. Alpha.8 includes the identical upstream Zlib notices from those exact dependency revisions. The editor behavior and captured interface are unchanged.

Changes below are relative to [v0.1.0-alpha.6](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.6). See the [usage guide](editor-workflow.md) for the complete workflow.

## Editor and playback

- A grouped inspector keeps status, speaker examples, attenuation, transitions and export easier to scan. Completed analysis shows Ready; Details opens an aligned timing breakdown.
- Attenuation and timing dials support vertical drag, fine adjustment, wheel input, arrow adjustment, reset and numeric entry. Indicators animate briefly between values while audio values change immediately.
- Fixed-size icon transport buttons stay centered. Toolbar outlines, selected states, keyboard focus and delayed command hints make navigation more consistent. Original, Processed and Removed listening modes share one selector.
- Three waveform buttons show Original, Processed or Both without changing the audio audition mode, gain curves, saved project or export.
- Catch up follows playback with a short leftward scroll at the timeline's right edge, then holds the view still until the next crossing. It stays enabled through manual navigation. Use its button or the customizable Shift+C shortcut to toggle it.
- Lock follows the playhead at its chosen screen position. Middle-drag temporarily suspends locked following and reanchors it on release. Center remains a one-shot navigation command.
- Context-aware shortcuts can be assigned, cleared or restored in Settings and persist independently of projects. Speaker examples save with S, while Enter remains available. Center, Lock and Catch up are keyboard-accessible. Effective bindings appear in hints and the shortcut reference.

## Recording media and feedback

- Open recordings directly from a readable SD card or another drive. Originals are read-only inputs; working projects, analysis copies and private processed audio use local application storage.
- Export into a new named folder inside the chosen destination. Existing files and original recordings are not replaced. Transient source-read errors receive bounded retries; changed or unreadable sources still block accepted output.
- Analysis reports preparation and speaker-detection progress within the workspace. Cancellation and failures retain actionable feedback.
- About's Copy version info reports the observed graphics backend, adapter and window scale for support. Unsupported display-refresh information stays explicitly unavailable. Project and audio paths are excluded from that report.

## Performance and reliability

- Unchanged background polls retain waveform and gain drawing caches. Actual source, edit, viewport, appearance and size changes still invalidate the relevant drawing.
- Single-track gain drafts reuse unaffected tracks' prepared envelopes. In the dense synthetic two-hour component fixture, draft p95 fell from 43.496 ms to 22.396 ms. This does not measure full-session analysis, import, export or physical response.
- Exact flat gain spans omit redundant drawing vertices. Two interleaved 2176-column runs reduced complete waveform-plus-gain preparation medians by about 1.35% and 1.9%; tails varied, and the wider 4096-column workload still missed its diagnostic budget.
- Confirmed minimization suspends optional visual work while playback and background jobs continue. Restoring the window refreshes current state.
- Playback retires decoded buffers and replaced envelopes outside device-sample iteration, including Stop to Play restart. Shared waveform construction keeps import and reopening on the same cache-building path.

The [performance summary](research/alpha7-performance-summary-2026-10-22.md) records workloads, source revisions, before/after values and limits. These changes make no universal FPS, physical audio-latency or inference-speed claim.

## Verification and limits

The development source at `6160da9` passed 353 tests with zero failures and 20 existing opt-in checks ignored, plus formatting, strict all-target/all-feature Clippy and script fixtures. Native Windows checks covered compact and wide layouts, Dark and Light appearance, analysis, repeated catch-up scrolling and shortcut activation. The owner accepted the development application for release preparation on 2026-10-22.

Final alpha.8 package checks, immutable tag/build identity and downloaded-package verification are recorded by the release preparation task before publication. Development checks alone do not establish fresh native qualification on every platform.

Quiet overlap without representative labels can receive a low ownership estimate. Listen to wanted speech and overlap, add representative examples and use protection or manual gain edits before exporting. Independent sound-quality comparisons, physical device-disconnect recovery and long-session qualification are not claimed complete. Keep original recordings and project backups.

The future menu redesign and automatic registration of new actions for customizable shortcuts remain separate roadmap work. The registration plan does not make every current button remappable.

## Downloads and installation

Use only assets attached to the published version and verify each download against its matching SHA-256 file.

- Windows 11 x64: extract the entire ZIP and open `CleanSlate.exe`.
- macOS 13 or newer: choose the Apple Silicon or Intel DMG and drag `CleanSlate.app` to Applications.
- Linux x64 with glibc 2.35 or newer: allow the AppImage to run as a program, then open it. Ubuntu 22.04 is the build baseline. Without FUSE, use `--appimage-extract-and-run`.

Windows packages are unsigned. Mac packages use ad-hoc signing and are not notarized, so the operating system may show a security warning. Package availability and native verification are version-specific; an attached experimental package is not a claim of fresh hardware testing.

Analysis runs offline with human reference labels, embedded Silero VAD and learned ownership models. Python, a separate model download and internet access are not required for analysis. Packages include the README, MIT license and third-party notices. [Report a problem](https://github.com/Today20092/podcast-bleed-reducer/issues/new/choose) with the version and relevant error text.

---

# Historical release notes: v0.1.0-alpha.6

The following section records the earlier release and its verification. Its planned-work statements describe alpha.6, not alpha.8.

CleanSlate v0.1.0-alpha.6: application logo, editor workflow and keyboard shortcuts.

This release brings the approved CleanSlate logo into the Windows executable and
window/taskbar icon, macOS application bundle and Linux desktop package. It also
publishes the newer editor workflow that was absent from the public alpha.2 download.

The alpha.3 candidate did not publish because its Windows headless tests crashed
in audio-device enumeration. Alpha.4 corrects that test/preparation path while
preserving production device-rate validation and the complete test suite.
The alpha.4 candidate also exposed a concurrent cache-publication race on Linux.
Alpha.5 accepts a competing publisher only after verifying its complete matching
artifact; invalid cache data and ordinary errors still block preparation.
Native render failure-injection tests also allow cold preparation time on loaded
CI runners without busy-spinning; failure and cancellation assertions are retained.

Changes since the last published desktop release, v0.1.0-alpha.2:

## Added

- Approved CleanSlate application icons and theme-aware README branding.
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

All four native platform test suites, strict Clippy, formatting and packaged
offline self-checks passed in
[the alpha.6 release run](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39447930397).
Packages were built from the immutable v0.1.0-alpha.6 tag at commit
9b2fa9dd8886aed07cc57d4d105ca9fb1bf2fd65, with clean build manifests.
The downloaded Windows package passed checksum, inventory and offline self-checks.
Its executable icon exactly matches the approved source icon; its embedded window
icon pixels, keyboard-shortcut reference and compiled alpha.6 version were verified.
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
