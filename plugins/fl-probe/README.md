# CleanSlate FL Probe 0.0.6

Windows x64 **Session prototype** for prerecorded synchronized microphone tracks, with VST3 and CLAP adapters. Session uses the maintained desktop analysis pipeline: embedded Silero, acoustic/transfer evidence, independent supervised ownership, temporal decisions and effective gain envelopes. Live Mic is later work. Native FL sound, delay compensation and export remain owner acceptance.

## First Session test

1. Close FL Studio before updating linked binaries. Use a disposable project copy. Load **CleanSlate FL Probe**, initially VST3, on each dedicated microphone insert. Use one format/version in one host process.
2. Old saved states keep the legacy interface: click **Open the persistent Session workspace**. Set the same group and unique Mic numbers (1–8). Stop/restart playback if host reactivation is needed.
3. Play through a section with processing off. Input history stays at DAW timestamps across loops/seeks. Playing 10–30 then 0–30 fills earlier gaps and verifies overlapping audio. Only audio delivered by the DAW can be captured.
4. Drag ranges and label **Room**, solo speakers and optional overlap sets. Speaker labels identify who is talking, including bleed heard by other microphones. Every participant must have captured the selection. Selection pins the viewport; Fit, zoom and earlier/later controls navigate without changing host transport.
5. Stop playback, then **Analyze**. It analyzes one verified common continuous interval containing all examples. Gaps are rejected, never synthesized as trusted silence. Sound settings are shared and require Analyze after changes. Shared sound host automation and per-track overrides are later work.
6. Enable processing on each insert and replay. Compare **Original**, **Processed** and **Removed**. Waveforms show input. Exact PCM fingerprints must match before attenuation; unknown, missing, changed or stale sections remain original. Changed example audio requires review/relabeling.
7. Save/reopen: cache references, examples and shared sound settings persist. Waveforms can reappear, but replay must verify source audio/label fingerprints, then Analyze before processing. Missing cache means recapture. A DAW project copy alone does not copy the cache.
8. Export with effects and sufficient trailing silence for the reported tail. Native FL playback/export qualification remains necessary, especially CLAP; the old streaming adapter's export failure is not claimed fixed by cached Session processing.

Session mode always reports **416 ms plus the host maximum block duration**, including when processing is off. All audition modes share it, so enabling processing does not request a host reload or discard analysis. This is not a near-zero-latency headphone path. Prefer Original inside the plugin for comparison; host slot bypass can alter delay compensation. Legacy probe mode retains zero added delay when disabled.

## Waveform navigation

Wheel over the waveform zooms around the pointer; Ctrl+wheel/pinch also zooms. Shift+wheel or horizontal scrolling pans. Middle-drag pans without changing the selection; Alt+drag fits a time range. Double-click or Home fits the captured history. Drag normally to select an example, then use Fit selection (F). Plus/Minus zoom, Left/Right pan, and C centers on the DAW playhead when the waveform has keyboard focus. Right-click lists these controls. The bar below the waveform moves the viewport through captured time.

Follow off keeps the view still. Catch up advances at the edge; Lock playhead follows continuously. Manual navigation temporarily suspends following. Transport remains controlled by the DAW. The display retains its last snapshot during disk-worker contention and refreshes captured peaks at most every 33 ms; the host/editor event loop and monitor determine actual presentation timing. Waveform evidence remains coarse 32 ms min/max bins, not sample editing.

## Storage and source safety

- Audio callbacks enqueue PCM and check in-memory fingerprints. Disk writes, waveform updates and analysis run outside callbacks. Local audio cache: `%LOCALAPPDATA%/CleanSlate/PluginSessions/<session-id>/<rate>/`. Clear captured cache clears this group's working audio/examples, never DAW sources.
- Current bounds: nonnegative DAW time through two hours, 8–192 kHz integer rates, up to eight mono/stereo inserts, callback sizes through 8,192 samples. Participants need the same rate. Analysis uses **input channel 1**; its envelope applies to both channels without mixing. Each insert represents one speaker's microphone.
- Analysis requires one verified common contiguous interval containing all labels. Fill gaps; disconnected captured islands stay visible. Partial source-verification blocks at interval boundaries remain original. Each immutable analysis WAV must fit RIFF's size limit; use shorter sections at high rates.
- Identity uses exact PCM, not waveform similarity. Place inserts before randomized/changing upstream effects. Resampling or nondeterministic processing can invalidate cached analysis. Unplayed DAW edits remain undetectable until delivered.
- Queue overflow/disk failure prevents trusting new capture. Old history can remain visible as unverified; replay it. RAM is bounded by coarse peaks, fingerprints and queues rather than whole native recordings. Long-session RAM/disk/time still need measurement.
- Saved identity overrides ambiguous numeric groups. Conflicting sessions/duplicate microphones are rejected; an exclusive cache lock prevents multiple processes writing one cache. Sandboxed/bridged processes do not share the in-process session.

## Verification and formats

Use `scripts/build-fl-probe.ps1` and `scripts/test-plugin-hosts.ps1` for device-free tests/build/actual-format checks. Regression tests cover loops/gaps, identical-extrema PCM changes, saved label identity, cache reopen, safe monitoring and the real desktop analysis bridge. The delivery receipt records actual passing checks and hashes; test source alone is not a passing receipt.

Old saved states can still use the paired streaming probe. Its rolling history, energy/coherence recipe and host sound parameters are the legacy path. CLAP attenuation remains disabled there. CSVs in `%LOCALAPPDATA%/CleanSlate/PluginProbe/` describe that transport proof, not Session gain qualification.

Primary formats: VST3/CLAP; only Windows x64 is built here. VST2 needs a separate adapter/distribution path; AAX/AU are deferred. Native macOS/Linux, shared sound automation, per-track overrides, point editing and Live Mic are not delivered here. Stable plugin IDs remain `org.cleanslate.fl-probe` and `CleanSlateProbe1`. Included notices cover dependencies and the embedded model/runtime.
