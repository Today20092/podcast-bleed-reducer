# Plugin 67: the next bounded proof

2026-10-24. Follow-up to [the feasibility report](live-plugin-feasibility-2026-10-23.md) and [ticket 67](../../.scratch/offline-supervised-editor/issues/67-live-clap-vst3-discovery.md). This records current primary-source checks and a proposed proof, not an accepted implementation specification or a host compatibility claim. No dependency or format has been selected.

## What the user would do

For the requested DAW workflow, insert CleanSlate on each dedicated microphone track, assign each instance a speaker and explicit session group, then use the host waveform to play a few solo/overlap/room examples while a shared calibration panel captures and labels the synchronized inputs. Fit the session calibration outside the audio callback. During ordinary playback/render, show group readiness, speech/reduction meters, reduction/fade controls and Original/Processed/Removed listening. This is the proposed interface shape; no additional waveform editor is required.

Short references teach the model; all microphone inputs still need to reach its subsequent decisions. Installing several inserts does not make their streams automatically synchronized or available to one another. The earlier report documents the current engine's fit/application coupling and future-context requirements; an unchanged call to the file-based pipeline from an audio callback will not work.

For solo OBS, the separate product shape is a microphone filter with soundcheck, speech/reduction meters and gate controls. The multitrack ownership features have no competing microphones in this use case. Speech detection and bounded gain control are the relevant reusable pieces. Gain-only processing cannot remove noise or another voice selectively while wanted speech occupies the same samples; it can attenuate pauses or non-owning microphones. This follows from the existing attenuation-only architecture, not a new quality measurement. [Current architecture](../adr/0006-offline-supervised-editable-envelopes.md), [engine investigation](live-plugin-feasibility-2026-10-23.md).

## Current host/API evidence

| Target | Verified fact | Practical implication |
| --- | --- | --- |
| OBS | Its built-in filter supports VST2.x and explicitly excludes VST3. Native `filter_audio` can modify or defer audio; returned new data must survive until the next callback/removal. | A DAW VST3/CLAP binary alone does not provide the desired directly installed OBS filter. A native OBS adapter is a viable API route, subject to buffering/timestamp and installation tests. |
| FL Studio | Official format documentation lists VST3 and CLAP. The wrapper documents threaded processing, smart disable, and VST bridging into another process. | Explicit timing, inactive-instance handling and process-isolation assumptions must be tested. A process-local shared registry must not silently assume bridged instances are present. |
| Resolve/Fairlight | Official product documentation lists Steinberg VST and Mac Audio Unit plugins on clips/tracks, cache/bounce options and adaptive tracks up to 36 channels. | Generic VST workflow exists, but this page alone does not establish exact VST3/version compatibility, CLAP, ARA, or the required insert/sidechain bus layout. The installed target version remains a host proof. |

Sources: [OBS VST filter](https://obsproject.com/kb/vst-2-x-plugin-filter), [OBS source API](https://docs.obsproject.com/reference-sources#c.obs_source_info.filter_audio), [FL supported formats](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins_supported.htm), [FL wrapper](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/wrapper.htm), [Fairlight](https://www.blackmagicdesign.com/products/davinciresolve/fairlight).

VST3 supplies each instance's configured audio buses and an optional process context. When context is present, its project sample time is available; hosts need not split a block exactly at a loop jump. CLAP explicitly allows its steady counter to be instance-specific and transport to be absent. Therefore neither callback counts nor equal local steady counters establish cross-instance alignment. These APIs do not promise that another instance receives the matching block before the current callback must return. Never wait inside instance A for instance B: a serial schedule could prevent B from running. That deadlock scenario is an architectural inference, not an observed host failure. [VST3 ProcessData](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1ProcessData.html), [VST3 ProcessContext](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1ProcessContext.html), [CLAP process contract](https://github.com/free-audio/clap/blob/main/include/clap/process.h).

## Rust wrapper options remain open

NIH-plug provides Rust VST3/CLAP wrappers and a background-task API, making it a candidate to evaluate rather than justification for inventing a wrapper. Context7 resolved `/robbert-vdh/nih-plug` and retrieved `ProcessContext::execute_background`, `AsyncExecutor`, and triple-buffer examples. However, the current upstream README explicitly places the framework in maintenance mode and points to the BillyDM community fork. Inspect and pin the chosen implementation, license/dependency inventory and relevant host support before adoption. Neither branch was built or selected here. Framework task support does not establish an inference deadline or synchronized group delivery. [Upstream README](https://github.com/robbert-vdh/nih-plug), [plugin API source](https://github.com/robbert-vdh/nih-plug/blob/master/src/plugin.rs), [background-task source](https://github.com/robbert-vdh/nih-plug/blob/master/src/context/gui.rs), [upstream-linked fork](https://codeberg.org/BillyDM/nih-plug).

## Smallest useful DAW proof

The owner selected **FL Studio on Windows** for the first working test after this investigation. This settles the first host, while OBS and other DAW targets remain goals. The installed FL Studio version, plugin format/framework and measured delay are still to be established.

Start with a loadable pass-through insert and two instances in the intended host, before a polished UI or full detector integration. The purpose is to answer the uncertain host-delivery question cheaply.

1. Use an explicit shared session and track assignment. Preallocate bounded capture/diagnostic buffers. Keep audio callbacks nonblocking; write logs and fit models elsewhere.
2. Preserve original audio while tracing timestamped input delivery, block sizes, sample rates, process mode and transport discontinuities. Do not collect or publish the owner's recordings as diagnostic logs.
3. Exercise normal playback, seeks, loops, stop/restart, reopen, instance removal/bypass, host smart disable, and offline bounce. Measure maximum cross-track delivery skew and determine whether every output can consume the matching evidence within a fixed, reported delay and bounded memory. No stale samples may cross a reset generation.
4. If that succeeds, split reference fitting from application using the existing model recipe; compare the same samples and references against the application at feature/probability/gain boundaries. Continuous playback equivalence and seek-start behavior are separate checks; require explicit warmup/preroll rules.
5. Connect the fitted inference and gain stage, then validate quiet speech, starts/ends, overlap, late/missing evidence, latency compensation and playback/bounce agreement. Missing or invalid shared evidence preserves the delayed original signal and exposes readiness rather than silently processing from incomplete inputs.

Success means a measured bounded-delivery contract in the named host, intact dry audio on missing evidence, source/group invalidation and a reproducible output comparison. Loading a UI or running one successful playback is insufficient. If grouped delivery fails, stop and present the measured limitation: a single routed multichannel/sidechain analysis point or explicit faster-than-realtime preparation are alternatives, not automatic substitutions for the accepted insert workflow. A mandatory hour-long realtime capture remains excluded.

## Separate smallest OBS proof

Load a native pass-through microphone filter, then add the bounded speech detector/gain path. Measure delay and deadline misses, preserve original audio on late/invalid inference, test quiet speech/onsets/endings/reactions, repeated scene/device activation and A/V synchronization. No numerical latency bound has been accepted; measure candidate settings and let the owner judge the sound/delay tradeoff. Single-input success does not establish multitrack ownership capability.

Not checked: installed host versions, SDK builds, compatibility, callback traces, throughput, latency, A/V sync, listening quality or playback/render parity. No plugin code or dependencies changed.
