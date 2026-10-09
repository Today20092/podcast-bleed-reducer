# Supervised editor workflow

The Iced workspace combines validated sources, 16 kHz analysis copies, human references, acoustic evidence, independent per-track ownership models and shared effective gain envelopes. Acoustic-first analysis is the default and current development focus. See [how ownership and bleed work](ownership-explained.md) for the full explanation and diagram.

Import synchronized mono tracks, declare supplied synchronization when appropriate, and listen in Original mode to find solo speech and room tone. Click to seek; Space plays/stops. Drag across a waveform to open the speaker multiselect immediately. Select nobody for room tone, one person for solo speech, or several for overlap, then save the label. Supply a solo reference for each speaker and a room reference. Additional examples across the recording can improve calibration. Analyze validates reference usability; merely saving a label does not establish that it is usable. Speaker numbers are stable identities, not inferred names.

Leave Use Nemotron off for the acoustic workflow. Nemotron-assisted analysis is experimental work in progress; its benefit over acoustic-only analysis has not been established. Development setup and verification requirements are in the [experimental runtime guide](development.md#experimental-nemotron-runtime). When selected, missing or changed assets produce an actionable error and retain the previous preview; the editor does not silently switch modes.

## Analysis and editing

Analyze is explicit. Several label changes can be collected before one job. Progress identifies the current stage and elapsed time. Preparation, acoustic ownership fitting and temporal decisions run in both modes; Nemotron activity analysis runs when selected. Cancellation, source changes, obsolete label generations, or changed runtime provenance prevent installation. Completed fixed evidence can be reused. A failed project save cannot advance the accepted effective-envelope cache.

The previous preview remains visible when stale. Gain-only changes compose immediately and do not run the neural extractor or ownership classifier. A gain edit made during Analyze is composed from the latest project when the result installs. The displayed estimates are inspectable model probabilities; they do not promise calibrated certainty.

The waveform displays source peaks and conservative processed bounds. Blue marks human references, gray marks automatic gain, teal marks effective gain, pink marks manual points, and gold marks unity protection. Overview pixels summarize their entire time interval, so a brief owned or uncertain region remains visible. Gain drawing includes per-pixel extrema, including excursions shorter than one pixel.

Switch the envelope tool on to drag manual points horizontally for fade duration or vertically for gain. A drag uses a transient draft; release commits one undo command and cancellation restores the saved curve and markers. Arrow keys move a selected point, Delete removes its whole override, and Ctrl+Z/Ctrl+Y undo/redo. Text inputs retain their captured keys. Preserve and Attenuate create region edits. Conflicts require the explicit **Replace entire overlapping overrides and protections** option; it removes the conflicting records as whole records, including the other bindings of a global protection.

Open/close settings specify finite exponential-shaped fade durations with exact endpoints. Shared evaluation keeps automatic fades outside preserved speech. Manual gain remains authoritative until removed. All source-time curves stay sparse; native-rate playback and rendering use the same evaluator.

Review the waveform and audition directly. There is no uncertainty review queue. Use labels to improve detection or Preserve/Attenuate and protection to correct gain. Ignore gaps shorter than bridges brief automatic attenuation gaps between speech spans; it does not erase explicit room-tone labels. Original works before analysis. When a preview exists, Original, Processed and Removed preserve transport and seek context while using the current effective envelopes.

Save project persists labels, gain overrides, protections, review states and recipe identities. Opening a project validates sources; Analyze reconstructs its preview using reusable fixed evidence.

Create final WAV render publishes the accepted effective envelopes directly at each source's native sample rate. Export copies only that accepted render after verifying its report, WAV hashes, source identities and saved project identity. Monitoring gain and comparison mode never enter exported audio. Label/model/source changes require current analysis; gain edits require a fresh render without repeating inference. Both final render and delivery can be cancelled. Incomplete or obsolete private attempts never replace prior deliverables. A published attempt that cannot be accepted remains beside the project as recovery material; deleting it is an explicit file operation.

## Historical validation and remaining checks

Earlier controlled tests exercised complete composition, label correction and batching, the former review navigation, latest gain edits, drag/undo, comparison, save/reload, cancellation, obsolete generations, changed runtime and failed saves. Synthetic references/history are development fixtures, not owner truth or release-quality evidence. App checks cover fresh Original playback, draft markers and captured-key isolation. See [owner verification](owner-verification.md) and [GUI development](gui-development.md) for newer evidence; these records do not establish release acceptance.

Two-hour mixed-rate drawing tests use the production visible kernels. On the development computer, 100 gain/ownership queries plus navigation/hit checks took 108 ms total; 300 waveform peak queries with navigation took 1.44 ms. These measurements exclude neural work, cache construction, window compositing and physical audio-device latency. The queries are bounded by visible pixel/bucket counts rather than all source samples.

Offline `model-check --verify-only` passed against the installed pinned model and verified native runtime without inference or downloading. Owner listening and physical window verification remain separate release gates.

The authorized three-track recording has an ignored ready-to-open project at `test-audio/261005-supervised.cleanslate` in the primary checkout. It uses Speaker 1–3 and declared supplied synchronization. It deliberately has no solo or room references; the owner chooses those while listening in Original.
