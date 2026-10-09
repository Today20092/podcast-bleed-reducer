# Supervised editor workflow

The Iced workspace runs the accepted pipeline: validated sources and 16 kHz analysis copies, usable human references, required verified Nemotron history, independent per-track ownership models, conservative temporal decisions, and shared effective envelopes. The retained detector is used only by its comparison tests.

Import synchronized mono tracks, declare supplied synchronization when appropriate, and listen in Original mode to find solo speech and room tone. Save a solo reference for each speaker and a room reference. Overlap references and additional solos are optional. Analyze validates reference usability; merely saving a label does not establish that it is usable. Speaker numbers are stable identities, not inferred names.

Verify the installed model offline, or download it once. Analyze rechecks actual model and native runtime hashes on its worker before using cached neural history. Missing or changed assets produce an actionable error and retain the previous preview. No alternate detector or network request is used as a fallback.

## Analysis and editing

Analyze is explicit. Several label changes can be collected before one job. Progress identifies preparation, required model/activity analysis, ownership fitting and temporal decisions. Cancellation, source changes, obsolete label generations, or changed runtime provenance prevent installation. Completed fixed evidence can be reused. A failed project save cannot advance the accepted effective-envelope cache.

The previous preview remains visible when stale. Gain-only changes compose immediately and do not run the neural extractor or ownership classifier. A gain edit made during Analyze is composed from the latest project when the result installs. The displayed estimates are inspectable model probabilities; they do not promise calibrated certainty.

The waveform displays source peaks and conservative processed bounds. Blue marks human references, gray marks automatic gain, teal marks effective gain, pink marks manual points, and gold marks unity protection. Overview pixels summarize their entire time interval, so a brief owned or uncertain region remains visible. Gain drawing includes per-pixel extrema, including excursions shorter than one pixel.

Switch the envelope tool on to drag manual points horizontally for fade duration or vertically for gain. A drag uses a transient draft; release commits one undo command and cancellation restores the saved curve and markers. Arrow keys move a selected point, Delete removes its whole override, and Ctrl+Z/Ctrl+Y undo/redo. Text inputs retain their captured keys. Preserve and Attenuate create region edits. Conflicts require the explicit **Replace entire overlapping overrides and protections** option; it removes the conflicting records as whole records, including the other bindings of a global protection.

Open/close settings specify finite exponential-shaped fade durations with exact endpoints. Shared evaluation keeps automatic fades outside preserved speech. Manual gain remains authoritative until removed. All source-time curves stay sparse; native-rate playback and rendering use the same evaluator.

Review controls select an uncertainty reason, seek the range and highlight its tracks without changing the comparison mode. Review keys include the accepted temporal identity and can be marked reviewed in the project. Original works before analysis. When a preview exists, Original, Processed and Removed retain the same effective envelopes behind their modes, preserving transport and seek context.

Save project persists labels, gain overrides, protections, review states and recipe identities. Opening a project validates sources; Analyze reconstructs its preview using reusable fixed evidence. This ticket does not publish a final render. Ticket 13 supplies current-result native export and playback/export parity.

## Validation

Controlled public tests exercise complete composition, label correction and batching, review seek, latest gain edits, drag/undo, comparison, save/reload, cancellation, obsolete generations, changed runtime and failed saves. Synthetic references/history are explicitly trusted development fixtures, not owner truth or release-quality evidence. App checks cover fresh Original playback, draft markers and captured-key isolation.

Two-hour mixed-rate drawing tests use the production visible kernels. On the development computer, 100 gain/ownership queries plus navigation/hit checks took 108 ms total; 300 waveform peak queries with navigation took 1.44 ms. These measurements exclude neural work, cache construction, window compositing and physical audio-device latency. The queries are bounded by visible pixel/bucket counts rather than all source samples.

Offline `model-check --verify-only` passed against the installed pinned model and verified native runtime without inference or downloading. Owner listening and physical window verification remain separate release gates.

The authorized three-track recording has an ignored ready-to-open project at `test-audio/261005-supervised.cleanslate` in the primary checkout. It uses Speaker 1–3 and declared supplied synchronization. It deliberately has no solo or room references; the owner chooses those while listening in Original.
