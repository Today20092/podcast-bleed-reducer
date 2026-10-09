# Add editor-first documentation visuals after the GUI is settled

Status: needs-triage
Type: task
Deferred: yes; implementation waits for the finished GUI
Blocked by: 16
Spec: ../../offline-supervised-migration/spec.md

## Problem

The README and guides explain labeling, microphone bleed and gain editing mostly in prose. Podcast editors should understand these behaviors before needing model or implementation details. The owner requested one consolidated ticket, rather than capturing an unfinished GUI now.

## Scope for later implementation

- README, below the introduction: one real editor screenshot with concise callouts for tracks, labels, gain and listening controls. Retain the compact Try it workflow from ticket 19.
- docs/editor-workflow.md, Analysis and editing: cropped screenshots of selecting and labeling a range, editing gain points, and preserving speech. Use the final GUI's actual controls, colors and shortcuts.
- docs/ownership-explained.md, You supply examples: waveform examples of solo speech, overlapping speakers and room tone, with the matching active-speaker selections. Explain that labels describe everyone speaking, not just the selected track's speaker.
- docs/acoustic-evidence.md, A simple example: two microphone tracks illustrating direct speech and leakage. Explain why loudness alone cannot establish who owns a track.
- docs/effective-envelopes.md: gain against time over a non-speaking gap, with closing/opening S-shaped fades, attenuation floor, duration and preserved speech marked. Contrast automatic fades with linear manual-point interpolation. Verify curves against the current shared evaluator before drawing them.
- docs/effective-envelopes.md and the ownership guide's gain discussion: automatic, manual, protected and final applied gain on a common timeline. Illustrate actual composition/conflict rules, not a fabricated override order.

Prefer interactive explanations when changing a value helps a podcast editor understand the result. Prefer animation when motion explains an editing action, such as dragging a gain point. Otherwise use static diagrams or screenshots. Choose any interactive hosting/delivery during triage; keep readable static equivalents in the README and Markdown guides, with links to interactive versions where appropriate. Do not create a documentation site solely to satisfy this ticket.

## Readiness

Ticket 16 must be done and the owner must confirm the GUI is settled enough to document. Triage selects the useful visuals and settles any interactive delivery before implementation. This ticket does not block release qualification or imply listening approval.

## Acceptance for later implementation

- Real screenshots come from the finished GUI using docs/gui-development.md and scripts/gui-dev.ps1 with a separate project copy; no invented UI or private recording details.
- Visuals teach actions and audio behavior in plain language for podcast editors. No decorative stock photos or unexplained model diagrams.
- Diagrams agree with the current evaluator, source-validation contract and audition/export behavior. Illustrative audio is labeled as illustrative and does not claim measured quality.
- Every visual has a caption and meaningful alt text or adjacent text equivalent. Meaning does not depend on color alone.
- Static versions remain understandable without motion or scripts. Interactive/animated additions have a specific teaching benefit and a static fallback.
- Verify rendering, legibility, links and light/dark presentation in the intended documentation viewer. Record checks and remaining limitations here.

## Comments

2026-10-09: Owner chose podcast editors as the primary audience, deferred the wider visual pass until the GUI is finished, and preferred interactive then animated then static where interaction or motion helps explain an action. No screenshots or wider visual assets are authorized for implementation in this turn.
