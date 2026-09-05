# CleanSlate

CleanSlate is a local, non-destructive automixer for synchronized mono podcast stems. Its language favors preservation when evidence is ambiguous.

## Language

**Session**:
One or more synchronized 48 kHz mono WAV stems imported for a single automix. Every stem has the same sample count. Matching BEXT time references establish alignment when present on every stem; absent BEXT is allowed with an unverified-alignment warning, while mixed or conflicting BEXT blocks rendering.
_Avoid_: Multitrack file, two-track minimum

**Project**:
The durable record of a session's source identities, protections, review states, and current or stale render identity. It autosaves review decisions; moved or changed sources must be relinked and revalidated.
_Avoid_: Render report, audio source

**Release acceptance gate**:
The combined automated and listening evidence required before the MVP is releasable: preservation ranges and intended overlap remain at unity, bleed improves measurably, outputs preserve structure and integrity, transitions pass listening review, and the full reference session completes safely.
_Avoid_: Python parity, detector benchmark

**Ownership decision**:
A categorical analysis result: `Owned(track set)`, `Uncertain`, or `NoSpeech`. It may carry short evidence reasons but never a numeric confidence.
_Avoid_: Confidence score, winner

**Attenuation region**:
A contiguous span sharing one applied attenuation decision, presented as the primary unit of inspection with its timing, affected track, attenuation, ownership decision, evidence reasons, and warning.
_Avoid_: Frame, envelope segment

**Preserved region**:
A timeline span that CleanSlate will keep at unity because the track owns speech. Its editor overlay is green.
_Avoid_: Active microphone, microphone on

**Attenuated region**:
A timeline span whose gain CleanSlate will reduce because another track owns speech or the ownership decision is `NoSpeech`. Its editor overlay is red.
_Avoid_: Inactive microphone, microphone off

**Uncertain region**:
A timeline span kept at unity because ownership evidence is ambiguous. Its editor overlay is amber and it remains eligible for review.
_Avoid_: Low-confidence region

**Review warning**:
A suspicious but safe result that renders conservatively and is promoted for inspection.
_Avoid_: Error

**Review list**:
The region-grouped collection of review warnings. Selecting an item seeks playback to its region and highlights the affected tracks.
_Avoid_: Error list

**Render blocker**:
An unsafe input or output condition that prevents rendering, such as misalignment, unsupported media, invalid samples, or failed output validation.
_Avoid_: Warning

**Protection**:
A persisted, non-destructive project decision that forces unity gain for an entire track or a selected time range across all tracks. Protections combine additively and always take precedence over attenuation.
_Avoid_: Rejection, envelope edit, owner reassignment

**Protect this region**:
A one-click action that creates a time-range protection using an attenuation region's exact bounds.
_Avoid_: Reject decision

**Original/processed comparison**:
A playback toggle that preserves the playhead and selected range while switching between source audio and the current gain envelopes applied live. It does not require rendered files.
_Avoid_: Render comparison, exported-file comparison

**Removed signal**:
The original signal minus the processed signal, auditioned to reveal what CleanSlate removes. “Difference” may appear only as supporting technical language.
_Avoid_: Difference mode, delta signal

**Monitoring gain**:
An identical, monitor-only headroom adjustment applied across Original, Processed, and Removed audition modes to prevent multitrack playback clipping. It never changes rendered or exported files.
_Avoid_: Normalization, render gain

**Stale render**:
A previously created deliverable that no longer reflects the project's current parameters or protections. It never prevents live Processed or Removed audition, but it cannot be treated as the current export.
_Avoid_: Invalid source, stale preview

**Editor workspace**:
The single persistent, synchronized multitrack workspace for importing, analyzing, adjusting, auditioning, protecting, rendering, and exporting a session. Each stem has a conventional amplitude waveform on one shared timeline.
_Avoid_: Review workspace, workflow rail, staged wizard, restoration suite

**Analysis preview**:
The current ownership overlays and parameter-derived gain curves shown without rendering audio. Parameter changes update this preview while making any existing final render stale.
_Avoid_: Export render

**Final render**:
The deliberate creation and validation of deliverable processed WAV files after the editor looks and sounds acceptable. It is not part of ordinary analysis or audition.
_Avoid_: Preview render, playback cache

**Gain-envelope preview**:
The read-only gain curve derived from current automix parameters and protections, drawn over a track waveform. It is not arbitrary DAW automation.
_Avoid_: Automation lane, editable fade

**Processed waveform**:
The source waveform reshaped by the current live gain envelope. It appears brightly over a faint Original waveform so parameter effects are visible without losing the source reference.
_Avoid_: Rendered waveform, replacement source

**Waveform cache**:
A persisted summary used to draw a source stem quickly without changing or replacing its audio. An incomplete cache is shown as loading, never as silence.
_Avoid_: Rendered audio, proxy audio

**Speech sensitivity**:
The user-facing control for how readily model evidence counts as speech. Higher sensitivity admits weaker speech evidence.
_Avoid_: Speech-probability threshold

**Bleed discrimination**:
The user-facing control for how strongly a track must rise above its own noise floor before it can own speech.
_Avoid_: Energy margin, gate threshold

**Speech padding**:
The preserved time before and after detected speech that prevents word boundaries from being attenuated.
_Avoid_: Preservation frames, preroll

**Open time**:
The one-pole time constant used when gain chases the preserved target. It controls recovery toward unity without defining a finite linear fade duration.
_Avoid_: Fade-in duration, transition samples

**Close time**:
The one-pole time constant used when gain chases the attenuated target. It controls reduction toward the attenuation floor without defining a finite linear fade duration.
_Avoid_: Fade-out duration, transition samples

**One-pole gain transition**:
The exponential target-chasing behavior shared by preview, audition, and final render. Gain moves by a fixed fraction of its remaining distance each analysis step and retargets from its current value.
_Avoid_: Linear ramp, fixed transition phases, fade automation

**Review state**:
The persisted reviewed or unreviewed status of a review warning; reviewed warnings remain visible and reopenable.
_Avoid_: Dismissed warning
