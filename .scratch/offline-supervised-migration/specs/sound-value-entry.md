# Typed sound values, mute and default reset

Status: accepted specification, 2026-10-11
Ideas: I07, I08
Build ticket: [32](../../offline-supervised-editor/issues/32-sound-value-entry.md)
Decision record: [owner interview](../../../docs/roadmap-ideas.md#interview-decisions-2026-10-11)

## Problem Statement

Sound sliders cannot express the desired attenuation or timing values, and there is no direct default-reset gesture.

## Solution

Retain sliders for ordinary adjustments and make displayed values editable. Offer explicit Mute, extended timing entry and discoverable parameter reset.

## User Stories

1. As an editor, I want to type attenuation from 0 to −120 dB so slider limits do not restrict my sound.
2. As an editor, I want explicit Mute so attenuated plateaus can be exactly silent.
3. As an editor, I want to enter fade and gap durations from 0 to 60,000 ms.
4. As an editor, I want zero fades to mean instant transitions and zero gap bridging to disable it.
5. As an editor, I want Enter or clicking elsewhere to apply a valid value and Escape to cancel.
6. As an editor, I want invalid text explained without changing the accepted value.
7. As an editor, I want Ctrl+click or a context action to restore a parameter's default.
8. As an editor, I want settings preserved on save/reopen and identical behavior in audition and export.
9. As an editor, I want manual edits and preserved speech to retain their existing meaning.

## Implementation Decisions

- Typed attenuation accepts finite values in [−120, 0] dB; typed fade and gap durations accept finite values in [0, 60000] ms. Reject malformed, nonfinite and out-of-range input with inline feedback. Display explicit units. Preserve existing numeric precision supported by the domain; do not add an arbitrary rounding rule.
- Retain current useful slider ranges (attenuation −30 to −3 dB; full-volume fade 1–1000 ms; reduced-volume fade 1–2000 ms; gap 10–5000 ms). An accepted typed value outside the slider range remains visible and unchanged by rendering the slider. Only deliberate slider movement adopts its value. Mute stays visibly distinct from a finite dB value.
- Clicking a displayed value begins an isolated text draft. Enter or focus loss commits valid text; Escape restores the previous accepted value. Invalid blur keeps an error and draft available for correction without blocking unrelated application actions. Typing suppresses playback/zoom/label shortcuts. Apply each committed change through the existing sound-setting path; no partial-keystroke sound updates.
- Represent Mute explicitly in validated project/settings data, not by an infinite dB number or an arbitrary finite floor. Use the smallest compatible representation, retaining the finite attenuation value for ordinary entry. Projects without the new choice load their existing finite attenuation unchanged. Validate mutually inconsistent or corrupt data; version/invalidate derived gain/render identities where their meaning changes.
- Mute targets exact zero where attenuation applies. S-shaped fades retain finite duration and exact endpoints; zero duration is immediate. Preserve human speech, uncertainty protections and protected manual constraints under the existing composition rules. Finite −120 dB remains a positive gain, distinct from Mute. Removed audition remains Original minus Processed. Integer export dither preserves exact mute silence.
- Extend domain gap validation from 10–5000 to 0–60000 ms. Zero disables automatic gap bridging. Long fades still fit inside available non-speaking gaps and never shorten preserved speech; gap bridging still cannot erase explicit room-tone truth or protected manual edits. Preserve native timing and sample grids.
- Gain-setting commits/reset regenerate effective envelopes from cached evidence without analysis reinference. Invalidate prepared delivery as needed and keep current-analysis/export safeguards. Update all settings load/save, headless validation, display, audition and rendering callers consistently.
- Ctrl+click a displayed parameter resets only it; right-click exposes Reset to default. Tooltip and shortcut reference explain both paths. Defaults remain attenuation −18 dB (not Mute), full-volume fade 50 ms, reduced-volume fade 500 ms, gap 768 ms. Reset uses the normal validated setting path and never discards manual gain edits.

## Testing Decisions

Reuse app-setting event tests, project round-trip tests and shared effective-envelope/render parity tests. Cover boundary values, invalid drafts, blur/Enter/Escape, slider rendering outside ordinary ranges, default reset, old-project defaults, explicit mute persistence and corrupt data. Verify exact zero mute plateaus in audition and integer/float exports, positive finite −120 dB, zero fades/gap, constrained long fades/gaps, protected speech/manual edits and stale delivery invalidation without reinference. Review keyboard/focus and context actions in the development GUI.

## Out of Scope

Amplification, arbitrary nonfinite numeric settings, changed parameter defaults, a master mute, new fade shapes, new manual-point controls or reset-to-automatic for manual edits.

## Further Notes

This supersedes the earlier gap-input bound only for this accepted extension. It does not change stable listening or release qualification.
