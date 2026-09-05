# Playback preparation and output

Ticket 05 uses one background preparation worker, a newest-request generation, and one cached session of shared PCM. Stop and source replacement cancel outstanding work. Reads check cancellation every 65,536 mono samples and reject truncated or non-finite audio. Source stamps are checked before reuse, after loading, and before output starts. Prepared audio must also match the completed preview cache's source stamps.

The output iterator snapshots mode, monitor gain, seek requests, and shared envelopes every 256 samples. Control changes take effect within 5.34 ms of source iteration at 48 kHz, plus the existing device buffering. The playhead is published once per block and at selection end. A seek has a separate pending slot so playback cannot overwrite the request. The sequential segment cursor uses exactly the same `powf` arithmetic as `gain_at`; tests require bit-for-bit equality rather than a relaxed tolerance. It removes the segment binary search, not the power operation.

## Memory and limits

PCM is retained once per prepared session and shared with playback through `Arc`. There is no decoded copy for each Play request. Storage is four bytes per mono source sample plus a 256 KiB read buffer and envelope/control metadata. The supplied three 239,660,221-sample stems would retain 2,875,922,652 bytes, about 2.68 GiB. This is a session-bounded cache, not a fixed-byte streaming cache. Replacing the session drops the retained PCM; a cancelled read frees partial PCM on its worker. Preview analysis may independently load PCM during cold preparation. Streaming should replace this cache if session size exceeds available memory.

Identity follows the existing ordered path, length, and modification-time stamp convention. Changes that deliberately preserve all stamp fields are outside that convention. Device creation remains on the editor thread after preparation; whole-source reads do not. No human listening or device-latency measurement is claimed. Tests use generated fixtures and do not enable the user's audio device or copy reference WAVs.

## Verification

- Red checks failed on absent sequential evaluation and prepared-audio cache methods before implementation.
- Exact cursor comparison covers retarget points, forward playback, and backward seeks.
- Sample output tests cover all modes, monitor gain, selected range end, seek, and live envelope replacement.
- Preparation tests cover shared PCM reuse, same-path replacement, stale preview identity, missing source errors, cancellation, superseded requests, and invalidation.
- The warm-cancellation regression failed before the review fix and passed afterward. Canceled warm results restore the shared cache but cannot start playback; explicit source invalidation prevents restoration.
- Editor message tests cover Stop during preparation, source installation/relink invalidation, pending seek, and load failure without opening output.
- Final `cargo test --target-dir target-playback -j 1`: 51 passed, 3 existing reference/benchmark checks ignored, no failures.
- `cargo clippy --all-targets --target-dir target-playback -j 1 -- -D warnings`, rustfmt checks on the five changed Rust files, and `git diff --check` passed.

The first parallel debug build exhausted memory in the graphics dependency. Retrying with `-j 1` compiled successfully; subsequent commands use one job.

## Review

Standards review found a missing memory-ceiling comment and duplicated point arithmetic. Both are fixed: the PCM cache names its streaming upgrade path, and `GainPoint::gain_at` is shared by both access patterns. Follow-up review has no findings.

Spec review found that canceling a warm Play could discard the cache. The lifecycle regression reproduces the failure and verifies the fix, including supersession and source invalidation. Follow-up review has no findings.
