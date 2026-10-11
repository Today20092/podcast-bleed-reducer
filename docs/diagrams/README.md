# Documentation diagrams

These diagrams replace the seven Mermaid blocks in the README, public guides and historical design records. Each SVG is linked from its Markdown image so readers can open it at full size. Its matching HTML contains the editable SVG source and a text equivalent.

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

Node wording is unchanged apart from line wrapping. The attenuation diagram abbreviates the branch labels to “OWN SPEECH” and “ABSENT”; the complete original conditions remain in its text equivalent and accessible description. The relabel loops and dashed optional-evidence connection are retained.

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
