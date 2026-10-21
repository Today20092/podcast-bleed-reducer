# 02: Retain unchanged waveform geometry during worker polling

Status: ready-for-agent
Type: task
Blocked by: None (can start immediately)
Source proposal: P1

**What to build:** Keep the existing waveform/gain caches valid when a background poll or unrelated interface action cannot change their contents, while every visible data, style, size and viewport change still redraws correctly.

## Acceptance criteria

- [ ] An unchanged background poll does not rebuild static waveform/gain geometry; record baseline and changed rebuild/timing evidence.
- [ ] Source/analysis/preview completion, selection and manual edits, theme/resize, zoom and viewport changes remain visible without stale geometry.
- [ ] Playhead and transient overlays continue to move without forcing unrelated static geometry work; keep lock, fit and middle-pan semantics.
- [ ] Use observable application/drawing behavior at confirmed seams to demonstrate the regression before the fix and verify it afterwards.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

