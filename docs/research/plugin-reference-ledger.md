# Plugin reference ledger

2026-10-26; [ticket 89](../../.scratch/offline-supervised-editor/issues/89-plugin-reference-rea.md). Static reference research is complete within the [resource contract](../plugin-overnight-plan.md). This is sufficient input for original product designs in ticket 90. Vendor runtime behavior, motion quality and detector equivalence remain unqualified.

The accepted products are separate: Podcast is supervised source-time multitrack processing; Smart Gate is causal single-input speech gating without initial automatic leveling. The historical Session/Live Mic names in earlier reports describe their predecessors.

## Evidence rules and retained sources

**Observed** means source bytes, a parsed table, manufacturer text, or a manufacturer illustration actually inspected. A documented feature is an observation about documentation, not a measured execution. **Inferred** marks a design interpretation. **Unknown** means the evidence cannot answer the question. Negative searches apply only to the stated boundary.

Reused first, rather than repeating expensive analysis:

- [Waveform UI investigation](plugin-waveform-ui-2026-10-24.md): Q4/R2 resources, help and illustrations; Pro-L 2 remains a secondary historical reference.
- [sonible first pass](sonible-static-pass-2026-10-24.md): exact extraction provenance, seven resource inventories, gate ONNX linkage and Speech wrapper evidence.
- [smart:gate follow-up](smartgate-followup-2026-10-25.md): manual controls, bounded string offsets and wrapper traces. Its proposed activation stage remains historical; it is not required work.
- [FabFilter performance investigation](fabfilter-performance-investigation-2026-10-21.md): same-hash retained native evidence, not a new decompilation or current runtime measurement. Later CleanSlate implementation receipts supersede that report's historical proposal status.

Local evidence root `E` is `C:/Users/User/Documents/Codex/Investigations/`. Files below are outside the repository; they contain reference evidence, not distributable product assets.

| Evidence set | Exact location under E | Coverage / provenance |
| --- | --- | --- |
| Fresh identity | `plugin-reference-89-inventory-2026-10-26.json`, reproducer `plugin-reference-89-inventory.ps1` | 29 regular files: 14 supplied originals, 3 installed VST3, 3 installed CHM, 7 retained VST3, Speech/Vocals. Streamed SHA256; all 14 originals match previous source inventory. |
| Fresh PE tables | `plugin-reference-89-pe-imports-2026-10-26.json` | All 10 exact VST3 header/import/export tables; parser `sonible-2026-10-24/inspect-pe.ps1`; source descriptor offsets retained. One file at a time, maximum admitted 200 MiB. No DLL loaded. |
| Fresh C3 resources | `plugin-reference-89-2026-10-26-c3.json` | Complete exported REA record; 64 MiB input / 2048 entry limits, 566 entries / 281 leaves. |
| Fresh manual excerpts | `plugin-reference-89-manuals/manual-<product>.txt` and `manual-<product>-page6.png` | All seven PDFs text-extracted sequentially with PDF-page delimiters; overview page 6 rendered and visually inspected for each. Text line numbers below refer to these retained extractions. |
| Fresh installed help | `plugin-reference-89-help/<Pro-Q 4,Pro-R 2,Pro-C 3>/Html/using/*.html` | Only `Html/using/*.html` and `Html/images/overview@2x.jpg` extracted with installed NanaZip. Focused topics below were read; extraction is not exhaustive topic review. C3 overview visually inspected; Q4/R2 illustrations reused from first report. |
| Earlier sonible resources | `sonible-2026-10-24/pe-resource-evidence.json` | Exact seven VST3 subjects freshly reconciled to inventory; sparse resource tables do not expose complete UI assets. |
| Earlier FabFilter resources | `fabfilter-plugin-ui-2026-10-24/pe-resources.json` | Exact Q4/R2 subjects freshly reconciled. The old empty ledger export is not used. |
| Earlier native records | `fabfilter-performance-2026-10-21/performance-evidence.json` | Retained current-hash Q4/R2/C3 procedure records. Selected IDs and subjects checked locally. |
| Earlier gate detail | `sonible-2026-10-24/smartgate-followup-string-candidates.json`, `smartgate-speech-followup-evidence.json`, `speech-rea-evidence.json` | Bounded source offsets / retained native records. Speech and Vocals freshly rehashed; no model decoded or invoked. |

No new dependency, API integration or platform implementation was selected. Local shipped manuals answer the product behavior questions; no current library API claim is made here.

## Exact artifact identity

Installed primary paths are `C:/Program Files/Common Files/VST3/FabFilter <product>.vst3`. Help paths are `C:/Program Files/FabFilter/<product>/FabFilter <product>.chm`. File version and displayed product version differ; both are retained rather than substituting a similarly named version.

| Product | File version / product version | VST3 bytes / SHA256 | CHM SHA256 |
| --- | --- | --- | --- |
| Pro-Q 4 | 4.1.1.12 / 4.11 | 4,779,520 / `8f4f09d08cb2b495dadf386af90c83f47f44d36841ba66800098da809223fe15` | `e20447ff09c7c603540e9254a1a3ab97769d268e9dc3b4c4c884671e53ee98a7` |
| Pro-R 2 | 2.0.5.24 / 2.05 | 3,914,752 / `de7ce568fb7acd12abf02cfaf73120d8bba2630f1f280cf77b87435148cf1fee` | `d1318597852626a79e10581d1c2c8ebf23450a6da874664b6633b01e6ceee1ce` |
| Pro-C 3 | 3.0.1.53 / 3.01 | 4,141,056 / `134890ede5a2f3f9786ec2009011ac02d6e3d2247a08c286692ad4157bfac42e` | `1f18c18f02946fed774680d063ea55940dbc6e232d3c116f244425f7ddf53974` |

sonible supplied originals are beneath `C:/Users/User/Downloads/To Review/`; folder names below are exact. Manual filenames are `manual-<slug>.pdf`, installer filenames `<slug>_<version>.exe`. Extracted subjects are retained under `E/sonible-2026-10-24/<slug>-<version>/`; the inventory records the exact inner VST3 path (some are bundles, some single files). Versions are verified installer metadata/package identity, also VST3 product metadata where present. Gate/deess VST3 product-version fields are blank; no version was invented from those fields. PDF versions are bound by hash and their supplied package, not an independently exposed PDF version field.

| Product/version; slug | Exact supplied folder | Installer SHA256 | Manual SHA256 |
| --- | --- | --- | --- |
| smart:gate 1.0.1; smartgate | `sonible smartgate win 1.0.1` | `05eb409cd5f3ecbfc0ffe0041e775a522986bc6aa4a92725e922d7bb1fac1210` | `616820ae4be56cd0334aefc90f1d1df0edc219e63cd8115ea80b1c5bc345c703` |
| smart:EQ 4 1.1.1; smarteq4 | `sonible_smarteq4_win_1.1.1` | `f31cf370c05e749e1426257321c1adaf958531a8ec951fb1dd2910d09f312bc5` | `3b1d32c83df72d52a0ddf24ab85bfd31d35d01ab08c6bfa959b70ba83cd12d41` |
| smart:comp 3 1.0.2; smartcomp3 | `sonible smartcomp3 win 1.0.2` | `b3908466fe521e9a5a1bd5b34e778b0a48b5da4c42989de5850cb92609c8e304` | `909d21f1f99773592698e443e90d05a9c884e689543488be0f5531394c317245` |
| smart:deess 1.0.2; smartdeess | `sonible smartdeess win 1.0.2` | `d7d4a5ad60a31ee8bd96df39c783a694924a2ec20e934a500b1ec2f28f4423d0` | `afc1b16774e6f043b7ade7eace422c227826a3b8e10d1b337d569fe60858a824` |
| smart:limit 1.2.0; smartlimit | `sonible smartlimit win 1.2.0` | `65e7f481af47bebd6bffee8d76e38d6801882263a69d17d686c31ea181b630e2` | `10b92080e27162a29bbca120976a074fddd85720a78e983ee1d5d2aa5eff139b` |
| smart:reverb 2 1.0.3; smartreverb2 | `sonible smartreverb2 win 1.0.3` | `979928ca9983e0011e39cafccad34ee514df133d69b9482fee3d92d051c704d6` | `eacbe8b89b9846f5fa8c2a8f317edabda4317583651255cc4dcf6cbd1b989092` |
| smart:chain 1.1.0; smartchain | `sonible smartchain win 1.1.0` | `c55df8312aa3100c25e8c2aa6f9f8aa0f668fbf9523221b1045f031c62708d2f` | `f36f0f4ae8903314d3de5159a095afc11f4d121e6b2b2e03a7db8d916284751d` |

| Extracted VST3 | Bytes / SHA256 | Reused REA resource Evidence ID; entries/leaves |
| --- | --- | --- |
| smartgate | 54,402,264 / `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c` | `ev_deb358ed66572fa68789acbfb6829ea93af4b33d5e711c15007fd7140b64bd2a`; 3/1 |
| smartEQ4 | 67,169,208 / `acd116e1ef8826aacd00a4381104a4c8343baa33b948b1eed87fefb279ab93ec` | `ev_301d29e22a5a0fbb52c6c62a268e0131db18b23ab807e7217cec4a19127571c3`; 6/2 |
| smartcomp3 | 84,871,888 / `48d2362715934552114e0db55144a658f9a53cc2068bbb1de5b16293104bbbe0` | `ev_afe78aaa2f5b044120fb5380aafd72dac400df6912f29e0cb097db52a7481bdb`; 6/2 |
| smartdeess | 66,057,944 / `c17c876ef56dd96ed567ca4529bc5d46319abb370305fb1883309ff4c0599cb5` | `ev_0c81b9f55d1c603bea88634512963d573572cefda96229449757d897d14a5f4c`; 3/1 |
| smartlimit | 57,755,856 / `4b0c6fd46d2977f9f8599f40186120385dce9af8c02099bd2dd43a9d443fe904` | `ev_4f8a554277d969d07e2e88318f06818442f3aedc22206e13e8326bc7d9326f65`; 6/2 |
| smartreverb2 | 78,133,696 / `2ee4aacb5d8ba53b52f68ae7ead1980e68e5ebd1e02e615e4d2bd868a90bd68c` | `ev_782afeb1b4240b3035619c5f7253120baa292ba07bef90215c60e046735185c9`; 6/2 |
| smartchain | 191,923,920 / `f9ceb8e06ac79d1a2d43d2c372a6dc0aca55abb7bc01ab9ae52816f830307ee8` | `ev_75b419daf41c2689942a6d57aa75df18dc31a209dccbda2a6ffe40a0927ec85d`; 6/2 |

Speech.nn: 1,649,152 bytes, SHA256 `4733eb7a586cfed02b0844c392db4025659d301ec95c8d5b3edf6779ecf89f16`; Vocals.nn: same length, `93537dce5abf14371717b9fceabf4dabbec5eab06d80248a6844fe90e913ab50`. Fresh hashes match prior wrapper evidence. No additional profiles were expanded.

FabFilter resource IDs: Q4 `ev_2c0d9b91b999c1afec34c34b603c38966df97549b45a011aa094b0648ddbfb45` (361 leaves), R2 `ev_b55835f11621bcdbd1b1421610d86d3b35670d945a0526f468398c3a6e9879be` (178), C3 `ev_80be1286dedcbf184051bc4f21a3dec472304011d7c7d7377bbde1fe0eb5d962` (281). Coverage is complete for each admitted resource tree, not complete product understanding. Named 2x variants establish packaged variants; opaque payloads do not establish live scaling or rendering quality.

## Bounded question matrix

A = answered for the explicitly documented/static question; P = partially answered; U = unresolved. H is hierarchy/spacing/type/color; D resize/DPI; I hover/focus/drag/zoom; V meters/history/priorities; M motion transitions; G paint/invalidation/cache/threading/graphics; C inter-instance overview/control; B gate readiness/activity/attenuation separation. Every P retains a runtime gap. Every U has a next action below. “No documented feature found” is never proof of absence.

| Exact target | H | D | I | V | M | G | C | B |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Pro-Q 4 4.11 | P | P | P | P | U | P | P | U |
| Pro-R 2 2.05 | P | P | P | P | P | P | U | U |
| Pro-C 3 3.01 | P | P | P | P | P | P | P | U |
| smart:gate 1.0.1 | P | U | P | P | U | P | U | A |
| smart:EQ 4 1.1.1 | P | U | P | P | P | P | P | U |
| smart:comp 3 1.0.2 | P | U | P | P | U | P | P | U |
| smart:deess 1.0.2 | P | U | P | P | U | P | U | U |
| smart:limit 1.2.0 | P | U | P | P | P | P | U | U |
| smart:reverb 2 1.0.3 | P | U | P | P | P | P | P | U |
| smart:chain 1.1.0 | P | U | P | P | U | P | P | P |

All H rows: illustrations show dark surfaces, restrained grids, compact sans-serif labels, separated control groups and brighter selected/processing marks. Exact font family, spacing tokens, contrast compliance and native text rendering remain unknown. Recommendations below are inferred original CleanSlate choices, not recovered vendor design tokens.

All D/I gaps: FabFilter `fullscreenandresize.html` documents normal/fullscreen sizes, remembered scaling and VST3 edge resize; `knobs.html` documents hover illumination/value display, vertical drag, wheel, typed values, fine/reset gestures and host exceptions. No physical DPI, keyboard focus order or accessibility test occurred. The seven sonible manuals were searched for resize/scaling/hover/drag/zoom terms; tooltip settings are documented, but no reliable full resize/DPI contract was established. Do not infer one from overview dimensions.

All U/P motion gaps: named animation, hover zoom, fading or pulsating progress only establishes intended behavior. Durations, easing, smoothness, input latency and presentation cadence remain unknown. All G rows have exact import/resource observations, but actual GPU utilization, full rendering architecture and audio-thread scheduling remain unknown. All nongate B=U cells mean the primary speech-gating contract is not established for that product; do not add gated features from unrelated processors.

Concrete follow-ups (research proposals only, not a request to install/run anything now):

- **N1, all H/D/I/V/M gaps:** an attended single FabFilter host, one plugin at a time, fixed size/scales and audio fixture; capture idle/playback, resize, hover, focus traversal, drag, wheel, selection and transition sequences. Record build, host, scale, refresh and input-to-present timing; distinguish screenshot appearance from presented-frame measurement. For sonible, retain the unknowns unless an already normally installed/activated target is separately selected. No installation or activation is required to continue ticket 90.
- **N2, all G gaps:** first run the original CleanSlate experiments below. Only if vendor implementation still matters, select one retained function/question under an enforceable owned-process memory/time limit; do not launch a shared-provider broad scan. Static evidence alone cannot clear a frame-pacing/GPU/threading claim.
- **N3, C=P/U:** use help-specific contracts below for design; for host qualification later test stable instance identity, missing metadata, duplicate names, peer loss, save/reopen and parameter propagation in one actual format/host. For R2/deess/limit/gate, first search that exact manual's coordination topic before proposing unsupported remote controls; leave cross-instance behavior unknown here.
- **N4, B=U/P and gate runtime gaps:** keep only the established gate lessons below. If a secondary product's gate behavior becomes a real requirement, inspect its exact gating chapter first. For causal CleanSlate qualification use independently labeled quiet speech, transients/noise, competing voices, onsets/tails, block/rate changes and measured latency; vendor category detection is not personal speaker ownership.

## Reusable original UI lessons

| Target and observed source | Finding and limit | Inferred CleanSlate recommendation / remaining action |
| --- | --- | --- |
| Q4: `eqdisplay.html`, `spectrumgrab.html`, `instance-list.html`; prior illustration | Main graph takes priority. Spectrum Grab documents freezing the spectrum while editing; controls follow selected objects. Instance list provides zoom-dependent detail, auto hover zoom, filter/quick jump, pins, minimap and direct remote edits. | Podcast keeps the source-time waveform and labels central; show details for the active selection. Offer a stable inspectable history without pausing/seeking the host. Avoid surprise hover expansion until N1 shows it helps labeling. |
| Q4 instance list, plus C3 `instancelist.html` | Installed Q4 help explicitly covers Q4/C3/DS/G, organized per track; this is richer than a Q4-only assumption. Help notes DAW/format limitations, alphabetical fallback and FL CLAP metadata/order limitations. | Podcast overview needs stable member IDs and explicit missing/disconnected states, with honest fallback names/order. Group UI is not proof of joint analysis, shared source access or synchronization. Validate N3 per host. |
| R2: `spectrumanalyzer.html`, `fullscreenandresize.html`; prior illustration | White live spectrum and yellow decay visualization have distinct meanings; principal controls are separate from supporting bars. Decay fading is documented, not timed. | Give source waveform, applied gain and review/label marks different roles. Keep controls readable at compact sizes and animate only transient feedback. N1 validates resizing/contrast/motion. |
| C3: overview image and `displays.html` | Gray input/output histories, red gain change, optional knee and side meters use related scales; Compact hides animated displays. Main parameters float near the history. | Smart Gate can put input/output and attenuation together with clear scales. Podcast must retain its editing timeline when compact; a compressor's meter-only mode is not an editing contract. Keep detailed timing settings subordinate to primary attenuation. |
| smart:gate: manual p6 illustration; text L160–210, pp7–10 | Activity, gate function, input/output and impact are visibly separate; green source activity is distinct from pink opening state. | Smart Gate shows activity evidence separately from applied attenuation, plus concise readiness and primary amount control. Use CleanSlate's own semantic palette/labels; color must not be the only distinction. |
| smart:EQ 4: manual p6, L198–253; pp20–23 | Direct filter editing and selection details; group view and spectra of group members. Changing display gain range explicitly does not alter processing. | Podcast viewport scale/zoom must never alter sound, labels or source identity. Show group members without conflating display linkage and analysis readiness. N3 checks peer editing and metadata. |
| smart:comp 3: manual p6, pp19–21; L716–751 | Matrix plus history/controls; group remote selection switches the whole interface. Grouping explicitly leaves analysis/compression independent while allowing linked edits and batch learning. | Podcast can edit a selected member from one overview, with selection identity always visible. Explicitly distinguish per-member readiness from shared synchronized analysis. Avoid implying that a batch action creates cross-track evidence. |
| smart:deess: manual p6, L162–214 | Separate sibilant/plosive colors, input/output history, reduction and spectral display; voice reanalysis is explicit. | Keep detected activity and actual reduction distinct and offer Removed audition. Reanalysis guidance is useful; its voiceprint wording is not evidence that CleanSlate recognizes personal identity. No spectral or phoneme UI is needed for current Gate scope. |
| smart:limit: manual p6, pp11–12,17,19; L520–525,641–643 | Main history plus expandable monitoring; confidence area concerns loudness measurement. Manual claims meter updates after parameter edits without replay. | Keep routine status quiet, details expandable and gain edits responsive from compatible cached evidence. Do not relabel loudness confidence as speech certainty or claim the vendor's cache implementation. Validate our reuse in experiment P2. |
| smart:reverb 2: manual p6, pp11–15; L177–240,539–598 | Main controls, style matrix, distance grid and group selection are distinct; remote editing and multiselection have explicit semantics. | Podcast overview should show selected/member/group scope before edits. Use one relevant timeline, not multiple decorative matrices. Preserve member-relative values only where our contract explicitly calls for it; test N3. |
| smart:chain: manual p6, pp9–12; L194–245,344–384 | Group tabs and instance boxes surround the selected-instance editor. Unconfirmed profile and unlearned states are explicit; learning can pause/resume. | Keep missing examples/source coverage visible beside Analyze while setup stays compact. Do not import automatic leveling, seven-module complexity or implicit profile switching into Smart Gate. |

These are design inputs for ticket 90, not new product controls or an instruction to replace established desktop behavior.

## Performance evidence and experiments

**Observed fresh imports:** Q4/R2/C3 each import OPENGL32, dwmapi and GDI32. All seven sonible subjects import OPENGL32/GDI32; chain/comp/EQ/limit/reverb also import d3d11/dwmapi. Records retain each library's descriptor offset and symbols. This is static linkage only: it proves neither an active backend, acceleration speed, actual GPU work, paint cadence nor DSP offload.

**Observed documented configuration:** sonible manuals describe disabling OpenGL for rendering problems (gate p13, EQ p26, comp p23, deess p13, limit p22, reverb p18, chain p24). Gate p13 also describes a multithreading option. These descriptions establish user-facing settings, not measured benefits, worker ownership or callback topology. No such switch is recommended for CleanSlate merely because it exists elsewhere.

**Observed retained native evidence, same fresh subject hashes:**

| Subject / source procedure | REA Evidence ID | Supported interpretation and boundary |
| --- | --- | --- |
| Q4 `0x1801497e0` | `ev_add5d7a198dd9e3b3ee4deed60495b4c1145b45896f076df590128cff2cee096` | Display timing/waits; scheduling evidence, not measured FPS. |
| R2 `0x18013bd00` | `ev_831edfdb2e1a572d2bf761e306de3f15fe908c48831cf7e765550802c4d6271c` | Same bounded scheduling interpretation. |
| C3 `0x1801454d0` | `ev_0da00248f17acb1cb2d71ae593619dfb289f33a53c02c1b6bad69b01bbbe5be6` | Timing/waits and conditional swap; constants are candidate target intervals, not presentation measurements. |
| C3 `0x180144d80` | `ev_731dfd5f8ce44aab899461779e8298da8748a55c4252ce2eb219721062885dfe` | Visibility/non-empty bounds guard drawing; inferred invalidation semantics. Other work before the guard means zero hidden CPU is unproved. |
| Q4 `0x18008a760` | `ev_b70e52b8b3c9cf723bec1ba8dca6d3dccede2c97a4bc711535fea60d8bb12eaa` | Retained texture/full and rectangle uploads; incomplete whole-renderer coverage. |
| R2 `0x1800815d0` | `ev_083db736b65780cf4c4f0c49d231e31867d48c38e06ab666f01f6ad6863f0f1b` | Same bounded texture-update interpretation. |
| C3 `0x180082890` | `ev_39d5b16ffacc1a599661ac5a784a68872d81a5752451f6499cf3f406137555d4` | Same bounded texture-update interpretation; no audio GPU claim. |

No new native analysis was started. Strings/imports alone were not used to reconstruct these functions; the retained procedure records are the stronger source. Full vendor cache keys, paint invalidation graph, thread relationships and runtime mode selection remain unknown.

Original CleanSlate hypotheses, each requiring measurements before extra machinery:

| Experiment | Hypothesis / smallest useful validation |
| --- | --- |
| P1 visible work | Reuse waveform bins/geometry; invalidate only data/viewport/style that changed. Count rebuilds on idle polls, selection, new capture, resize, theme and follow. Record full-frame p95/p99 and input feedback with waveform plus gain, not an isolated strip. Never suppress required redraws to improve a number. |
| P2 control versus analysis | Gain/timing edits should reuse compatible fixed evidence; labels require explicit reanalysis. Record generation IDs, inference counts, edit latency and allocations on long history. Assert sound/display/export share the effective envelope. Vendor responsive metering supplies a UX goal, not a recovered algorithm. |
| P3 visibility and bounded snapshots | Stop unnecessary GUI refresh while hidden; coalesce display updates with bounded immutable snapshots off the audio callback. Measure idle/visible/hidden CPU and callback deadlines during backlog, peer dropout and editor close/reopen. Do not gate audio processing on visibility. |
| P4 ownership and pressure | Retain the global single-heavy-work rule and separately budget UI, analysis and audio. Measure owned working set/commit and callback deadline tails at supported block/rate extremes; check cancellation/stale-result rejection. Add no worker-count setting or renderer replacement without a demonstrated bottleneck. |

Earlier desktop performance work already implemented several related optimizations. Reuse current product patterns after code discovery; this ledger does not declare those changes missing or reopen the owner's closed formal benchmark gate.

## Gate behavior lessons

**Answered at manual-contract level:** smart:gate pp6–11 distinguish source-category activity, initialization, threshold/tolerance, gate opening and output reduction. Activity is not calibrated confidence. Profiles are categories, not personal enrollment. Selecting a profile waits for representative target audio, then becomes initialized; reset clears initialization. Same-session reuse is documented; persistence across host restart is unknown.

Level bias changes amplitude dependence (p9); it does not replace a separate gain controller. Attack, release hold and release have distinct jobs. Tolerance is a soft transition region: the manual's 10% example starts opening at 90% of the threshold. It does not establish recovered two-threshold hysteresis. Impact permits partial suppression (p10); difference audition exposes removed signal (p11). Multiband impact exists in the reference but is outside accepted Smart Gate scope.

**Observed static corroboration:** fresh smartgate import descriptor at file offset `52178204` names `sonible_onnxruntime_v1-15-1.dll` / `OrtGetApiBase`. Prior type-name offsets `52686256` and `52993776` support only a spectral-interface hypothesis. Prior profile-loading string candidates begin around `48706296`; they are not executed xrefs. Speech wrapper constant/data-pointer records `ev_4aa3ca64b008729581c7de4a90d14010eec241868061f9736e17e3c707cec133` and `ev_3da8436e4200c658d971683f49bfc0b2cfafa3168f1b17fcd9010b03166e140f` do not recover a neural graph. Retained follow-up `ev_acc4ffb1c3b905e6ec26ae47df25f76b702685b8637d6d3bfd8e9dbcd2059af7` traces once-only wrapper initialization, not detector training.

**Inferred original design:** keep detector readiness/activity, temporal decision and attenuation separate. Show waiting, ready, stale/error and reset/reanalyze actions honestly. Use finite attenuation, protect word edges and retain Original/Processed/Removed comparisons. Podcast requires source-bound human labels and synchronized cross-track ownership; a speech gate or linked controls cannot supply them. Smart:chain p15 documents smart gating for supported profiles and conventional level gating otherwise; this does not qualify its detector or justify a silent fallback in CleanSlate.

**Unresolved with N4 follow-up:** inference cadence/window/lookahead, neural topology/weights, exact equations, profile persistence, confidence calibration, quiet-speech/noise/other-voice behavior, onset loss, pumping, latency and CPU cost. Use the causal detector harness in ticket 92 to qualify our implementation. No recovered proprietary algorithm/model, quality equivalence or callback-ready Silero adaptation is claimed.

## Resource and completion receipt

Work started from clean scoped branch `codex/plugin-reference-89`, development base `9d8ce6afee67698fe8ddee695db47f6c043830e7`. Dependency 88 was done; 99 was already integrated. Only this report and ticket 89 are changed.

- Initial physical availability about 4.96 GiB; commit headroom about 5.16 GiB. Hash pass snapshot: 4.73 / 4.26 GiB; final snapshot about 5.23 / 4.98 GiB. Machine has 31.9 GiB physical RAM. Snapshots fluctuate and are not peak system measurements.
- 29-file streamed identity pass: 3.17 s, PowerShell peak working set 102.05 MiB. An initial count assertion rejected a filter that missed three single-file VST3 layouts; corrected count is 29, with no substitutions.
- Sequential ten-file import pass: 2.77 s, PowerShell peak working set 292.02 MiB. Raw file buffer maximum 191,923,920 bytes; collected between targets. This is not an enforceable shared MCP memory cap.
- Seven PDF text extractions: 1.97 s; 25/24/14/14/23/19/27 pages respectively for chain/comp/deess/gate/limit/reverb/EQ, each input under 6 MB and fewer than 100 pages. Seven page-6 renders: 2.97 s total; Poppler emitted font-weight warnings, but all seven rendered and were inspected. PDF/CHM parser and shared REA peak memory were not instrumented; no invented peaks.
- Connected REA 6.7.0 reported `open=false`, `cleanup_pending=false` before and after target-free resource inspection. Fresh C3 full evidence export is 218,068 bytes / one record. No provider/native target was opened, so no owned native session needed closing. Prior reports record closure of their sessions.
- Deep shared-provider analysis was skipped because ownership/cap cannot be enforced and available RAM is below the 14 GiB heavy-start threshold. No daemon repair, engine download, broad decompilation, vendor installation/activation, native host, full build, link change, push/tag/release or cleanup occurred.

Checks: exact source reconciliation, retained Evidence ID/subject reconciliation, all report hashes against inventory, local Markdown links and `git diff --check`. No product tests/builds are needed for this documentation-only change. Useful findings have original UI recommendations or bounded experiments; every matrix gap maps to N1–N4. Runtime unavailability is an explicit scoped limitation permitted by ticket 89, not a passing runtime result.

No vendor image, font, branding, code, model weight or extracted asset was added to CleanSlate. References stay outside the repository. No activation/licensing bypass occurred. Remaining risk is treating documented or inferred behavior as runtime qualification; ticket 90 must retain these evidence boundaries.
