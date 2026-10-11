# Replace Mermaid documentation diagrams

Status: claimed
Assignee: Codex
Type: task
Blocked by: none
Spec: ../../offline-supervised-migration/spec.md

## Scope

Owner authorized on 2026-10-11. Replace all seven Mermaid diagrams in README, docs and scratch roadmap documents using diagram-design. Match the current CleanSlate GUI palette and Segoe UI typography. Preserve all nodes, relationships, branch labels and historical meaning. Ship inline SVG images with accessible HTML sources. No application behavior changes.

## Acceptance

- All seven source diagrams replaced; no remaining Mermaid fences in repository Markdown.
- Preserve source semantics, including historical Nemotron paths, and record a fidelity ledger.
- Check HTML accessibility, SVG XML, local links, geometry and browser rendering.
- Record results and limitations here before marking done.

## Results

2026-10-11: Replaced all seven diagrams in README, ownership and acoustic guides,
and both scratch design records. Added seven editable HTML sources and matching
SVGs under docs/diagrams. All 63 nodes and 65 directed connections are retained.
The historical Nemotron paths remain explicitly historical. The owner selected
the existing GUI palette and typography. Used the verified editor theme and Segoe UI.

Fidelity, style and export instructions: [diagram inventory](../../../docs/diagrams/README.md).

Passed: seven diagram-design HTML self-checks, seven SVG XML parses, exact node-label
and edge comparison, 21 local diagram links, orthogonal routing and no transit
through unrelated nodes, repository-wide scan with no Mermaid fences, and
git diff --check. All seven SVGs were rendered using ImageMagick and visually
inspected. Moved three section headings away from connectors.

No application source, dependencies, audio or project data changed. Runtime
tests were not needed for this documentation-only change.

Remaining acceptance: browser rendering, responsive scrolling and computed font
selection. Chrome is unavailable and the in-app browser rejected file URLs under
its URL policy. No workaround was used for that browser restriction; static SVG
rasterization was used for visual inspection. Status remains claimed until the
browser check is completed.

2026-10-11: Owner authorized shipping these documentation changes with the
reported browser-review limitation. That remaining check stays open.
