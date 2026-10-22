# Selection playback latency references

Researched 2026-10-12 for ticket 40. Cargo.lock currently pins Rodio 0.21.1 and CPAL 0.16.0. These sources support implementation choices, not measured latency claims for CleanSlate or the owner's audio hardware. Current CPAL references below require version-specific verification before implementation.

## Recommendation

First measure and improve the existing Rodio path. Keep the output stream alive across transport commands, reuse compatible prepared sources, and let selection changes update a frame cursor and range rather than initiate source conversion. Separate source/device invalidation from selection changes. Preserve current one-shot selection playback and speaker-popup looping. A new DAW engine dependency is not justified by these references alone.

Keep decoding, resampling and source installation outside audio processing. Transport commands must have bounded processing and obsolete commands must not restart old recordings. Underflow, device loss and source replacement need explicit behavior. These are engineering recommendations inferred from the sources below, not claims that our current callback already meets them.

## Existing library path

- Rodio 0.21.1 documents that dropping `OutputStream` disposes the output stream and ends playback. Its mixer can serve successive sources without reopening the device. `Sink::connect_new` attaches playback to that mixer. This supports retaining the device stream even if replacing a lightweight source or sink remains the smallest correct implementation. [OutputStream](https://docs.rs/rodio/0.21.1/rodio/stream/struct.OutputStream.html), [Sink](https://docs.rs/rodio/0.21.1/rodio/struct.Sink.html).
- `Sink::try_seek` requires every underlying source to support seeking and is documented to block for approximately 0 to 5 ms. `stop` empties its queue, while `pause` preserves resumable playback. Do not assume an arbitrary custom source becomes seekable or can restart after queue exhaustion. Verify the installed source and sink behavior before choosing seek versus source replacement. [Rodio 0.21.1 Sink](https://docs.rs/rodio/0.21.1/rodio/struct.Sink.html).
- CPAL supplies a running stream with a periodic output callback on a dedicated audio thread on modern platforms. A retained stream can output silence while transport is stopped. Buffer and backend settings still affect when sound reaches speakers. This is not a zero-latency guarantee. [CPAL documentation](https://docs.rs/cpal/latest/cpal/).
- Context7 returned current Rodio examples using `Player`, while the version-pinned 0.21.1 documentation uses `Sink`. CleanSlate declares Rodio 0.21 in `Cargo.toml`; implementation must use the actual locked Rodio and transitive CPAL versions rather than copying current-master API examples. No dependency upgrade is proposed.

## Rust FOSS references

| Reference | Useful pattern | Limit and reuse status |
| --- | --- | --- |
| [Firewheel design](https://github.com/BillyDM/firewheel/blob/main/DESIGN_DOC.md) | An activated device stream owns a processor; the user sends updates through a realtime-safe channel. The design explicitly avoids mutexes in realtime processing and separates sample resources from transport events. | A game/general audio graph engine, explicitly not a complete DAW engine. Read as an architecture reference before considering adoption. [MIT OR Apache-2.0 manifest](https://github.com/BillyDM/firewheel/blob/main/Cargo.toml). |
| [Creek read-stream design](https://github.com/MeadowlarkDAW/creek/blob/main/README.md) | An IO server performs non-realtime work. Preloaded range caches support immediate cached seeks and seamless loop starts; look-ahead buffers service subsequent audio. Cache misses require buffering or silence, so arbitrary distant seeks are not promised immediate. | Useful if full-session prepared PCM memory becomes a measured problem. Bound cache count and size explicitly. The README links the project's move to Codeberg and includes a loop-player demo. [MIT OR Apache-2.0 manifest](https://github.com/MeadowlarkDAW/creek/blob/main/Cargo.toml). |
| [Dropseed](https://github.com/MeadowlarkDAW/Dropseed) | A Rust DAW engine reference for broader transport design. | Its GitHub repository is archived and points to Codeberg. No end-to-end maturity or performance claim is established here. [AGPL-3.0 license](https://github.com/MeadowlarkDAW/Dropseed/blob/main/LICENSE); no code copying is proposed. |

Rodio is [MIT OR Apache-2.0](https://github.com/RustAudio/rodio/blob/v0.21.1/Cargo.toml); current CPAL is [Apache-2.0](https://github.com/RustAudio/cpal/blob/master/Cargo.toml). Before importing any source, verify the exact revision, file and dependency licenses and preserve required notices. Dropseed's copyleft license needs separate review before incorporating code into this MIT project. Patterns and public APIs are the proposed reuse, not copied source.

## Checks for implementation

Record request-to-worker, prepared-source lookup, device-open, source-attach and first-output timing separately. A callback timestamp establishes submitted audio, not physical speaker onset. Use loopback or owner listening for audible onset where available. Compare cold preparation and repeated warm selection audition with short and long recordings and mixed rates. Count decoder/resampler jobs and device opens to demonstrate reuse. Exercise rapid selection changes, stop during pending preparation, range ends, loop boundaries, project replacement and device failure.

No benchmark, application change, dependency installation or source copying was performed for this research.
