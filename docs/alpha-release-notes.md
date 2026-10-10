CleanSlate's experimental offline desktop alpha.

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
