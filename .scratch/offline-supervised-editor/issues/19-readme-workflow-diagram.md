# Add a podcast-editor workflow diagram to the README

Status: done
Assignee: Codex
Type: task
Blocked by: none
Spec: ../../offline-supervised-migration/spec.md

## Scope

Add one compact Mermaid diagram to README's Try it section, using podcast-editor language and the existing five-step instructions as its text equivalent. Show synchronized microphone tracks, solo/room-tone examples, explicit Analyze, listening and edits, then the editable project and individual processed WAVs. Do not imply automatic alignment or automatic quality approval.

## Acceptance

- The diagram is in Try it and agrees with its numbered instructions.
- Listening precedes delivery; corrected labels return to Analyze.
- Existing README content and unrelated working-tree changes are preserved.
- Check Markdown fences and git diff whitespace. No app code or dependencies change.

## Comments

2026-10-09: Owner explicitly authorized this diagram. Claimed before editing. The wider visual documentation work is deferred in ticket 20 until the GUI is settled.

2026-10-09: Added the five-step Mermaid flow under Try it, with a label-correction loop to Analyze and listening before delivery. The existing numbered steps remain its text equivalent; a synchronization reminder prevents implying automatic alignment. Added tickets 19/20 to the implementation index. Markdown fence/node and ticket/link checks passed; git diff --check passed. No app code or dependencies changed. Visual rendering in a Markdown viewer was not checked.

2026-10-09: Reclaimed for owner feedback that the diagram was too tall. Changed to left-to-right flow, shortened node labels and retained the relabel/Analyze loop. The numbered instructions supply the full details. Scoped git diff --check passed; visual rendering was not checked. Marked done after the documentation check.
