# Documentation diagrams

This collection contains seven Mermaid replacements and three audio-behavior teaching figures. Each SVG is linked from its Markdown image so readers can open it at full size. Its matching HTML contains the editable SVG source and a text equivalent.

## Files and fidelity

| Diagram | HTML source | SVG | Nodes | Connections |
| --- | --- | --- | --- | --- |
| README workflow | [workflow.html](workflow.html) | [workflow.svg](workflow.svg) | 5 | 5 |
| Ownership process | [ownership.html](ownership.html) | [ownership.svg](ownership.svg) | 9 | 8 |
| Acoustic evidence | [acoustic-evidence.html](acoustic-evidence.html) | [acoustic-evidence.svg](acoustic-evidence.svg) | 6 | 5 |
| Confirmed workflow record | [planned-workflow.html](planned-workflow.html) | [planned-workflow.svg](planned-workflow.svg) | 7 | 6 |
| Attenuation design record | [planned-attenuation.html](planned-attenuation.html) | [planned-attenuation.svg](planned-attenuation.svg) | 10 | 10 |
| Original analysis proposal | [historical-analysis.html](historical-analysis.html) | [historical-analysis.svg](historical-analysis.svg) | 19 | 24 |
| Original review loop | [historical-workflow.html](historical-workflow.html) | [historical-workflow.svg](historical-workflow.svg) | 7 | 7 |

All 63 source nodes and 65 directed connections are retained. Nothing was merged, collapsed or dropped. The ownership figure retains its three source groups. The two larger figures add labeled sections for readability. The historical records retain their Nemotron paths; they do not describe the current shipping architecture.

The ownership figure groups learning, whole-recording use and gain. Measurement of labeled frames necessarily precedes model training, even though the full-recording measurement node appears in the second group. Follow the [ownership guide's calculation order](../ownership-explained.md#the-process) for the execution sequence. The historical attenuation and analysis figures preserve retired optional/required Nemotron proposals; the [current temporal policy](../temporal-ownership.md) defines today's thresholds and protection behavior.

Node wording is unchanged apart from line wrapping. The attenuation diagram abbreviates the branch labels to “OWN SPEECH” and “ABSENT”; the complete original conditions remain in its text equivalent and accessible description. The relabel loops and dashed optional-evidence connection are retained.

## Audio-behavior teaching figures

| Figure | HTML source | SVG | Guide placement |
| --- | --- | --- | --- |
| Fades, compressed gaps and ignored gaps | [fades-and-gaps.html](fades-and-gaps.html) | [fades-and-gaps.svg](fades-and-gaps.svg) | [Effective envelopes](../effective-envelopes.md) |
| Solo, overlap and room-tone labels | [speaker-label-examples.html](speaker-label-examples.html) | [speaker-label-examples.svg](speaker-label-examples.svg) | [Ownership guide](../ownership-explained.md#1-you-supply-examples) |
| Direct speech and microphone bleed | [microphone-bleed.html](microphone-bleed.html) | [microphone-bleed.svg](microphone-bleed.svg) | [Acoustic evidence](../acoustic-evidence.md#a-simple-example) |

These figures use clearly marked illustrative settings and synthetic waveform envelopes. They are not recording measurements, model results, GUI screenshots or listening-quality evidence. The existing README visuals remain unchanged. Gain-layer composition and an interactive companion remain deferred.

The fade figure uses three aligned line plots with identical time and linear-gain scales. The label figure uses three waveform panels with explicit speaker selections. The bleed figure combines a small acoustic-path diagram with two aligned waveform strips. Continuous curves are sampled densely to preserve their shape, rather than reduced to the milestone points used for ordinary trend charts.

### Gain-data provenance

A local validation probe sampled `AutomaticEnvelope::from_ownership` and `gain_at_time` for long, short and ignored automatic gaps. It sampled 48 kHz source-time envelopes every 4 ms, checked exact preserved gain and attenuation endpoints, and checked the smootherstep midpoint and quarter point. The figure plots the first 2400 ms of each case. Every one of its 1803 points was compared with this output after coordinate rounding. The probe is not part of this documentation-only change.

The synthetic ownership evidence checks gain mechanics; it does not test real-recording detection. The explicit-room-tone exception is additionally covered by the existing effective-envelope test. When the evaluator contract changes, resample it using the settings in the figure's text equivalent, update the HTML polyline coordinates, then regenerate the SVG with the export helper below.

## Design

Created with diagram-design 2.6.74 using faithful detail for a mixed audience. Workflow figures use sequence layouts; evidence figures use architecture layouts; the attenuation policy uses a flowchart. No specialized semantic pattern was needed. Fit-sized canvases keep each source complete without shrinking labels. Figures are static and require no scripts or network resources.

The project-specific appearance follows `editor_theme` and the default font in [src/app.rs](../../src/app.rs):

| Role | Value |
| --- | --- |
| Background | `#202226` |
| Section background | `#26292E` |
| Text | `#E4E7EB` |
| Accent | `#779BC7` |
| Accent fill | `#2B3A4D` |
| Secondary text and connectors | `#ABB4C0` |
| Node border | `#7E8998` |
| Section border | `#4A505A` |
| Typeface | Segoe UI, then Arial, then sans-serif |

The background, text, accent and Segoe UI come directly from the application. The remaining colors are diagram-specific supporting tones. Segoe UI is a system font on Windows. Other systems may substitute the listed fallbacks. Blue emphasizes at most two nodes; labels and arrows carry the meaning independently of color.

## Editing and export

Edit the HTML source first. Export its SVG with the installed diagram-design helper:

```powershell
python "<diagram-design-skill>/scripts/self_check.py" docs/diagrams/workflow.html
python "<diagram-design-skill>/scripts/export_svg.py" docs/diagrams/workflow.html --system-fonts
```

Replace the skill path and diagram filename as needed. The export helper preserves styles, namespaces marker IDs and writes standalone XML. Keep the SVG alongside its HTML.

## Validation

On 2026-10-11, all seven HTML self-checks and SVG XML checks passed. Source-to-output comparison verified every node label and directed connection. All 21 diagram links in the five edited Markdown files resolve. Route checks found no diagonal segments or lines passing through unrelated nodes. All seven SVGs were rasterized locally with ImageMagick and visually inspected; section headings that crossed connectors were repositioned. The final Markdown scan found no Mermaid fences.

Browser rendering, responsive scrolling and computed font selection remain unverified. Chrome was unavailable; the in-app browser's URL policy rejected local-file navigation. Static raster review does not replace those browser checks. The SVGs use an opaque dark background on both light and dark documentation pages.

The three teaching figures passed the same HTML self-checks and SVG XML parsing, and were rasterized with ImageMagick for visual inspection. Their local guide and inventory/proposal links resolve. The local gain-sampling probe passed its built-in assertions, rustfmt and strict Clippy. The existing room-tone/manual-gain regression test passed with one build job after a parallel compilation exceeded Windows paging-file capacity. Browser review remains unverified for these additions as well.
