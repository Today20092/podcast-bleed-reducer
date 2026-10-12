# Enter custom sound values and reset parameter defaults

Status: done
Type: task
Assignee: Codex sound_entry_32
Blocked by: 02, 11, 13, 18, 28
Spec: ../../offline-supervised-migration/specs/sound-value-entry.md

## Scope

Implement the [accepted sound-entry specification](../../offline-supervised-migration/specs/sound-value-entry.md) through GUI entry, validation, project persistence, shared effective-envelope evaluation, live audition and export. Retain ordinary sliders; accept finite attenuation −120 to 0 dB plus explicit exact-silence Mute, and fade/gap values 0–60,000 ms. Add Enter/blur commit, Escape cancel, clear invalid drafts and Ctrl+click/context-menu reset of individual parameter defaults.

Dependencies are done. Coordinate shared GUI edits with ticket 23. If shortcut-reference ticket 34 has landed, update its entries; otherwise ensure ticket 34 can list the implemented bindings. Neither ordering requires rebuilding the other scope. Existing manual curve editing/reset-to-automatic remains outside this ticket.

## Acceptance checks

- [x] Typed values accept both boundaries, zero and valid interior values; reject nonfinite, malformed and out-of-range input without changing accepted settings.
- [x] Enter/blur commit valid input, Escape restores it, invalid blur retains editable feedback, and typing suppresses workspace shortcuts. Out-of-slider-range values remain stable until deliberate slider movement.
- [x] Mute is explicit in validated saved settings; old projects retain finite attenuation. Corrupt data is rejected and changed gain/render identities cannot reuse incompatible prepared delivery.
- [x] Default reset affects only the clicked parameter (−18 dB, 50 ms, 500 ms or 768 ms); tooltip/context action/shortcut guidance is discoverable and manual edits survive.
- [x] Zero fades are immediate; zero gap disables bridging; long values preserve speech, explicit room-tone truth, manual constraints and source timing. Gain changes reuse cached ownership without reinference.
- [x] Shared display/audition/removed/render checks distinguish finite −120 dB from exact-zero Mute. Integer and float exports preserve mute silence, including integer dither behavior and normal safe publication.
- [x] Behavioral setting, project round-trip, old-project and envelope/export tests pass for new values, mute, invalid inputs, protected speech and stale-delivery rejection.
- [x] Use the GUI development guide and scripts/gui-dev.ps1; run appropriate Rust checks, formatting and strict Clippy. Review editing, validation, Mute and reset gestures in the development GUI; record unavailable checks.

## Results

2026-10-12: Integrated `52b61f0b2c638375d6aa914fe391eafdd540939a` on `codex/editor-workflow-31-37`. Explicit default-false Mute retains finite attenuation and legacy JSON compatibility. Validation accepts finite attenuation −120 to 0 dB and times 0–60000 ms. Shared envelopes support zero fades/gap, preserve speech/manual constraints and distinguish exact zero from finite attenuation; recipe v5 invalidates incompatible prepared delivery. Drafts commit via existing settings on Enter, mouse/Tab/window blur, and cancel with Escape. Invalid blur retains editable feedback without blocking unrelated actions. Sliders retain their ordinary bounds and never change accepted values during view. Individual Ctrl-click/right-click reset is discoverable; attenuation reset exits Mute. No dependency or reinference path added.

TDD zero-fade regression failed on the old validator and passed after implementation. Scoped library/project/effective/native/render/supervised/export suite passed 135 tests with 7 opt-in tests ignored. Regressions cover finite boundaries, malformed/nonfinite input, project round trips, corrupt/legacy JSON, exact mute versus −120 dB, zero/long durations, protections, stale delivery, Original/Processed/Removed parity and safe PCM24/float separate/multichannel exports. All-target/all-feature compilation, strict Clippy, formatting and whitespace checks passed. Final `scripts/gui-dev.ps1 -BuildOnly` passed in 23.41 seconds.

Development GUI inspection on the separate gui-dev project confirmed Enter accepts −120 dB without clamping, Tab on 60001 ms retains visible validation feedback, Escape restores the accepted 501 ms value, Mute has distinct visible state, right-click reset restores −18 dB and exits Mute without changing other parameters, and clicking away commits zero fade time. Tooltips explain typing/reset gestures. The development copy now contains these test settings; original project/source audio were not changed.

Unavailable checks: physical listening, a complete narrow-window/keyboard-only pass and real Ctrl-click gesture. Ctrl-click and valid/invalid focus transitions have app-event coverage. Seven opt-in diagnostics/performance/full-owner-file checks were skipped. This changes no detector or stable release qualification.

## Comments

2026-10-11: Owner confirmed limits, explicit Mute and reset semantics and authorized specification/ticket creation. Expanded gap bounds deliberately supersede the prior 10–5000 ms contract for this extension.
