# 05: Prepare and audition without blocking the editor

**Working branch:** codex/responsive-playback
**Blocked by:** 01, 02, 03
**Status:** ready-for-agent
**What to build:** Background audio preparation and efficient equivalent playback of shared envelopes.

- [ ] Starting audition does not read all stems synchronously on the UI thread.
- [ ] Repeated audition avoids needless full-source reloads with bounded/shared resources and valid identities.
- [ ] Remove per-output-sample lock overhead; sequential curve evaluation preserves output within documented numeric tolerance.
- [ ] Preserve Original/Processed/Removed playhead, selection, source-minus-processed, monitor gain and envelope replacement.
- [ ] Test cancel/relink/error paths and playback equivalence; document memory behavior.
