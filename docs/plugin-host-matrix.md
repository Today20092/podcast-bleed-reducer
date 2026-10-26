# Plugin formats and host qualification

Evidence checked 2026-10-26 for [ticket 97](../.scratch/offline-supervised-editor/issues/97-plugin-format-host-coverage.md). The [product plan](plugin-products.md) and [resource contract](plugin-overnight-plan.md) govern implementation. This is a research receipt and bounded plan, not a compatibility announcement.

## Installed Windows hosts

Read-only file-version and uninstall-registry inspection at 2026-10-26 08:16 EDT; no host was launched. Registry entries alone do not establish edition, configuration or usable activation.

| Host | Exact inspected executable | File version | Registry evidence |
| --- | --- | --- | --- |
| OBS Studio | `C:/Program Files/obs-studio/bin/64bit/obs64.exe` | 32.2.2 | OBS Studio 32.2.2 |
| FL Studio 2026 | `C:/Program Files/Image-Line/FL Studio 2026/FL64.exe` | 26.1.1.5547 | FL Studio 2026, 26.1.1.5547 |
| DaVinci Resolve / Fairlight | `C:/Program Files/Blackmagic Design/DaVinci Resolve/Resolve.exe` | 21.1.1.10 | DaVinci Resolve 21.1.10010; Free/Studio edition unknown |

No macOS/Linux installations were inspected. Known paths for FL 2024/2025 were absent; that does not inventory all installations. Physical memory snapshot: 31.92 GiB total, 6.28 GiB available, below the guard's 14 GiB heavy-work start threshold. No prototype/build was attempted; no peak-memory or runtime-performance measurement exists for this research.

## Evidence and adapter boundary

Context7 resolved and queried `/obsproject/obs-studio`, `/steinbergmedia/vst3sdk` and `/websites/image-line_fl-studio-learning_fl-studio-online-manual_html`. Nice-plug resolution returned NIH-plug, a different project; Resolve resolution returned scripting/community projects, not the vendor audio-host manual. Those substitutes were not treated as authority. Primary sources below fill these gaps. Codeberg/docs.rs retrieval failed; installed, already cached nice-plug package metadata and README supplied exact-version evidence without downloading anything.

The graph search for `FlProbe`, `ClapPlugin` and `Vst3Plugin` in the indexed primary project returned no symbols; that desktop index does not cover the retained plugin worktrees. Narrow source reads were used for this gap; no memory-intensive reindex was run. Source snapshots inspected:

| Product workspace | HEAD at inspection | Adapter source |
| --- | --- | --- |
| `C:/Users/User/.codex/worktrees/plugin-smart-gate/podcast-automix-rust-gui` | `3bbda73` | `plugins/fl-probe` 0.0.6 scaffolding |
| `C:/Users/User/.codex/worktrees/plugin-podcast/podcast-automix-rust-gui` | `cf1cc52` | `plugins/fl-probe` 0.0.6 scaffolding |

Both `plugins/fl-probe/Cargo.toml` files pin nice-plug 0.4.2 with `vst3`, `editor`, `assert_process_allocs`; both local plugin lockfiles resolve nice-plug 0.4.2, clap-sys 0.5.0 and vst3 0.3.0. The plugin has a separate workspace/lockfile from desktop. Lockfile SHA256: Gate `bc064bfbd3ce5ff5b7b113b0eef7726f524abc4249f78313704a7e63854015a7`; Podcast `9ccc98ab0d339bb7093fc73d094cfa85bd1d5809ec92aa1ff4a14fa11be96490`. These are source receipts, not hashes of product binaries.

`src/lib.rs` implements `Plugin`, `ClapPlugin`, `Vst3Plugin`, and invokes `nice_export_clap!` / `nice_export_vst3!`. Layouts advertise matched 1-in/1-out and 2-in/2-out, without sidechain buses. No VST2 export or native OBS module exists. The cached upstream README describes VST3/CLAP and standalone exports, and its Cargo metadata/license identifies ISC; [upstream](https://codeberg.org/RustAudio/nice-plug) is the recorded origin. A standalone export is not a VST2 adapter or ready Windows audio-routing application.

`src/shared_audio.rs` owns a process-static `OnceLock<Bus>`; Podcast's synchronized instances depend on the same loaded module/process. Separate plugin processes, format modules or bridge instances do not share this Rust static. A host's ability to load each instance therefore does not prove grouped analysis works. Missing peer evidence must remain visible and preserve audio; no IPC transport is implied.

## Documented support versus product acceptance

“Supported” below means the host/vendor documents that format/API. “Tested” means recorded CleanSlate evidence; “planned” is future qualification; “blocked” states the missing prerequisite. No separate Smart Gate or Podcast release is qualified by the old probe's tests.

| Product / host / platform | Supported by host documentation | Actually tested | Planned | Blocked / limitation |
| --- | --- | --- | --- | --- |
| Smart Gate / FL / Windows x64 | VST3, CLAP, legacy VST [F1] | Historical probe ABI/navigation only [L1]; no causal Gate sound acceptance | Separate Gate VST3/CLAP identity, detector/DSP/UI, native tests | 91–94; current Session probe is not a live gate |
| Smart Gate / OBS / Windows x64 | Built-in VST2.x filter; native module API [O1,O2] | No CleanSlate OBS test | Prefer bounded native-filter feasibility after causal DSP exists | Current exports cannot load directly in built-in filter; VST2 route unresolved |
| Smart Gate / OBS / macOS and Linux | VST2 filter documented on both; native module API [O1,O2] | None | Native module/package qualification after Windows | No platform adapter, package or native gate evidence |
| Smart Gate / FL / macOS x64 and arm64 | VST3, CLAP [F1] | None | Native architecture/editor/audio validation | Packaging and native qualification absent |
| Podcast / FL / Windows x64 | VST3, CLAP [F1] | Probe actual-format checks and native navigation [L1]; broader sound/export/PDC acceptance remains open | Separate Podcast identity; in-process grouped VST3 first, CLAP independently | 95/96 and 80–83; do not inherit native sound acceptance |
| Podcast / Resolve / Windows x64 | Vendor VST effects / clip and track insertion; VST3 added in 17.4 [R1,R2] | None | VST3 track inserts on aligned dedicated-mic tracks | Group scheduling, source-time identity, offline render and PDC unqualified; CLAP not documented |
| Podcast / Resolve / macOS x64 and arm64 | Vendor VST / Mac AU; VST3 introduced in 17.4 [R1,R2] | None | VST3 only, native tests on both architectures | Current vendor platform-specific VST3/architecture details still need native confirmation; AU deferred |
| Podcast / Resolve / Linux | No verified vendor third-party VST3/CLAP support found in this pass | None | Hold this host target pending explicit vendor evidence | Do not infer Linux VST hosting from Linux Resolve availability or Windows support |
| Podcast / FL / macOS x64 and arm64 | VST3, CLAP [F1] | None | Same-process native group tests | No native package/acceptance; macOS has no FL bit bridge [F2] |
| Both / other Linux hosts x64 | VST3/CLAP formats are portable; no host selected or qualified here | None | Native format harness, then a documented compatible host | FL has no native Linux lane in cited format list; Wine/bridges excluded |
| Both / VST2 / any platform | Legacy host support is separate from developer rights | None | Only reopen with a verified supported legal route | Missing adapter and distribution-rights evidence |
| Both / AU and AAX | Outside accepted initial scope | None | Deferred | No adapter/SDK/host work authorized by this matrix |

Historical CLAP export-timing failure and disabled attenuation are retained in [76](../.scratch/offline-supervised-editor/issues/76-fl-studio-shared-audio.md) and [78](../.scratch/offline-supervised-editor/issues/78-supervised-plugin-listening-build.md); later ABI tests do not erase those product acceptance obligations.

## VST2 decision and OBS alternatives

Steinberg's developer announcement ended VST2 SDK distribution/support in October 2018 [S1]. Its discontinuation guidance recommends migration [S2]. Current VST3 SDK documentation says MIT [S3]; this applies to VST3 and does not establish VST2 rights. No evidence of an applicable pre-existing CleanSlate VST2 agreement was supplied or found in these scoped project records. Installed FabFilter/other VST2 binaries confer no development or redistribution permission.

A supported legal VST2 build/distribution route is **not established**. This is an evidence blocker, not a claim that every independently written ABI implementation is unlawful. RustAudio's own archived `vst-rs` README deprecates the crate and warns about distribution licensing [S4]; its code license is insufficient to certify this project's rights. Do not add that crate, fetch unofficial SDK headers, accept agreements, or treat another vendor's build as permission. Reopen only with an applicable rights record, exact maintained adapter provenance/license and review of binary/source redistribution requirements. Until then, VST3/CLAP remain the DAW alternatives and native OBS integration is the direct OBS alternative.

OBS built-in support does **not** accept our VST3/CLAP artifacts: its official filter page explicitly excludes VST3 and lists VST2 discovery locations [O1]. CLAP has no documented built-in loader in the sources checked. The pinned OBS 32.2.2 VST implementation calls the effect inside the host process [O3]; use a native library matching the host OS/architecture. There is no qualified cross-bitness bridge or crash sandbox for CleanSlate in that path.

Bounded alternatives, in order:

1. **Native OBS audio filter, preferred feasibility direction:** after 91–93 supply a distinct causal Gate core, write the smallest module registering an audio filter through `obs_source_info.filter_audio`, sharing maintained DSP rather than duplicating it [O2]. OBS's GPLv2 licensing must be satisfied for the module and linked dependencies [O4]; inventory the exact graph/notices before distribution. The documented API removes reliance on VST2 SDK rights, but it is not an implemented adapter. Start with internal pass-through/silence tests; then an attended disposable OBS scene on 32.2.2. No module was compiled, installed or loaded here.
2. **Existing VST3/CLAP host output into OBS:** possible integration architecture only if an already available audio device/loopback route can carry the processed signal into an OBS audio input. Qualify actual routing, clock/rate, additional buffering and A/V sync, prevent duplicate dry monitoring/feedback, and retain original settings. No suitable configured route was verified. Do not install virtual drivers or promise low latency.
3. **Additional OBS format-host component:** would need its own primary maintained-source, license, architecture, security and latency review, installation consent and disposable-scene check. No component was selected/verified; no automatic wrapper or bridge support is promised.

No new ready adapter ticket was created: VST2 rights are unresolved, and the native OBS direction still depends on the Gate DSP boundary and exact module dependency-license review. Keep this feasibility plan under [85](../.scratch/offline-supervised-editor/issues/85-smart-gate-plugin.md); carve a narrow implementation ticket when those prerequisites are concrete. Research completion does not unblock causal DSP by itself.

## Qualification gates and minimum versions

The following are **proposed first-test baselines**, not advertised minimum support. Lowest supported versions are unset until native packages actually pass; do not advertise all later versions from a single result. Format introduction floors are distinct: FL CLAP starts at 2024.1 [F2], Resolve VST3 at 17.4 [R2]. They are not CleanSlate qualification floors.

| Lane | First candidate baseline | Required package/native gate |
| --- | --- | --- |
| Windows first | Windows 11 x64; FL 26.1.1.5547, Resolve 21.1.1.10; OBS 32.2.2 only after native adapter | Independently named VST3 bundle and CLAP binary, stable separate IDs/state, dependency notices, exact hashes, native scan/editor/audio/save/render; OBS module separately packaged/checked |
| macOS next | macOS 14, Intel x64 and Apple arm64; record actual host versions before testing | Build each architecture, validate bundle layout/deployment target/dylibs, native scan/editor/audio; package signing/notarization only in owner-authorized release lane; no Rosetta result substitutes for native arm64 |
| Linux next | Ubuntu 22.04 x64 as an initial ABI test candidate, not a Resolve support claim | Native `.vst3` bundle / `.clap` shared object, glibc/runtime dependency and display-server checks, native harness then documented compatible host; Resolve held pending vendor evidence |

Cross compilation validates compilation/layout only. Native loading on the declared oldest OS and host establishes a tested floor; record OS/host/CPU architecture, source commit, product/format ID, package path/SHA256 and exceptions. A Linux build cannot establish FL or Resolve hosting. ARM Windows, 32-bit binaries, surround, Wine and bridge configurations are outside the first qualification lane.

Run one heavy task under 88's guard, never concurrent hosts/builds. Before any prototype, require available memory >=14 GiB and its global owned-process lock, 8 GiB reserve, 6 GiB owned-tree limit and 30-minute timeout. Record stopped/skipped reasons and retained logs. The current low-memory snapshot blocks that lane without preventing documentation work.

Windows acceptance sequence, separately for each product and each format:

1. Lightweight actual-format harness first: discover, instantiate, matched mono/stereo negotiation, variable/zero/max block sizes, supported sample-rate changes, finite output, bypass/reset/teardown, state round trip/corruption and advertised latency. Reject unsupported layouts safely; never infer stereo from mono success. Keep allocation/heavy analysis off the callback.
2. Gate: one microphone, mono and stereo source paths, sustained speech/quiet/overlap-like noise examples, bypass/unavailable-detector preservation, attack/release/lookahead and reported-vs-measured latency, start/stop/device reconfiguration and no automatic leveling. OBS channel mapping/mute/monitor order, timestamp/A/V-sync behavior and filter enable/disable need their own checks. If the module adds buffering, preserve timestamps and verify compensation rather than assuming a DAW's PDC exists in OBS.
3. Podcast: 1–8 synchronized dedicated microphone inserts, same format/module/process, no hidden sum of different speakers into one stereo input; verify stereo channel-selection semantics before advertising that workflow. Check labels, seek/loop/discontinuity, readiness/missing peers, capture/cache invalidation, save/reopen, ordinary/offline render, repeated export and PDC/sample alignment against the maintained offline engine. Use aligned track inserts; clip retiming, host cache/bounce and isolated render processes need explicit invalidation/recovery checks before acceptance. A plugin insert does not grant arbitrary access to other tracks or uncaptured source PCM.
4. FL exposes bridged processing [F3]. Disable bridging for the first grouped Podcast lane, and record process placement. Test a deliberately isolated peer as a safe blocked/missing-evidence case, never as supported grouped processing. Mixed VST3/CLAP groups are also unqualified. Gate has no cross-instance PCM requirement, but bridges still add latency/state/editor qualification work. Resolve process placement is unknown until observed; inability to keep synchronized Podcast peers together blocks that host.
5. Only then repeat the native package/audio/state/mono-stereo sequence on macOS/Linux. Listening, native export, presentation/latency and product-quality signoff remain separate from SDK harness success. Use copies of projects/scenes and preserve existing probe binaries/development links throughout.

## Primary source ledger

All external links checked 2026-10-26; documentation may describe newer versions than the installed host. No community reports are used to assert compatibility.

- **O1:** [OBS VST2 filter](https://obsproject.com/kb/vst-2-x-plugin-filter), published 2022-01-12; current page excludes VST3, lists platform discovery paths and cautions about plugin crashes.
- **O2:** [OBS native plugin initialization](https://docs.obsproject.com/plugins) and [source/filter API](https://docs.obsproject.com/reference-sources); Context7 source documentation and current web API reference (33.0.0) establish the API direction, not tested 32.2.2 module compatibility.
- **O3:** [OBS 32.2.2 VST implementation](https://github.com/obsproject/obs-studio/blob/32.2.2/plugins/obs-vst/VSTPlugin.cpp), inspected source, no target execution.
- **O4:** [OBS license](https://github.com/obsproject/obs-studio/blob/master/COPYING), GPLv2 text; exact shipped module/license graph remains a release gate.
- **F1:** [Image-Line supported plugin standards](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins_supported.htm), Windows/macOS formats.
- **F2:** [Image-Line external-plugin installation and compatibility](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_externalplugins.htm), CLAP introduction and bitness/platform constraints.
- **F3:** [FL Wrapper processing/bridging](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/wrapper.htm), bridged process placement; its availability does not qualify CleanSlate sharing.
- **R1:** [Blackmagic Fairlight](https://www.blackmagicdesign.com/products/davinciresolve/fairlight), vendor clip/track effect workflow; the general product page does not certify Linux third-party plugin hosting.
- **R2:** [Blackmagic support release archive](https://www.blackmagicdesign.com/support), DaVinci Resolve 17.4 release note adds Steinberg VST3. No newer release was installed to perform this research.
- **S1:** [Steinberg developer announcement](https://forums.steinberg.net/t/vst-2-sdk-discontinued/201774), vendor announcement by Yvan, 2018-05-17; SDK distribution/support ended October 2018. Replies are not authority.
- **S2:** [Steinberg VST2 discontinuation guidance](https://helpcenter.steinberg.de/hc/en-us/articles/4409561018258-VST-2-Discontinued), updated 2022-03-08; historical transition guidance, not a current promise about any particular host.
- **S3:** [Current Steinberg VST3 SDK licensing](https://github.com/steinbergmedia/vst3sdk/blob/master/README.md), Context7 current SDK reference 3.8.1/MIT. This does not alter the project's pinned adapter or prove old SDK files acquired MIT terms.
- **S4:** [RustAudio vst-rs upstream README](https://github.com/RustAudio/vst-rs), archived/deprecated, warning about VST2 distribution rights.
- **L1:** [Existing probe native navigation receipt](research/plugin-navigation-delivery-2026-10-25.md) and [Session delivery receipt](research/plugin-session-delivery-2026-10-25.md), 2026-10-25; these identify old probe tests/artifacts and unfinished native sound/export obligations. This pass adds no runtime evidence.
