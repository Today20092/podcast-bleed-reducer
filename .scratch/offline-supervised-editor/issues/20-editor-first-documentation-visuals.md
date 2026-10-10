# Add editor-first documentation visuals after the GUI is settled

Status: claimed
Assignee: Codex
Type: task
Deferred: wider diagrams remain pending; current development-build screenshots authorized
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

2026-10-09 owner-requested recapture: replaced the README overview with the current zoomed analyzed development window, including New project, visible automatic gain fades/reduction and speaker estimates at the playhead. Added `docs/images/gain-and-speaker-estimates.png` and a workflow caption explaining Active/Inactive percentages as model estimates rather than volume or guaranteed accuracy. Redacted project/source filenames, retained actual plotted data and UI. Visually inspected the new crop; diff whitespace check passed. Captured without modifying playback, labels, gains or project files. Light/dark viewer checks and the wider visual scope remain pending.

2026-10-09 screenshot pass: captured the already-running `CleanSlate - GUI development` window from the gui-dev executable on its separate `.gui-dev` project. Ran Analyze against the existing 36 examples to capture current applied gain. Opened the label popup using a transient range and cancelled it without saving an example. Added four real PNGs under `docs/images/`: analyzed editor overview, selecting speakers, sound controls and listening controls. README and editor-workflow include captions, meaningful alt text and adjacent control explanations. Overview recording filenames are redacted; detail crops exclude them. No recording audio is added. Screenshots describe development behavior, not accepted audio quality.

Checks: visually inspected overview, label and sound crops at native resolution; checked every new image link exists and has alt text; `git diff --check` passed. The overview remains full resolution for zooming, while detail crops keep important controls readable. No app source changes or new dependencies. Remaining: intended Markdown viewer light/dark rendering, gain-point dragging/preservation captures, verified solo/overlap/room examples, bleed explanation, shared-evaluator fade/composition diagrams and interactive-delivery decision. Ticket remains claimed for those acceptance items. Separate ongoing GUI changes may require recapturing screenshots.

2026-10-09: Owner explicitly authorized starting ticket 20 with screenshots from the open development build. Ticket 16 is now done. This supersedes the earlier screenshot deferral for this scoped documentation pass. Capture the real editor and labeling/gain controls, remove private recording filenames, add captions and text equivalents to README/workflow, and preserve the separate GUI development project. Wider behavioral diagrams remain pending; do not mark the whole ticket done from screenshots alone.

2026-10-09: Owner chose podcast editors as the primary audience, deferred the wider visual pass until the GUI is finished, and preferred interactive then animated then static where interaction or motion helps explain an action. No screenshots or wider visual assets are authorized for implementation in this turn.
