# CleanSlate

CleanSlate is a local, non-destructive automixer for synchronized mono podcast stems. Its language favors preservation when evidence is ambiguous.

These terms describe the accepted migration target. Historical code may still follow earlier contracts; see [ADR-0006](docs/adr/0006-offline-supervised-editable-envelopes.md) and the [decision map](.scratch/offline-supervised-migration/map.md).

## Language

**Session**:
One to eight synchronized mono speaker tracks imported for a single automix, with one dedicated track per speaker. Tracks may have different native rates and supported formats; synchronization is a shared physical timeline rather than equal sample counts.
_Avoid_: Multitrack file, two-track minimum

**Project**:
The durable record of a session's source identities, speaker assignments, reference labels, manual edits, protections, review states and current or stale results. Moved or changed sources must be relinked and revalidated before bound decisions apply.
_Avoid_: Render report, audio source

**Release acceptance gate**:
The combined independent automated and listening evidence required before release: wanted speech and overlap remain at unity absent intentional edits, bleed improves measurably, outputs preserve integrity, transitions pass listening review and full-session operation completes safely.
_Avoid_: Python parity, detector benchmark

**Ownership decision**:
A timeline conclusion identifying the locally speaking track set, no speech or uncertainty. It is distinct from the underlying probability estimates and can retain multiple simultaneous owners.
_Avoid_: Single winner, VAD probability

**Speaker activity**:
Evidence about which person is speaking at a time, regardless of which microphones hear their voice.
_Avoid_: Microphone ownership, gain target

**Local ownership**:
The presence of a track's assigned person speaking directly into that track's microphone. That person's voice on another track as bleed does not make the other track an owner.
_Avoid_: Speech presence, loudest microphone

**Ownership estimate**:
A probability estimate that a track's assigned person is speaking locally. Estimates are independent across tracks, so overlap is allowed; an estimate is not a guarantee of calibrated confidence.
_Avoid_: VAD score, speaker winner

**Reference label**:
A trusted, source-bound time interval identifying an active speaker set or room tone. Solo and overlap references describe known dialogue; room tone describes background without speakers.
_Avoid_: Predicted label, gain edit

**Session calibration**:
Knowledge of the current recording's noise and microphone relationships learned from trusted references. Multiple references can describe small changes during one recording.
_Avoid_: Global model training, neural fine-tuning

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
A persisted manual preserve constraint that forces unity gain for a track or selected time range instead of automatic attenuation. Conflicting manual constraints require explicit replacement.
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
The completed ownership overlays and effective gain curves shown without rendering audio. Label/source changes leave it visibly stale until Analyze completes, while gain edits update it immediately.
_Avoid_: Export render

**Final render**:
The deliberate creation and validation of deliverable processed WAV files after the editor looks and sounds acceptable. It is not part of ordinary analysis or audition.
_Avoid_: Preview render, playback cache

**Gain-envelope preview**:
The displayed effective gain curve over a track waveform, with automatic gain and protected manual edits distinguishable. It can be edited without changing source audio.
_Avoid_: Rendered audio, source waveform edit

**Automatic envelope**:
The attenuation curve generated from ownership decisions and gain settings before manual constraints apply.
_Avoid_: Final sound, manual automation

**Manual gain edit**:
A persisted source-bound gain decision that remains authoritative until removed, even when later labels or analysis disagree.
_Avoid_: Training label, temporary preview adjustment

**Effective envelope**:
The final gain curve after automatic gain and manual constraints are combined. It is the common decision used for preview, audition and rendering.
_Avoid_: Detector probability, alternate preview curve

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

**Fade-in duration**:
The explicit duration of the exponential-shaped recovery toward preserved gain, ending at the target before preserved speech.
_Avoid_: Asymptotic Open time, speech onset detector

**Fade-out duration**:
The explicit duration of the exponential-shaped reduction toward attenuated gain outside preserved speech.
_Avoid_: Asymptotic Close time, silence classifier

**Exponential-shaped fade**:
A curved transition with an explicit start, end and exact target gain. It retains an exponential feel without an indefinitely approaching endpoint.
_Avoid_: Linear-only fade, one-pole time constant

**Review state**:
The persisted reviewed or unreviewed status of a review warning; reviewed warnings remain visible and reopenable.
_Avoid_: Dismissed warning
