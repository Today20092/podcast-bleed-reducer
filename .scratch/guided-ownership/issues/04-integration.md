# 04: Deliver one guided-ownership Windows build

**Working branch:** codex/cleanslate-ownership
**What to build:** Merge all work, connect editor settings to cached preview and final render, verify source safety/responsiveness/quality, and package one identified EXE.
**Blocked by:** 01, 02, 03
**Status:** ready-for-agent
**Completion:** Delivered the test build from `a2de197`; quality acceptance remains open in issue 05.

- [x] Resolve merge overlaps by intent and preserve all required behaviors.
- [x] Editor passes an immutable ownership-settings snapshot to preview/render; a render completed after settings/sources/protections changed cannot become current for export.
- [x] All-target tests, fmt, strict Clippy, locked release build and relevant render checks pass or pre-existing failures are explicitly differentiated.
- [x] Full-session warm timings and memory measured; no duplicated inference on slider/calibration edits.
- [x] Independent final Standards/Spec review findings resolved.
- [x] Branch and EXE identified; brief calibration instructions and evidence ledger provided.
- [x] User opens the app themselves; no claimed listening sign-off.

## Comments

Branch `codex/cleanslate-ownership`; package details in `../evidence/build.md`. Main and prior executables remain unchanged. Delivery completion does not close the unresolved screenshot suppression or exact-unity unprotected wanted-speech requirements.
