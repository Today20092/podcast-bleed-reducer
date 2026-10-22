CleanSlate @VERSION@ - @PLATFORM@ experimental alpha

Windows 11 x64: extract the entire ZIP to a writable folder and open CleanSlate.exe.
Keep its support DLLs beside the application.
macOS 13 or later: open the DMG and drag CleanSlate.app to Applications.
Use the Apple Silicon or Intel download that matches your Mac.
Linux x64: allow the AppImage to run as a program in its file properties, then open
it. You can also use chmod +x CleanSlate-*.AppImage and run ./CleanSlate-*.AppImage.
The Linux build targets Ubuntu 22.04 or newer with glibc 2.35 or newer. It requires
a working desktop/audio system. If FUSE is unavailable, run the AppImage with
--appimage-extract-and-run.
No installation, Python, model download or internet connection is required.
Acoustic analysis uses your reference labels and the embedded Silero VAD.

This alpha is experimental. Windows builds are unsigned; macOS builds use ad-hoc
signing and are not notarized. Your operating system may show a security warning.
Keep your original recordings and save project copies before trying it.
Listening quality and long-session qualification are still pending.
Quiet unlabeled overlap can receive a low ownership estimate. Review and label
wanted speech/overlap or protect it before exporting.

Import aligned recordings, label room tone and each speaker's solo speech,
Analyze, review the gain envelopes, audition, then export to a new folder.
New project starts an empty session; save your current project first.

Use the Timeline waveform icons to show Original, Processed or Both independently
of the audio you audition. Catch up follows playback at the right edge and stays
enabled when you navigate; Shift+C toggles it. Lock follows continuously.
Menu > Settings lets you change appearance and customize keyboard shortcuts.
Hover over command buttons for their current bindings. Details beside Ready shows
analysis stage timings. The online usage guide explains editing and export:
https://github.com/Today20092/podcast-bleed-reducer/blob/main/docs/editor-workflow.md

Application source: https://github.com/Today20092/podcast-bleed-reducer
Application license: LICENSE (MIT). Dependency notices: THIRD-PARTY-NOTICES.txt.
Model and runtime license texts are included in licenses/.
