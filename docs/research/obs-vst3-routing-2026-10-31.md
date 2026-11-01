# Windows VST3 routing into OBS Studio

2026-10-31. This note checks how a Windows user can get a 64-bit VST3 effect into an OBS Studio stream or recording, and which path CleanSlate should describe for Smart Gate. It uses OBS's knowledge base, the OBS Studio 33.0.1 tag, the open VST3 pull request, Steinberg's VST2 notice, and first-party host and virtual-device pages. An older project note, [plugin-host-matrix.md](../plugin-host-matrix.md), is treated only as a lead. Its claim that the built-in filter is VST2 and excludes VST3 still matches the current sources below. It does not, by itself, prove the 33.0.1 behavior.

This research did not install OBS Studio, did not install a virtual audio driver, and did not load CleanSlate Smart Gate in OBS or in any host.

## What the Smart Gate preview is

The shipping preview is the JUCE Windows VST3 in `plugins/juce-gate`, not a VST2, CLAP, AU, or AAX build. The user guide says the zip is VST3 only. [Smart Gate](../smart-gate.md)

The processor accepts a matched mono layout or a matched stereo layout. `isBusesLayoutSupported` rejects a mismatch. An earlier pass of this note saw stereo only. [PluginProcessor.cpp](../../plugins/juce-gate/Source/PluginProcessor.cpp)

On startup it loads `cleanslate_smart_gate.dll` from the directory that contains the plug-in binary. A second path, under `plugins/smart-gate/target/release/`, is a build-tree fallback. [PluginProcessor.cpp](../../plugins/juce-gate/Source/PluginProcessor.cpp) The user guide tells a packager to copy that DLL into `CleanSlate Smart Gate.vst3/Contents/x86_64-win/` beside the binary. [Smart Gate](../smart-gate.md)

`prepareToPlay` passes the host rate into `cleanslate_gate_create`. [PluginProcessor.cpp](../../plugins/juce-gate/Source/PluginProcessor.cpp) `GainPolicy::new` accepts only 44100, 48000, and 96000. [gain.rs](../../plugins/smart-gate/src/gain.rs) The user guide states the same three rates. [Smart Gate](../smart-gate.md) Those are repository statements. This pass did not measure them in a host.

## The built-in filter does not load VST3

OBS's current knowledge-base page is still titled "VST 2.x Plugin Filter" and is dated 2022-01-12. It says OBS Studio supports many VST 2.x plug-ins, and that "VST1.x, VST3.x, MIDI control/input in VST plugins, and shell VST plugins are not supported at this time." It also says OBS has not tested every plug-in, that some can crash OBS, and that some are CPU-heavy. [VST 2.x Plugin Filter](https://obsproject.com/kb/vst-2-x-plugin-filter)

On Windows that page says the files use the `.dll` extension and lists these search directories, with `%ProgramFiles%` meaning `C:\Program Files` or `C:\Program Files (x86)`:

- `%ProgramFiles%\Steinberg\VstPlugins\`
- `%ProgramFiles%\Common Files\Steinberg/Shared Components\`
- `%ProgramFiles%\Common Files\VST2\`
- `%ProgramFiles%\Common Files\VSTPlugins\`
- `%ProgramFiles%\VSTPlugins\`

[VST 2.x Plugin Filter](https://obsproject.com/kb/vst-2-x-plugin-filter)

The latest GitHub release on this date is OBS Studio 33.0.1, tagged 29 Oct 2026. The Windows artifact is `OBS-Studio-33.0.1-Windows-x64-Installer.exe`. The 33.0 and 33.0.1 notes do not mention VST3. [OBS Studio 33.0.1](https://github.com/obsproject/obs-studio/releases/tag/33.0.1)

The 33.0.1 tree has `plugins/obs-vst` and does not have `plugins/obs-vst3`. [plugins at 33.0.1](https://github.com/obsproject/obs-studio/tree/33.0.1/plugins)

In that tag, `obs_module_description` returns "VST 2.x Plug-in filter". The registered source id is `vst_filter`, the type is `OBS_SOURCE_TYPE_FILTER`, the output flag is `OBS_SOURCE_AUDIO`, and the audio callback is `filter_audio`. [obs-vst.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/obs-vst.cpp) The English string for the filter name is "VST 2.x Plug-in". [en-US.ini](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/data/locale/en-US.ini)

`fill_out_plugins` is the scanner. On a 64-bit Windows build (`_WIN64`), it searches `ProgramFiles` and `CommonProgramFiles`, not the `(x86)` variables. The directories are:

- `%ProgramFiles%/Steinberg/VstPlugins/`
- `%CommonProgramFiles%/Steinberg/Shared Components/`
- `%CommonProgramFiles%/VST2`
- `%CommonProgramFiles%/Steinberg/VST2`
- `%CommonProgramFiles%/VSTPlugins/`
- `%ProgramFiles%/VSTPlugins/`

The name filter is `*.dll`. The walk includes subdirectories. [obs-vst.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/obs-vst.cpp)

That list is not identical to the knowledge-base list. The shipped scanner adds `%CommonProgramFiles%/Steinberg/VST2`, which the knowledge-base page does not name. Neither list is a VST3 directory, and neither filter is `*.vst3`. [obs-vst.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/obs-vst.cpp) [VST 2.x Plugin Filter](https://obsproject.com/kb/vst-2-x-plugin-filter)

The loaded file is opened with `LoadLibraryW` inside the OBS process. [VSTPlugin-win.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/win/VSTPlugin-win.cpp) The host callback reports VST version 2400, which is the VST 2.4 host version number, and the effect is checked for `kEffectMagic`. Audio is processed with `processReplacing` on the OBS audio callback. The sample rate passed to the effect is `audio_output_get_sample_rate(obs_get_audio())`. [VSTPlugin.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/VSTPlugin.cpp)

OBS Settings, in the 33.0.1 UI form, offers only "44.1 kHz" and "48 kHz". Saving treats every other label as 44100. There is no 96 kHz choice. [OBSBasicSettings.ui](https://github.com/obsproject/obs-studio/blob/33.0.1/frontend/forms/OBSBasicSettings.ui) [OBSBasicSettings.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/frontend/settings/OBSBasicSettings.cpp)

A `.vst3` bundle therefore does not match the scanner. Putting `CleanSlate Smart Gate.vst3` in `C:\Program Files\Common Files\VST3` does not put it on this list. That is what the 33.0.1 source does. This pass did not launch OBS to watch the menu.

## Official VST3 support is not in a release

Pull request 12752, "Add a VST3 host filter to obs-studio", by pkviet, is open. The GitHub API reports `"state": "open"`, `"merged": false`, and `"draft": false`. Its `updated_at` is 2026-10-27T21:22:36Z. It is labeled Seeking Testers, kind/feature, and Windows, macOS, and Linux. [PR 12752 API](https://api.github.com/repos/obsproject/obs-studio/pulls/12752) [PR 12752](https://github.com/obsproject/obs-studio/pull/12752)

The PR text says the host is limited to audio effects, with one main input bus and one main output bus, and that sidechain is supported. It says the work follows Steinberg licensing VST3 SDK 3.8.0 under MIT or a proprietary license, and it points at Steinberg's developer page. [PR 12752](https://github.com/obsproject/obs-studio/pull/12752) That developer page says the open-source VST technology is under the MIT license. [Steinberg developer portal](https://www.steinberg.net/developers/)

The PR is on the milestone "OBS Studio 33.0". That milestone is still open. Its description says items are under consideration for OBS 33.0 and "are not promised to be in the listed release". [PR 12752 API](https://api.github.com/repos/obsproject/obs-studio/pulls/12752) 33.0.1 has already shipped without a VST3 note and without an `obs-vst3` directory. [OBS Studio 33.0.1](https://github.com/obsproject/obs-studio/releases/tag/33.0.1) [plugins at 33.0.1](https://github.com/obsproject/obs-studio/tree/33.0.1/plugins)

The older bounty discussion is still discussion 5074. A maintainer comment on 31 Oct 2025 points at PR 12752 after the license change. [Discussion 5074](https://github.com/obsproject/obs-studio/discussions/5074) Tracking issue 5075 was closed as not planned on 18 Oct 2025. The close event on the issue page does not explain the reason. [Issue 5075](https://github.com/obsproject/obs-studio/issues/5075)

The OBS ideas page for "Add VST 3.x support" still shows the status Planned, links discussion 5074, and links PR 12752 as work that has started and is ready for testing. [Ideas: Add VST 3.x support](https://ideas.obsproject.com/posts/75/add-vst-3-x-support) Planned, and an open pull request, are not a released filter.

Seeking Testers means CI can build artifacts. Those artifacts are not the OBS Studio release. This note does not treat them as something a user guide should install.

## A third-party OBS plug-in can host VST3

atkAudio publishes "Plugin for OBS" at `atkAudio/PluginForObsRelease`. The repository license file is the GNU Affero General Public License version 3. [LICENSE](https://github.com/atkAudio/PluginForObsRelease/blob/main/LICENSE) The README describes PluginHost as a VST3 plug-in host for OBS, with MIDI, sidechain, optional multithreading, and direct hardware routing. PluginHost2 adds multi-plug-in graphs. [README](https://github.com/atkAudio/PluginForObsRelease/)

The OBS Forum resource, not the OBS knowledge base, lists author atkAudio, version 0.36.0, 64-bit, Windows, macOS, and Linux, and a minimum of OBS Studio 31.1.1. The source-code link is the GitHub repository above. The 0.35.0 update text says "unified Windows installer", "auto-detects CPU architecture", and "defaults to the new OBS 33 plugin" location. [OBS Forum resource](https://obsproject.com/forum/resources/atkaudio-plugin.2099/) The GitHub releases index marks 0.36.0 as a pre-release. [Releases](https://github.com/atkAudio/PluginForObsRelease/releases)

The host is an OBS audio filter. `plugin_host.cpp` sets `OBS_SOURCE_TYPE_FILTER` and `OBS_SOURCE_AUDIO`, and the filter name string is "atkAudio PluginHost". [plugin_host.cpp](https://github.com/atkAudio/PluginForObsRelease/blob/main/src/plugin_host.cpp)

A search for atkAudio on `obsproject.com/kb` returned the built-in VST 2.x page, which still says VST3 is not supported. It did not return an OBS knowledge-base article that documents atkAudio. [VST 2.x Plugin Filter](https://obsproject.com/kb/vst-2-x-plugin-filter) OBS's own plug-in loading article describes how OBS loads modules. It does not list atkAudio. [Plugins](https://docs.obsproject.com/plugins)

This project has not installed that filter and has not loaded Smart Gate inside it. The author's README is a vendor claim that PluginHost hosts VST3. It is not a CleanSlate test. Because the built-in VST page warns that a plug-in can crash OBS, an in-process VST3 host has the same class of risk until someone measures this plug-in. [VST 2.x Plugin Filter](https://obsproject.com/kb/vst-2-x-plugin-filter) Do not describe atkAudio as an OBS feature, and do not copy it into CleanSlate. Its license is AGPL-3.0, which is the author's license, not OBS's. [LICENSE](https://github.com/atkAudio/PluginForObsRelease/blob/main/LICENSE)

It is maintained as of October 2026, so it is not an abandoned binary. It is still the wrong thing to call the supported Smart Gate install until a build of Smart Gate has been loaded in a named atkAudio version.

## Route a VST3 host into OBS

OBS can take audio from a device without loading the VST3. The knowledge-base audio-sources page says Audio Input Capture and Audio Output Capture are available on Windows. The user picks a device. The same page warns that an Audio Input or Output Capture source can echo if that device is also selected under Settings, Audio, and says to disable the global device first. "Use Device Timestamps" exists on Windows and defaults to off. [Audio Sources](https://obsproject.com/kb/audio-sources)

OBS 33.0.1 also ships a source whose English name is "Application Audio Capture (BETA)". [win-wasapi en-US.ini](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/win-wasapi/data/locale/en-US.ini) The implementation activates a WASAPI client with `AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK` and `PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE`, on the virtual device `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK`. [win-wasapi.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/win-wasapi/win-wasapi.cpp) Microsoft documents that activation type as "Process loopback activation, allowing for the inclusion or exclusion of audio rendered by the specified process and its child processes." The minimum client listed for the structure is Windows 10 Build 20348. [AUDIOCLIENT_ACTIVATION_TYPE](https://learn.microsoft.com/en-us/windows/win32/api/audioclientactivationparams/ne-audioclientactivationparams-audioclient_activation_type) The OBS pull request that added process capture was merged, and a later comment says the feature has been included since OBS Studio 28.0. [PR 5218](https://github.com/obsproject/obs-studio/pull/5218) The word BETA is still in the 33.0.1 string. This research did not find an OBS or Microsoft sentence that says this capture includes audio sent only through a hardware ASIO driver. Do not claim that it does.

### FL Studio

Image-Line's current manual says FL Studio on Windows supports 32-bit and 64-bit VST 1/2, VST3, and CLAP. VST plug-ins open in the Fruity Wrapper. [Plugin Standards](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins_supported.htm) The install page says a VST3 must be in an exact folder or the scan will miss it. For 64-bit plug-ins on 64-bit Windows, those folders include `..\Program Files\Common Files\VST3`. [Installing Plugins](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_externalplugins.htm) That is the same folder the Smart Gate guide already names. [Smart Gate](../smart-gate.md) This pass did not rescan FL Studio.

Audio settings are Options, Audio settings, or F10. The manual says the Device menu selects the driver, and that a native ASIO driver talks to the interface, while FL Studio ASIO is not a hardware ASIO driver. FL Studio ASIO is a layer on the Windows audio system (WASAPI). It can pick a specific Windows playback or recording device, and it is described as multi-client. The manual's default sample rate is 44100 Hz. Buffer length is the delay between a live input and hearing it, and the manual's Intel/AMD target is about 10 to 20 ms before underruns. [Audio Settings](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/envsettings_audio.htm) Those are Image-Line's numbers, not a Smart Gate measurement.

### REAPER

Cockos's product page for REAPER 7.84, dated October 28, 2026, says REAPER supports third-party plug-ins "including VST, VST3, LV2, AU, CLAP, DX, and JS." It also says the full version can be evaluated for 60 days. [REAPER](https://www.reaper.fm/) The VST SDK extensions page says REAPER supports VST through 2.4 and version 3, and that the extensions document itself applies to 2.x. [Extensions to VST SDK](https://www.reaper.fm/sdk/vst/) The current 7.84 user guide is a 29 MB PDF. This pass did not extract a page from it. [User Guide](https://www.reaper.fm/userguide.php) The troubleshooting guide, which is older than 7.84, tells the user to check Options, Preferences, Audio Device, and says ASIO is preferred and that a larger ASIO buffer trades latency for stability. [Troubleshooting guide](https://www.reaper.fm/guides/ReaperTroubleshootingGuide.pdf) Do not treat that PDF as a 7.84 menu specification.

### Virtual cable

VB-Audio documents VB-CABLE as a Windows driver with no hardware. Audio that enters the CABLE Input playback device is forwarded to the CABLE Output recording device. The driver presents MME, WASAPI, KS, and DirectX endpoints. The reference manual says the playback and recording ends support 44.1 kHz through 96 kHz or 192 kHz, and that the cable converts format automatically when the two applications differ. The package named on the product page is `VBCABLE_Driver_Pack45.zip`, dated October 2024, for XP through Windows 11, 32-bit, 64-bit, and Arm64. [VB-CABLE](https://vb-audio.com/Cable/) [Reference manual](https://vb-audio.com/Cable/VBCABLE_ReferenceManual.pdf) The pages retrieved do not give one millisecond figure for this route. Do not invent one. This project has not installed the driver.

A practical chain that uses only those vendor statements:

1. Install the Smart Gate bundle in `C:\Program Files\Common Files\VST3`, with `cleanslate_smart_gate.dll` beside the binary inside the bundle. [Smart Gate](../smart-gate.md)
2. Scan it in FL Studio or REAPER, on a stereo track. FL documents the VST3 folder. REAPER's product page documents VST3. [Installing Plugins](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_externalplugins.htm) [REAPER](https://www.reaper.fm/)
3. In FL Studio, prefer FL Studio ASIO when the processed audio must enter a Windows playback device such as CABLE Input. Image-Line says that driver sits on WASAPI and can select an installed playback device. A manufacturer ASIO driver is the lower-latency choice for the interface, and this note does not claim OBS process loopback hears it. [Audio Settings](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/envsettings_audio.htm)
4. Install VB-CABLE. Send the host output to CABLE Input. In OBS, add Audio Input Capture on CABLE Output. Do not also select that device under Settings, Audio. [VB-CABLE](https://vb-audio.com/Cable/) [Audio Sources](https://obsproject.com/kb/audio-sources)
5. Set the host, the cable format, and OBS to the same rate. OBS 33.0.1 offers 44.1 kHz and 48 kHz only. Smart Gate's gain policy also accepts 96 kHz, but OBS cannot be set to that rate in this UI. [OBSBasicSettings.ui](https://github.com/obsproject/obs-studio/blob/33.0.1/frontend/forms/OBSBasicSettings.ui) [gain.rs](../../plugins/smart-gate/src/gain.rs)

Mute or disable the dry microphone inside OBS. Otherwise the stream hears the unprocessed mic plus the gated copy. The echo warning on the audio-sources page is the closest official statement. [Audio Sources](https://obsproject.com/kb/audio-sources)

For monitoring, the 33.0.1 English UI strings include "Audio Monitoring", "Monitor Off", "Monitor Only (mute output)", and "Monitoring Enabled". [en-US.ini](https://github.com/obsproject/obs-studio/blob/33.0.1/frontend/data/locale/en-US.ini) Hearing the mic in the host, on a real playback device, avoids using OBS as the monitor. Hearing it by monitoring the cable inside OBS adds whatever delay OBS monitoring adds. This pass did not measure either path. FL's documented live delay starts at the audio buffer, about 10 to 20 ms on Intel and AMD when the buffer is stable, before the cable and before OBS. [Audio Settings](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/envsettings_audio.htm)

Application Audio Capture is the route with one fewer driver: capture the host process, and do not capture the dry mic. It stays labeled BETA in 33.0.1. [win-wasapi en-US.ini](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/win-wasapi/data/locale/en-US.ini) OBS issue 8064 records reports of long-session distortion on this capture. Those reports are not a CleanSlate test. [Issue 8064](https://github.com/obsproject/obs-studio/issues/8064) Use it only when the host is actually rendering through Windows audio, which FL documents for FL Studio ASIO and not for a native ASIO driver. [Audio Settings](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/envsettings_audio.htm) [AUDIOCLIENT_ACTIVATION_TYPE](https://learn.microsoft.com/en-us/windows/win32/api/audioclientactivationparams/ne-audioclientactivationparams-audioclient_activation_type)

Step count, as documentation, not as a timed test:

| Path | Extra installs beyond Smart Gate | What the user touches | Latency evidence |
| --- | --- | --- | --- |
| Built-in OBS VST filter | None | One audio filter | Not available. The filter does not scan VST3. [obs-vst.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/obs-vst.cpp) |
| atkAudio PluginHost | That OBS module | Add the filter, then open the VST3 inside it | Vendor claim of in-process hosting. No Smart Gate measurement. [README](https://github.com/atkAudio/PluginForObsRelease/) |
| FL Studio or REAPER, then VB-CABLE, then Audio Input Capture | The host, if missing, and the VB-Audio driver | Scan VST3, set the host output, add one OBS input, disable the dry mic | FL documents a buffer of about 10 to 20 ms before other stages. Cable and OBS delays were not measured. [Audio Settings](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/envsettings_audio.htm) |
| FL Studio ASIO, then Application Audio Capture | The host, if missing | Scan VST3, set a Windows-audio output, add the beta source, disable the dry mic | Still beta in 33.0.1. No Smart Gate measurement. [win-wasapi en-US.ini](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/win-wasapi/data/locale/en-US.ini) |

## A native OBS filter is a later engineering path

OBS 33.0.1's source reference documents `obs_source_info.filter_audio`. It is called to filter raw audio, only for audio filters. The callback may modify the buffer it receives, or return new data that must stay valid until the next call or until the filter is removed. [Source API Reference](https://docs.obsproject.com/reference-sources) The plug-in tutorial says a native module includes `obs-module.h`, uses `OBS_DECLARE_MODULE()`, and registers sources from `obs_module_load`. [Plugins](https://docs.obsproject.com/plugins) The VST2 module is one example of that registration. [obs-vst.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/obs-vst.cpp)

There is no CleanSlate OBS module in this repository. A user cannot install this path today.

OBS publishes its license as the GNU General Public License, version 2, in `COPYING` at the 33.0.1 tag. Section 0 says a "work based on the Program" includes a derivative that contains the Program or a portion of it. Section 2 says a distributed modification must be licensed to all third parties under the terms of this license. [COPYING](https://github.com/obsproject/obs-studio/blob/33.0.1/COPYING) The `obs-vst` file header adds "either version 2 of the License, or (at your option) any later version." [VSTPlugin.cpp](https://github.com/obsproject/obs-studio/blob/33.0.1/plugins/obs-vst/VSTPlugin.cpp) A future CleanSlate module that is distributed as a work based on OBS has to satisfy that license, including for code it links into the module. This note does not inventory that graph. The plug-in tutorial does not, on the pages retrieved, restate the GPL in its own words. [Plugins](https://docs.obsproject.com/plugins)

## Do not ship a VST2 wrapper

Steinberg's Japanese news post dated 2018.05.18 says that from October 2018 VST2 maintenance, technical support, and SDK distribution end, and that Steinberg recommends moving to VST3. [VST 2 notice](https://japan.steinberg.net/jp/news_events/news_list/detail/article/vst-2-4747.html) The English developer announcement on the Steinberg forum, 16 May 2018, says the same thing: from October 2018 "we are closing down the second version of VST for good," and the VST2 subset of the VST3 SDK will be omitted. [VST 2 SDK discontinued](https://forums.steinberg.net/t/vst-2-sdk-discontinued/201774) A later Steinberg staff reply, 2020, says that after talking with their lawyers it is not possible to license VST2 for a new host either, and that Steinberg no longer enters into VST2 license agreements. [VST2 host license thread](https://forums.steinberg.net/t/can-i-create-vst-2-support-for-host-application-vst-3-license/202012)

CleanSlate therefore has no documented legal path to build or ship a VST2 plug-in, or a VST2 shell whose purpose is to load this VST3 inside OBS's VST2 filter. A third-party product that already has its own VST2 license is that vendor's product. This pass did not review one, and it is not a CleanSlate shipping format. The current VST3 SDK license does not restore VST2 rights. [Steinberg developer portal](https://www.steinberg.net/developers/)

## A VST2 shell that hosts VST3

Checked 2026-10-31 after the cable recommendation. These are vendor pages. None of these shells was installed, and Smart Gate was not loaded in any of them.

Waves documents the pattern people see in OBS. The article "How to Use Waves Plugins in OBS Studio", dated Jun 24, 2024, says to add a filter of type VST 2.x Plug-In and choose Waves StudioRack for OBS. On Windows that file is `Waves StudioRack for OBS.dll` in `C:\Program Files\VSTPlugIns`. The same article says StudioVerse Audio Effects then opens and "All Waves plug-ins installed on your system will be available." [How to Use Waves Plugins in OBS Studio](https://www.waves.com/support/how-to-use-waves-plugins-in-obs)

The Waves supported-hosts data lists OBS Studio version 32, "Via StudioVerse, Stereo only", on Windows and Mac. It does not list OBS Studio 33. The page says the listed DAW versions are the latest Waves tested, and that a newer sub-version works unless stated otherwise. [Supported Hosts](https://www.waves.com/support/tech-specs/supported-hosts)

A separate Waves article, also dated Jun 24, 2024, says StudioRack V14 and StudioVerse Audio Effects can scan non-Waves VST3 plug-ins from `C:\Program Files\Common Files\VST3`. It tells the reader to load StudioRack on a channel insert. It does not say that the OBS-only file, Waves StudioRack for OBS, has that scan. [How to Scan Non-Waves VST3 Plugins in StudioRack](https://www.waves.com/support/how-to-scan-non-waves-vst3-plugins-in-studiorack) So Waves is the OBS shell people remember. Their OBS instructions name Waves plug-ins. Their third-party VST3 scan is documented for StudioRack in a DAW, not for the OBS shell.

Blue Cat's PatchWork is the Patcher-shaped product. The product page says it ships as VST, VST3, AAX, and Audio Unit, and that one instance hosts up to 64 third-party VST, VST3, or Audio Unit plug-ins. It also ships as a standalone application. The listed price on that page is 89 euros or 89 dollars. [PatchWork](https://www.bluecataudio.com/Products/Product_PatchWork/) A 2016 changelog line says "V1.73 (2016/05/10) VST3 plug-ins hosting." [PatchWork changelog](https://www.bluecataudio.com/Products/Product_PatchWork/) FL Studio Patcher is not this. Patcher runs inside FL Studio. It is not a VST2 file OBS scans.

DDMF Metaplugin is the same shape. The product page says it loads other VST and VST3 plug-ins, and that the formats it installs include VST and VST3. The manual says the same, and says it can wrap one format inside another. [Metaplugin](https://ddmf.eu/metaplugin-chainer-vst-au-rtas-aax-wrapper/) [Metaplugin manual](https://ddmf.eu/pdfmanuals/MetapluginManual.pdf) DDMF also sells Virtual Audio Stream, a Windows driver plus a VST rack. That is another cable-plus-host product, not an OBS filter. [DDMF home](https://ddmf.eu/)

Kushview Element is free under GPLv3 and hosts VST and VST3. The project says it runs standalone or as a plug-in. The format table on that page is what Element loads, not a statement that OBS can scan Element's own file as a VST2 dll. [Element on GitHub](https://github.com/kushview/element) [Element product page](https://kushview.net/element/) A standalone Element session still needs a route into OBS.

Smart Gate accepts a stereo pair only. Waves' OBS listing is stereo only. A mono OBS microphone source may not present the bus these shells expect. That match was not tested.

## OBS VST3 and CLAP, checked again

Pull request 12752 remains the official VST3 host. A third-party tracker of that PR quotes an October 11, 2026 update from pkviet: the scanner moved to an external binary, and VST3 stays limited to audio effects with one main input bus and one main output bus. The same page says the pull request is open. [PR 12752 tracker](https://obs-versions.com/community/pulls/12752) OBS 33.0.0 shipped on October 23, 2026, and its feature list does not mention VST3. [OBS Studio 33.0.0 notes](https://newreleases.io/project/github/obsproject/obs-studio/release/33.0.0) The GitHub tag 33.0.1 still has no `plugins/obs-vst3` directory, as recorded above.

A search of current OBS release material did not find a CLAP host filter. The 2022 bounty discussion includes comments proposing CLAP, including one from falkTX that mentions Carla. Those comments are proposals. [Discussion 5074](https://github.com/obsproject/obs-studio/discussions/5074) CLAP itself is a separate plug-in format maintained by free-audio. Its wrapper project can present a CLAP plug-in as a VST3. It does not add a CLAP loader to OBS. [clap-wrapper](https://github.com/free-audio/clap-wrapper)

## Recommendation

Document the cable path in the user guide now. Install the Windows x64 VST3 in a host whose vendor documents VST3. FL Studio is the host the current Smart Gate guide already names, and its manual names both VST3 and `C:\Program Files\Common Files\VST3`. REAPER 7.84 is the documented alternative when the user does not have FL Studio. Send that host's processed output to VB-Audio's CABLE Input, and capture CABLE Output with OBS Audio Input Capture. Keep the dry microphone out of OBS. Use 48 kHz, or 44.1 kHz, at every stage OBS can actually set. Say that the user is hearing the host's buffer plus an unmeasured cable and capture delay, and that Smart Gate has not been run through this chain.

Also name the shell path, and do not make it the default. Waves StudioRack for OBS is a real VST2 that OBS loads, and Waves' OBS article lists Waves plug-ins after that. Blue Cat PatchWork and DDMF Metaplugin publish a VST build that hosts VST3, which is the Patcher shape. A person who already owns one of those can try it. Smart Gate has not been loaded in any of them, and a shell runs inside the OBS process. FL Studio Patcher does not load in OBS.

The path that would make install easy later is a VST3 filter inside OBS, so the user adds one audio filter and never installs a DAW or a virtual driver. The official candidate is PR 12752. It is not merged, it is not in 33.0.1, and the milestone text says listing is not a promise. Waiting for that release costs a dependency on OBS, not a CleanSlate installer. The CleanSlate-owned version of the same idea is a native module registered with `obs_source_info.filter_audio`. That costs an OBS module build, a GPL review of the module and anything it links, a stereo feed at 44.1 or 48 kHz, and a real OBS test. atkAudio is the existing third-party version of the easy install. It is maintained and AGPL-3.0, OBS does not document it, and Smart Gate has not been loaded in it. Do not call it supported.

Do not claim that Smart Gate appears in OBS's built-in plug-in list. Do not claim a TikTok Live plug-in. Do not claim a one-click installer. None of those builds exist. The preview remains a VST3 zip. [Smart Gate](../smart-gate.md)

This research did not install OBS or load Smart Gate.
