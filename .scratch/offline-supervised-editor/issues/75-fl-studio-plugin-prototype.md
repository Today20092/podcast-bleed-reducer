# FL Studio grouped plugin delivery prototype

Status: claimed
Assignee: Codex
Type: prototype
Blocked by: none
Related: 67

## Owner authorization — 2026-10-24

Build the first FL Studio on Windows prototype. The owner also requests VST2, VST3 and CLAP as product targets. This bounded first delivery exports VST3 and CLAP; investigate the VST2 adapter/licensing path and retain it as unfinished scope rather than claiming compatibility. OBS remains a separate goal.

## Scope

Two grouped pass-through inserts with persistent group/member parameters, bounded nonblocking timing diagnostics, transport discontinuity handling and original audio preservation. Collect no recording contents in diagnostic logs. Use host parameter controls rather than building a custom editor for this diagnostic proof. No attenuation, training, audio recording, source separation or changes to the desktop pipeline.

## Acceptance

- [x] Build and preserve Windows x64 VST3 and CLAP artifacts with pinned dependencies and a reproducible script.
- [x] Test diagnostic alignment, discontinuities, missing/duplicate members and bounded storage without changing audio samples.
- [x] Verify exports/loadability with an automated host where available (CLAP processing; VST3 export symbol only).
- [x] Document FL Studio installation, group setup and playback/seek/loop/offline-export checks; distinguish executed from pending checks.
- [x] Record exact paths/hashes, licensing limits and next go/no-go criteria in this ticket.
- [ ] Owner FL Studio qualification: discover/load both formats, generic parameter access, two tracks, playback/seek/loop/bypass/Smart disable, project reopen and offline export.

Ticket 67 remains open for calibration, inference, host qualification and the other format/integration goals. A loadable pass-through proof is not completed bleed reduction.

## Delivery — 2026-10-24

Built on `development`, base commit `88dda154daa9f442018302e19f1c792513fd0874`, with the new plugin sources uncommitted. The experimental workspace is `plugins/fl-probe/`, separate from the desktop manifest/lockfile. `scripts/build-fl-probe.ps1 -Fetch` performs initial locked dependency fetch; subsequent runs are offline and use the shared application-data Cargo cache and pinned toolchain. Framework: exact ISC-licensed nice-plug 0.4.2, with Cargo.lock. It exports CLAP and VST3; VST2 remains requested, needs a separate adapter and a verified licensing route. Steinberg's closed VST2 SDK licensing does not apply to current VST3 licensing; no VST2 binary is delivered.

Artifacts (both 1,186,304 bytes, same DLL content):

- `C:/Users/User/Documents/podcast-automix-rust-gui/dist/fl-probe/CleanSlate FL Probe.clap`
- `C:/Users/User/Documents/podcast-automix-rust-gui/dist/fl-probe/CleanSlate FL Probe.vst3/Contents/x86_64-win/CleanSlate FL Probe.vst3` (install the whole outer `.vst3` directory)
- SHA-256 for each: `d83262707e58be573d8a85552f168a3139b3f8291dc2fad2ff2c925949a52fb2`.
- Build source DLL: `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/cleanslate_fl_probe.dll`; packaged copies above preserve the tested build.
- Setup and evidence interpretation: [README](../../../plugins/fl-probe/README.md).

Checks: three unit tests pass; strict plugin Clippy and formatting pass. The synthetic CLAP ABI host loads two actual instances, changes group/member parameters through the host API, and passes 288 bit-exact stereo blocks with separate and in-place buffers at 44.1/48 kHz, realtime/offline modes, lengths 1–1024, resets/seeks, stopped transport, negative position and absent transport. The DLL exports `GetPluginFactory`; VST3 processing was not host-tested. Eight finalized CLAP CSVs each contain 36 rows, zero dropped/omitted events and no I/O failure. Their statuses distinguish serial skew, matching blocks, reset epochs and missing timestamps. Framework debug allocation checks are enabled. No recordings are captured or logged.

The required repository `scripts/check.ps1 -Local` also completed successfully: workflow/packaging/release/action fixtures, desktop formatting, strict all-target/all-feature Clippy and all applicable desktop tests. Existing opt-in ignored tests remain ignored. No desktop GUI rebuild was needed because the desktop sources, manifest and lockfile are unchanged.

The owner explicitly requested to test FL Studio themselves. FL Studio 2026 was opened, but no plugin was loaded and no project was changed by automation. Native host/GUI, project restoration, Smart disable, sample-for-sample FL export and real scheduling bounds remain unclaimed. Ticket remains claimed pending those checks; no plugin release or desktop audio behavior change occurred.

Known proof limits: only two members per group; latest peer snapshot is diagnostic metadata, not synchronized audio delivery or inference readiness. No cross-process or mixed-format grouping. Logs are bounded per activation (4,096 queued events, 200,000 saved rows) but accumulate across activations. Repeated seeks conservatively invalidate the whole group. Next go/no-go: owner traces must show usable timeline metadata and motivate/test a bounded actual-audio queue before connecting calibration or attenuation. Missing evidence always leaves samples untouched in this prototype.

### Scan-location correction — 2026-10-24

The owner reported that rescanning `dist/fl-probe` did not find the plugin. The earlier custom-search-directory instructions were incorrect for FL Studio's VST3/CLAP discovery. Its [official installation manual](https://www.image-line.com/fl-studio-learning-content/fl-studio-online-manual/html/basics_externalplugins.htm#vstfolders) requires standard VST3 locations and documents `C:/Program Files/Common Files/CLAP` for CLAP. Neither CleanSlate artifact was present at the standard locations. README now specifies those paths, verified rescanning and the exact display name. An attempted direct copy to the standard CLAP location failed with access denied, so no plugin was installed and the VST3 copy was not attempted. Owner must copy the files through Explorer with administrator approval, then rescan. No FL Studio UI was controlled and no host compatibility is claimed.

### Development links — 2026-10-24

The owner prefers development links so the project build output remains authoritative. Added `scripts/link-fl-probe.ps1`: creates the VST3 bundle junction and CLAP file symlink at their standard locations, refuses unrelated/ordinary existing destinations, permits rerunning with the expected links, and verifies binary hashes through each link. No deletes, replacement, self-elevation or security changes. README documents closing FL Studio before builds (links do not hot-reload loaded DLLs) and recreating links if the repository moves.

PowerShell parser check passed. Running the script created and hash-verified `C:/Program Files/Common Files/VST3/CleanSlate FL Probe.vst3` as a junction to the project bundle. CLAP symlink creation failed with access denied; it remains absent. The owner can rerun the concrete script once from administrator PowerShell to finish CLAP setup. The verified VST3 junction is already available for the owner's rescan, but no native scan/load result is claimed. This supersedes the copy-only recommendation above. No plugin binary changed.

### Owner host test and log review — 2026-10-24

Owner ran the administrator link setup and supplied successful hash verification for both links. Native screenshots then confirm discovery/loading of VST3 and CLAP, visibility of both published parameters, and FL's generated two-knob interface for CLAP (the VST3 wrapper uses Browser parameter access). The owner subsequently reports setting both CLAP instances to group 1, members 1/2, playing audio and exporting. Export completion is owner-reported; output audio has not been independently inspected.

Reviewed local process-28764 metadata logs under `%LOCALAPPDATA%/CleanSlate/PluginProbe/`. The snapshot comprised 78 files, including empty-activation reports and setup activity. Early duplicate member-1 observations occurred while configuring; the completed correctly assigned playback segments contain no duplicate-member, missing-time or sample-rate mismatch rows.

Bounded completed evidence uses these two activation pairs (all prefixed `probe-28764-`):

- `1792882946159613800-1.csv` / `1792882946160053400-2.csv`: 1,548 rows per member, 1,304 playing callbacks each, 3.1733 seconds of playing audio per member, with matching first/last playing timeline positions and two marked discontinuities per member.
- `1792882952748254600-1.csv` / `1792882952748751400-2.csv`: 614 playing rows per member, 1.4933 seconds per member, matching first/last playing positions and one marked discontinuity per member.

All four finalized files report zero queue drops, zero omitted rows and no I/O failure. Sample rate is 48 kHz, block lengths range from 1 to 215 samples, timing comes from host seconds reconstructed to sample positions. Within continuous playing runs there are no unmarked position gaps in these four files. Across their 3,836 playing observations: 1,916 SameBlock, 1,839 DifferentBlock, 78 MissingPeer and 3 EpochMismatch. Largest absolute start-position difference from an available peer snapshot is 215 samples (4.4792 ms at 48 kHz). This is observed latest-snapshot scheduling skew, not measured processing latency, guaranteed delivery bounds or a proof that an audio/inference queue can meet deadlines. Serial/asynchronous callback order is consistent with the alternating same/different-block pattern.

The latest pair `1792882956190956000-1.csv` / `1792882956191411200-2.csv` was still open during review and lacked final loss/I/O counters; do not treat it as complete. All inspected process-28764 rows mark offline=false. Therefore the current traces do not independently distinguish or qualify the reported export pass. Investigate host render-mode reporting and the probe's activation-time mode capture before claiming offline processing coverage. No source or binary was changed during this review; the owner need not rebuild for this finding. Save/close the project to finalize remaining reports. The evidence supports continued development of a bounded actual-audio delivery queue, while native export parity, full seek/loop/Smart-disable/reopen acceptance and ML processing remain outstanding.

### Saved project and finalized traces — 2026-10-24

Owner supplied `C:/Users/User/Documents/Image-Line/FL Studio/Projects/cleanslate_testing/cleanslate_testing.flp`. The file exists (51,540 bytes); read-only inspection finds two CLAP IDs and two embedded group/member parameter sets: group 1/member 1 and group 1/member 2. This establishes that the chosen assignments reached saved plugin state, not successful state restoration after reopening. The containing folder has only the FLP; no exported WAV is available there for independent output comparison. The project was not modified or opened by automation.

Previously open pair `1792882956190956000-1` / `1792882956191411200-2` is now finalized: 15,851 rows per instance, with 15,271 playing pairs. Pairing by callback index yields identical start, length and playing state for all 15,851 rows, zero missing timeline positions/rate mismatches/duplicate-member observations, and zero dropped/omitted rows or I/O failure in both footers. During playing pairs, process-local observation timestamp differences are median 4 us, p95 13 us, maximum 218 us. These are metadata observation differences for this run, not DSP latency/deadline measurements or audio-content equality. Latest-peer start-position difference remains at most 215 samples. That pattern is consistent with separately scheduled callbacks receiving matching timeline intervals.

An additional finalized pair `1792883223285792900-1` / `1792883223286135000-2` contains 4,968 rows per instance, all stopped; corresponding timeline intervals match, and both footers have zero losses/I/O failure. All reviewed rows still mark offline=false, so export mode remains unqualified. Native FL loading, parameter access, assignment persistence into the saved project and matching-timeline callback delivery are now supported by owner evidence. Actual synchronized sample queues, explicit seek/preroll policy, render-mode instrumentation and dry/processed export comparison are the next engineering proofs before connecting short-reference model fitting/inference. Ticket remains open; no binary changed during these reviews.
