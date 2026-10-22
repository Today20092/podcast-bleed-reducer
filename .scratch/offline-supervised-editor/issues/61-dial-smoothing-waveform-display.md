# Smooth dial movement and select original/processed waveform visibility

Status: done
Type: task
Assignee: Codex
Blocked by: 57
Labels: ready-for-agent

## Owner request

2026-10-22: dial wheel steps appear rigid. Animate the visual transition between committed values without delaying audio or changing step sizes. Provide Original, Processed and Both waveform display options to make attenuation easier to inspect.

## Scope and acceptance

- Reuse the shared dial implementation for attenuation and transition/gap controls. Animate the indicator over a short bounded interval, retarget smoothly during repeated wheel input and stop requesting redraws after settling. Preserve fine modifiers, direct typing, drag, reset, validation and immediate sound updates.
- Add a plainly labeled three-choice waveform display control, independent of audition mode. Default to Both and retain the selected display through ordinary editor operations. Original draws source peaks; Processed draws effective-envelope peaks; Both draws both. Without an effective envelope the processed view shows unity source peaks.
- Include the display mode in waveform cache identity, preserving shared source peaks, gain curves, labels, selection and audio/export behavior.
- Verify animation convergence/retargeting and waveform selection/cache invalidation with focused tests, run local checks and GUI build. Record visual checks and unavailable physical checks explicitly.

## Results

Claimed on development before implementation; dependency 57 is done and integrated. Menu research remains separate in ticket 60.

Implemented 110 ms cubic ease-out indicator motion in the shared dial canvas. Changes retarget from the interpolated position, publish no audio messages and request redraw only until the finite transition completes. Sound values and existing input steps/validation remain immediate.

The Timeline toolbar now provides Waveform: Original / Processed / Both. It defaults to Both for each application session and does not enter saved projects or affect audition mode. The mode participates in canvas geometry invalidation, leaving shared source waveform caches intact. Without an effective envelope, Processed draws the unity source. Gain curves, shading and speaker labels remain visible.

Final scripts/check.ps1 -Local passed: 343 tests, zero failures, 20 existing ignored across 38 targets, plus formatting, strict Clippy and script fixtures. New tests cover motion continuity/settling, redraws publishing no sound changes, all three drawn geometries, retained-versus-fresh cache parity, unity fallback and unchanged audition/parameters. Native build/check results follow below.

scripts/gui-dev.ps1 passed in 95.54 seconds. Native Dark rendering was inspected at 1028×930 and 2560×1392. The retained three-track development copy completed Analyze in its reported 23.9 seconds (not an isolated benchmark). Both, Original and Processed visibly select the intended waveform layers while the gain curve/shading remain; audition stayed Original. A native attenuation wheel step changed -11 to -10 dB and the inverse restored -11 dB. The app remains open with Processed waveform display for owner review. No project save/export was triggered during these checks.

Executable C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe, source development base 31d81db plus tickets 59/61 working changes, SHA-256 44665FE930570442F388E20CA7A435D44D12E3FA245461EBF44508569DE36E9B. Light/enlarged OS text, physical animation timing/feel and listening were not independently measured. The bounded interpolation/redraw behavior has deterministic test coverage; final tactile judgment remains with the owner.
