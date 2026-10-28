# Bounded shared audio windows for the FL Studio prototype

Status: claimed
Assignee: Codex
Type: prototype
Blocked by: none
Related: 67, 75

## Authorization and prerequisite

2026-10-24: owner requests continued implementation and notification when another native test is needed. Ticket 75's verified native loading, persistent group/member settings and matching-timeline callbacks establish this narrow prerequisite; its remaining host matrix does not become done. Continue sequentially on development without changing desktop processing or the owner's FL project.

## Scope

Version 0.0.2 shares native-rate PCM in bounded, process-local queues between the two explicit members. Validate a delayed same-timeline window using preallocated histories. Audio output remains bit-exact pass-through, with no added output delay or attenuation. Reset/group/member/source discontinuities invalidate eligibility. Missing/late, invalid, overwritten or incompatible evidence is reported rather than inferred. No callback waits for its peer or allocates. Never log audio samples or signal-derived features.

Investigate activation-time render reporting. Preserve actual host-reported mode without assuming it identifies export; add an explicitly manual Playback/Export run label for the next owner test if needed. No framework fork or fabricated automatic export detection. Keep group/member IDs and plugin identity compatible with the saved FL project. Native host testing remains owner-operated.

## Acceptance

- [x] Unit tests cover exact PCM delivery across variable packet sizes and callback ordering; group/seek/rate/channel resets; absent/duplicate/replaced members; nonfinite samples; overflow and bounded work.
- [x] Actual CLAP ABI host verifies pass-through, queue readiness and state round-trip, including mono/stereo, 44.1/48 kHz, realtime/offline and missing transport.
- [x] Logs distinguish validated shared windows, warmup/missing data, queue losses and activation-mode versus manual run label, without audio contents.
- [x] Build/check in a candidate directory; preserve 0.0.1 and publish the tested candidate to the linked output only after automatic checks pass.
- [x] Run applicable repository checks; record sources, hashes and remaining limits.
- [ ] Owner FL Studio playback, seek/loop, reopen and separately labeled export traces qualify the new queue before connecting calibration/inference.

The analysis lookback is a diagnostic window policy, not reported output latency or proof of the desktop algorithm's realtime equivalence. Calibration/inference, VST2 and OBS remain separate follow-up scope under 67. A fixed queue cannot promise arbitrary host branch-ahead rendering support.

## Delivery — 2026-10-24

Built on development base `88dda154daa9f442018302e19f1c792513fd0874` with uncommitted plugin changes. No desktop pipeline/dependency changes, native UI automation or edits to the owner's FL project. Code graph discovery was attempted but its transport was closed; local source inspection covered the existing adapter and pinned framework API.

The shared bus has 16 outgoing queues (eight explicit two-member groups), each with at most 64 immutable packets of 256 native frames and up to two channels. Each instance preallocates two 16,384-frame local histories outside processing. A callback above 8,192 frames skips sharing instead of truncating. Queue drains have a fixed 64-packet upper bound; callbacks never wait for peers and debug allocation checks remain enabled. Owner/recipient IDs, epoch, native rate/channel count, exact time ranges and continuous-history coverage qualify a 512-frame PCM window starting 2,048 samples before the current block. That lookback is diagnostic only, with zero output latency/attenuation. Nonfinite/missing/incompatible data, seeks/replacements, gaps, overwritten history and full queues cannot fabricate a Ready window. No samples, levels or signal fingerprints are logged.

Confirmed render-mode limitation in pinned nice-plug 0.4.2: its public `BufferConfig::process_mode` reaches Plugin::activate; CLAP ext_render_set requests a host restart when mode changes while active. Therefore `activation_mode`/`offline` explicitly describe activation state. The synthetic host proves the restart request, but FL's exact render callback behavior remains unknown; no framework defect is claimed. New saved BoolParam ID `probe_export_run` labels a test as Export manually and does not change mode, processing eligibility or sound. Logs retain actual host playing state separately from processing intent, and add API, activation mode, manual label, shared-window status/start/frames, cumulative audio-queue drops and remaining peer queue depth. Original group/member IDs and plugin IDs remain stable.

Checks passed:

- Nine unit tests, including exact PCM payloads, history wrap/gaps, alternating and concurrent callback order, peer replacement, missing time, invalid samples, unsupported buffers, format mismatch, full queues and duplicate members.
- Synthetic actual-CLAP host: 1,858 bit-exact mono/stereo blocks at 44.1/48 kHz, normal/offline activation, variable blocks and missing transport; 1,168 Ready PCM windows with zero normal-schedule audio drops; clean finalized reporting; active mode-change restart requests; manual-label reporting.
- State loading uses the exact 0.0.1 JSON parameter shape observed read-only in the saved test FLP, restoring group/member correctly with the new manual label defaulting off; 0.0.2 save/mutate/load also restores all controls.
- Plugin formatting and strict all-target Clippy; PowerShell parser and diff checks.
- `scripts/check.ps1 -Local`: 370 tests passed, 20 existing ignored; workflow/package/release/action fixtures, desktop formatting and strict Clippy passed. No desktop GUI build was needed for the isolated workspace.

`scripts/build-fl-probe.ps1` now builds/checks `dist/fl-probe-candidate/`, including the actual ABI host, then preserves the previous linked output in `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/fl-probe/<sha256>/` and copies the tested candidate into `dist/fl-probe/`, verifying both format hashes. The original 0.0.1 remains preserved under `d83262707e58be573d8a85552f168a3139b3f8291dc2fad2ff2c925949a52fb2/`. Existing installation links keep their targets. Close FL Studio before rebuilding.

Final 0.0.2 CLAP and VST3 DLL SHA-256: `073d5b01001a94df4153a138390c2341f120c56459a5cdadef517ec2ca2e5b45`.

- `C:/Users/User/Documents/podcast-automix-rust-gui/dist/fl-probe/CleanSlate FL Probe.clap`
- `C:/Users/User/Documents/podcast-automix-rust-gui/dist/fl-probe/CleanSlate FL Probe.vst3/Contents/x86_64-win/CleanSlate FL Probe.vst3`
- Source DLL: `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/cleanslate_fl_probe.dll`.
- [Instructions](../../../plugins/fl-probe/README.md).

Remaining native check: fully reopen FL Studio and the saved test project, confirm restored member assignments/new Export label, run 10 seconds of playback plus seeks/loop, then mark Export in both instances and render a short WAV beside the FLP. Close the project for finalized 0.0.2 reports. Validate Ready coverage after warmup, resets, audio-queue losses and export flags before connecting reference calibration. VST3 processing, latency budgets, desktop sound parity, full native matrix and inference are unclaimed. Ticket remains claimed awaiting owner test.

## Native CLAP test result — 2026-10-24

Owner reports playback, seeks and export with the manual label enabled, and provides `C:/Users/User/Documents/Image-Line/FL Studio/Projects/cleanslate_testing/cleanslate_testing.wav`. Reviewed process-33412 version-0.0.2 reports. No native UI control or binary change during review.

Playback proof: three finalized paired activations beginning `1792884348324...`, `1792884354407...`, `1792884365358...` contain respectively 765, 1,865 and 10,539 active callbacks per member. Ready counts per member are 746, 1,827 and 10,464: 26,074 validated PCM windows across the two members in those activations, about 99% of their active callbacks. Boundary warmup/reset is visible. All finalized reports have zero reporting-queue drops, zero omitted rows and no I/O failure; no audio-queue drops or incompatible/invalid-audio/unsupported-buffer states appear. Group 1/member assignments persisted into the new process. This supports playback sharing under tested conditions, not model deadline or universal host guarantees.

Export identifies a blocker rather than an acceptance pass. Actual CLAP `activation_mode=Offline` and `manual_export=true` now appear in finalized reports `probe-33412-1792884406654029600-2.csv` (member 1, 2,188 rows) and `probe-33412-1792884406679119100-1.csv` (member 2, 1,696 rows). Neither has a Ready window. Member 1 starts with 493 MissingPeer rows before the other instance is present in the group; subsequent rows are Warmup with boundary Reset observations. The available peer start positions during playing differ by 72,857–73,287 samples (median 73,072), or approximately 1.518–1.527 seconds at 48 kHz. This greatly exceeds the 2,048-sample diagnostic lookback and 16,384-frame history. Both queue overflow and report loss remain zero, so raising packet capacity alone is not an established fix. Staggered member reactivation and differing reported timeline positions are observed; whether the underlying cause is CLAP transport reporting, host branch scheduling or restart/lifecycle timing remains unproven. Do not infer equivalent PCM content or actual rendered drift from timestamp metadata alone. Do not connect attenuation or disguise the failure by merely choosing a larger lookback.

The WAV validates as RIFF/WAVE, PCM encoding 1, stereo 48,000 Hz, 24 bits, block alignment 6; 3,090,858 data bytes = 515,143 frames = 10.7321458333 seconds. Declared RIFF size matches the actual 3,090,974 bytes and all scanned chunks fit the file. SHA-256 `a4074c66a2214b0da79c434d8ab8f7049addf9f68e1a73244ce40e126daea300`. It is a valid nonempty container; no dry-reference sample comparison or listening qualification was performed.

Next discriminating owner test: in a copy of this test project, use VST3 on both microphone inserts with group 1/members 1 and 2, enable the manual Export label and export the same short range to `cleanslate_testing_vst3.wav`. The existing linked 0.0.2 VST3 needs no rebuild. Compare VST3 offline transport and shared-window coverage against the CLAP trace before choosing a host/format adapter or buffered-render architecture. Preserve the existing CLAP FLP/WAV. Further tests remain owner-operated. Two later idle reports were still open at review; finalized export reports above are sufficient to establish this failure.

## Native VST3 comparison — 2026-10-24

Owner supplied `cleanslate_testing_vst3.wav` beside the prior WAV. Finalized VST3 reports `probe-33412-1792885018095849700-1.csv` (member 2) and `probe-33412-1792885018098265400-2.csv` (member 1) each have 2,400 active callbacks, with identical timeline intervals at every matching index. Each validates 2,389 shared PCM windows after startup warmup. No audio-queue drops, reporting drops, omitted rows or I/O failure; maximum available peer start-position difference is 215 samples, and remaining peer queue depth is at most one. Both traces carry the manual Export label. They cover 514,286 active sample frames in approximately 0.161 seconds of wall time, consistent with the reported faster-than-realtime bounce. The host still supplies Realtime activation mode, so actual bounce identification relies on owner context plus accelerated timeline progression, not an Offline flag. Five stopped callbacks per instance remain Inactive, and end-of-file inference handling is not qualified.

VST3 WAV validates with the same 515,143-frame, 10.7321458333-second stereo 48 kHz/24-bit PCM format as the CLAP WAV. File SHA-256 `ac167d12d10a11bfe21fa84aa5b9de9ae1c6c4a4b8c230676898d5700d582b05`. PCM bytes are not bit-identical: 2,284 of 1,030,286 sample values differ (0.221686%), every difference exactly one 24-bit integer step, RMS difference approximately -165.02 dBFS. Do not attribute those tiny rounding differences to a specific cause without controlled export settings, or call this a dry-reference comparison. This supports the native VST3 shared-window route in the tested project; the large export timing discrepancy remains specific to the observed CLAP route, without yet assigning blame to the host or framework.

Owner now asks for lightweight autonomous testing instead of repeated DAW sessions. Continue with headless format hosts/validators and recorded-schedule regression tests, measuring memory where feasible. Do not launch FL Studio or another DAW for routine checks. Reserve owner testing for integration behavior that cannot be established by those harnesses. Existing actual CLAP ABI tests remain useful; VST3 currently checks its factory symbol only and needs a real headless processing harness.
