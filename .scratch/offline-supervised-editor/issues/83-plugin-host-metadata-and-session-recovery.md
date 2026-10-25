# Host track metadata and Session identity recovery

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 80

The candidate discovers participants in the confirmed numeric group and protects saved cache identity, with Mic N labels. Complete optional host-provided names/colors and recovery without treating display metadata as stable speaker/project identity.

- [ ] Show host names/colors when available, with stable explicit microphone assignment and manual fallbacks.
- [ ] Provide deliberate join/fork/recover actions for cloned projects, cache identity conflicts and invalid/missing working caches, preserving source audio and reviewable labels.
- [ ] Qualify supported in-process host discovery; reject unsupported bridged/process-isolated sharing rather than silently combining sources.
- [ ] Test project isolation, duplicate inserts, renamed/reordered tracks and safe host state restore.
