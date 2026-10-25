# Waveform-first plugin: installed FabFilter reference investigation

The owner rejects the 0.0.3 form-based editor and requests an interface closer to the full CleanSlate application, with visible waveform history that updates during playback. This investigation uses the explicitly requested REA skill on the installed Pro-L 2, Pro-Q 4 and Pro-R 2. It establishes shipped resources, help descriptions and illustrations; it does not establish observed runtime behavior or recovered rendering implementation.

## Reference identity and evidence

All three installed targets are native x86-64 PE libraries under `C:/Program Files/Common Files/VST3/`. REA bound Pro-L 2 with `open_binary`; target-free `inspect_pe_resources` inspected each complete resource tree. The native session was closed afterward. No plugin/DAW was executed and no installed artifact or owner project was modified.

| Installed target | Product version / bytes | SHA-256 | REA resource Evidence ID |
|---|---|---|---|
| FabFilter Pro-L 2.vst3 | 2.25 / 3,015,168 | `033bc8bce956634336eefc5f57a2ec368e6c5d9cd2c753e450bb071740c1bf0f` | `ev_4a5991b8e71ab4bf3f9574b2839975443d857dffae70bfada5f0d69b9b0cfa28` |
| FabFilter Pro-Q 4.vst3 | 4.11 / 4,779,520 | `8f4f09d08cb2b495dadf386af90c83f47f44d36841ba66800098da809223fe15` | `ev_2c0d9b91b999c1afec34c34b603c38966df97549b45a011aa094b0648ddbfb45` |
| FabFilter Pro-R 2.vst3 | 2.05 / 3,914,752 | `de7ce568fb7acd12abf02cfaf73120d8bba2630f1f280cf77b87435148cf1fee` | `ev_b55835f11621bcdbd1b1421610d86d3b35670d945a0526f468398c3a6e9879be` |

Resource coverage was complete for the parser: respectively 137, 361 and 178 resource leaves. Named 2x resources include display/control assets. These observations establish packaged asset identity and scaling variants, not how the renderer uses them. Opaque RCDATA was not treated as decoded artwork or executed UI.

Full inline Evidence records are retained at `C:/Users/User/Documents/Codex/Investigations/fabfilter-plugin-ui-2026-10-24/pe-resources.json`. The server's ledger export after close contained zero records; the separately retained complete records are the evidence source, not that empty export. Earlier Pro-L 2 Ghidra help-string evidence in [the edge-scroll investigation](pro-l2-edge-scroll-2026-10-22.md) matches the exact current binary SHA-256 and need not be regenerated.

Installed help archives under `C:/Program Files/FabFilter/<product>/` were read with the existing 7-Zip/NanaZip command. Selected HTML and `Html/images/overview@2x.jpg` illustrations were extracted to `C:/Users/User/AppData/Local/Temp/cleanslate-fabfilter-ui-20261024/<product>/`. The three overview illustrations were visually inspected. They are manufacturer illustrations shipped with these installations, not screenshots of a running host session.

| Help archive | SHA-256 | Relevant members |
|---|---|---|
| FabFilter Pro-L 2.chm | `f98324c91fe93fba016a8885ea45a5344bfc7ebb8fcedf1e6a168a7ed5ca0c0e` | `Html/using/metering.html`, `Html/images/overview@2x.jpg` |
| FabFilter Pro-Q 4.chm | `e20447ff09c7c603540e9254a1a3ab97769d268e9dc3b4c4c884671e53ee98a7` | `Html/using/eqdisplay.html`, `Html/using/spectrumgrab.html`, `Html/images/overview@2x.jpg` |
| FabFilter Pro-R 2.chm | `d1318597852626a79e10581d1c2c8ebf23450a6da874664b6633b01e6ceee1ce` | `Html/using/spectrumanalyzer.html`, `Html/images/overview@2x.jpg` |

## What the references establish

**Pro-L 2 — time history and effect visibility.** Its shipped metering description and illustration place the evolving level history at the center. Different layers distinguish input, output and gain reduction. The documented display modes vary the time presentation; Slow Down makes room with an edge return, while Infinite compresses accumulating history. This is the closest reference for CleanSlate's temporal view. It is a level history, not proof of a sample-accurate waveform editor or retrospective calibration labeling.

**Pro-Q 4 — interaction on the visual object.** The shipped EQ workflow describes creating/editing bands directly in the large display, with previews and selection-dependent controls. Spectrum Grab can freeze the spectrum so a user can manipulate a visible peak. The overview makes the graph dominant and shows controls close to the selected band. This supports a visual-first workflow; it does not establish that a time-domain training label should be represented as an EQ node.

**Pro-R 2 — meaning encoded in the display.** Its shipped analyzer description distinguishes a live output trace from a decaying reverb visualization. The overview separates the large interactive display, principal sound controls and small supporting toolbar. This supports making the processing effect visible without allowing settings to consume the workspace. Its frequency/decay dimensions are not CleanSlate's source-time axis.

**Shared visible pattern.** These illustrations prioritize the signal/graph, keep persistent controls organized, reserve emphasis for current values/selected objects, and use compact supporting bars. The lesson is interaction and hierarchy; CleanSlate should inherit its own application palette, typography, waveform and label conventions. No FabFilter artwork, brand, proprietary code or exact skin should ship in CleanSlate.

Unknowns: runtime animation curves, actual frame pacing, input latency, implementation/framework internals, accessibility behavior and exact hover/keyboard interactions were not measured in this pass. Installed help explains intended behavior, not execution.

## Recommended CleanSlate workflow

The owner-requested direction is a waveform-centered editor. Proposed primary workflow: **play in the host → freeze visible history → select a time range → label it → learn → audition**. The structured preference question about retrospective selection versus live capture remains optional; do not describe an unanswered choice as owner confirmation.

- Show both grouped microphones as synchronized stacked waveform lanes in the main area. Use CleanSlate's existing application appearance and speaker/room/overlap semantics. Keep Original visible; add Processed and an attenuation envelope when valid processing evidence exists. Unknown/unpaired regions must be marked honestly.
- Start with a bounded rolling history, proposed 30 seconds. This duration is a CleanSlate design choice, not recovered from FabFilter. The display updates as the host delivers audio; freezing the display must not pause, seek or stop the DAW.
- Freezing pins an inspectable snapshot so data being overwritten in the live ring cannot change the selected examples. Dragging selects one common source-time interval across both lanes. Contextual actions label room, mic 1, mic 2 or both. Saved labels remain visible, editable and removable; the user need not time a capture-button press.
- Show concise coverage/readiness and one Learn action beside the selected range. Explain missing classes or insufficient usable duration there. Changing labels invalidates the old learned result until relearned. Keep the existing validation and explicit enable boundary.
- Keep Original / Processed / Removed always reachable as a segmented audition control. Feature attenuation prominently; place fade details behind a disclosure. Put group/member setup behind a compact connection control; keep diagnostic export labeling and report details out of the principal workflow.
- At compact sizes preserve the waveform, label controls and audition actions without a long settings form. Use a larger/resizable editor for inspection, readable time/value scales, clear selected/focus states and simple empty/loading/error messages.

## Required implementation boundary

A streaming plugin receives audio blocks, not automatic access to the DAW's entire clip or original filename. It can show already-delivered history; it must not invent unseen waveform data, seek the host when the user freezes the view, or suggest it has loaded a complete recording. Audio replay/scrubbing inside the plugin is a separate feature and is not implied by a frozen display.

Retrospective labeling needs retained synchronized features as well as waveform bins. A decorative waveform placed over the existing live capture buttons would leave the usability problem intact. Add bounded source-time-indexed peak/min-max bins and native 32 ms paired feature history. Preserve rate/layout, source-time and transport generation; expose gaps and reject incomplete paired selection. Snap training coverage to real complete feature frames and show the resulting usable duration.

Keep callback writes bounded, nonblocking and allocation-free. Transfer display snapshots off the audio thread; GUI painting, fonts and geometry must not acquire audio-thread locks. Repaint only while visible/data changes, keep history bounded, and do not persist raw PCM or waveform caches into the project. Saved labels/calibration follow the existing validation/confirmation rules. Seek/loop/replacement behavior must distinguish discontinuity from matching source-time history instead of concatenating unrelated audio.

Before delivery, test snapshot stability under overwrite, peer dropout, seeks/loops, label changes/removal, missing coverage, min/max waveform correctness and unchanged audio/latency. Inspect the actual native editor at compact and enlarged sizes; headless API dimensions alone cannot qualify appearance. This report changes no DSP, binary, installed link or host project.
