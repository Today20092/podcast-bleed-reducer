# Simplify and center transport controls and workspace heading

Status: done
Type: task
Assignee: Codex
Blocked by: 58
Labels: ready-for-agent

## Owner request

2026-10-22 desktop feedback: Previous, Play/Playing and Stop labels duplicate their icons; state text changes button widths. Remove Playback ready without adding a status dot. Center the transport in the window. Reduce repeated CleanSlate headings and redundant Analysis complete text.

## Scope and acceptance

- Use existing bundled icons in equal fixed-size transport buttons, retaining command hints, shortcuts, enabled states, reference-loop pause/resume and playback preparation/error feedback.
- Center the three-button group relative to the full workspace at wide and compact sizes, independently of the timestamp and listening controls.
- Keep native window identity and one project heading; omit the extra app heading and completed-analysis line while retaining actionable status.
- Use the shared development build/check scripts. Verify widget layout, relevant transport regressions and build; record native checks or explicit limits. No sound, persistence or export behavior changes.

## Results

Claimed before implementation on development. Dependency 58 is done and integrated.

Implemented equal 40×36 icon buttons with retained shortcut hints and enabled states. Equal flexible sides center the group on the full workspace; listening controls move to a separate row below 1400 logical pixels. Project name replaces repeated application branding; completed-analysis and Playback ready text are removed. Preparation/buffering feedback remains beside the timestamp.

Final scripts/check.ps1 -Local passed: 343 tests, zero failures, 20 existing ignored across 38 targets; formatting, strict all-target/all-feature Clippy and script fixtures passed. The new software widget-layout test verifies equal button bounds and window centering at 1024/1390/1600/2560 logical pixels. Initial test-helper compile errors were fixed before this passing run. Native build/check results follow below.

scripts/gui-dev.ps1 passed in 95.54 seconds and opened the retained development project. Native Dark checks passed at 1028×930 and 2560×1392: fixed icon buttons remain centered, both footer arrangements fit and the redundant heading/status are absent. Source is development base 31d81db plus this ticket/61 working changes; executable C:/Users/User/AppData/Local/CleanSlate/BuildCache/debug/rust-app.exe has SHA-256 44665FE930570442F388E20CA7A435D44D12E3FA245461EBF44508569DE36E9B. Light/enlarged OS text, native playing/pause and listening were not repeated; sound/transport commands were unchanged and their existing tests passed.
