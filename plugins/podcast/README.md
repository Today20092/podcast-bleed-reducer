# CleanSlate Podcast source candidate

Ticket 95, experimental package 0.0.1. This is source preparation, not a qualified
build or release. The maintained Session engine, source-time capture, waveform,
label/history and conservative cached rendering are compiled directly from
`../fl-probe/src/lib.rs`; no detector or trainer is copied. The Podcast editor
enters the existing Session workspace unconditionally. No live gate mode is
offered. Ticket 96 owns broader workflow/UI acceptance; 80–83 retain their gaps.

| Identity | Podcast | Preserved probe | Reserved Smart Gate |
| --- | --- | --- | --- |
| Cargo package | `cleanslate-podcast` | `cleanslate-fl-probe` | `cleanslate-smart-gate` |
| Display name | CleanSlate Podcast | CleanSlate FL Probe | CleanSlate Smart Gate |
| CLAP | `org.cleanslate.podcast` | `org.cleanslate.fl-probe` | `org.cleanslate.smart-gate` |
| VST3 raw 16 bytes | `CleanSlatePodc01` | `CleanSlateProbe1` | `CleanSlateGate01` |

The shared build script selects identity from `CARGO_PKG_NAME`, never optional
features: `--no-default-features` and `--all-features` retain package identity.
VST3 Windows byte ordering remains the framework's responsibility. Probe IDs,
legacy state migration and its default host runner are preserved.

Podcast state requires persisted `org.cleanslate.podcast.state-version = 1`.
Existing Session field names live inside that product state boundary. A missing,
foreign or future version is **not imported**: nice-plug's void `filter_state`
hook cannot return a rejected load, so it replaces the incoming instance state
with disconnected, processing-disabled defaults, forces Session mode, and saves
`org.cleanslate.podcast.state-rejected = true`. The editor shows the rejection
and recovery instruction. The input project/state file is never rewritten by
this hook. Reopen the original project with its original plugin. There is no
automatic migration or claim of cross-product preset interchange.

Podcast cache root: `<LOCALAPPDATA or XDG_CACHE_HOME or temp>/CleanSlate/Podcast/Sessions/v1`.
Probe retains `CleanSlate/PluginSessions`. Cache keys, source validation,
verified-PCM requirements and labels keep the existing Session contract; a new
product cannot resolve an old cache key through the old directory. Reports use
`%LOCALAPPDATA%/CleanSlate/Podcast/Reports`; probe reports remain `PluginProbe`.
Each product DLL owns its in-process group registry independently.

## Build and acceptance

On Windows run `pwsh -NoProfile -File scripts/build-podcast.ps1`. Every Cargo
command uses the global resource guard and shared locked/offline single-job
cache. Compilation keeps 14 GiB start / 8 GiB reserve / 6 GiB job cap. The script
runs the inherited unit and Session/egui event tests, strict Clippy, builds the
DLL and separately named `podcast_host_check` / `podcast_vst3_host_check` examples,
then runs both actual formats. These checks still need a resource-safe run.
The CLAP host checks the new name/ID, foreign-state disconnection and state
round-trip, and uses a FIFO oracle for mandatory Session latency; the VST3 host
retains its existing latency-aware processing checks. Synthetic host caches
remain isolated by the existing test runner.

Outputs are fresh `dist/podcast-candidate/<run-id>/CleanSlate Podcast.clap` and
`CleanSlate Podcast.vst3/Contents/x86_64-win/CleanSlate Podcast.vst3`, sourced
from `debug/cleanslate_podcast.dll`. The script cannot publish to probe/Gate
output or activate Common Files links. A future deliberate installation would
use separately named `CleanSlate Podcast.clap` / `CleanSlate Podcast.vst3` links;
no link-activation script is supplied by ticket 95. Pass the exact candidate
directory to `scripts/test-plugin-hosts.ps1 -Product Podcast` for a rerun.

No running desktop GUI is needed. The shared `rust-app` path dependency has
`default-features = false`, but its manifest still pulls Iced, rfd and rodio;
appearance, editor_drawing, timeline, waveform, audition and other mixed
UI/cache modules remain compiled by the root library. This is not UI-free core
extraction. Focused extraction is deferred; no renderer or dependency rewrite
is necessary for identity separation.

Documentation lookup: Context7 resolved only upstream `/robbert-vdh/nih-plug`,
not the pinned fork. Exact state-hook, persistent-field, editor and format-ID
contracts were therefore checked against installed nice-plug/core 0.4.2,
nice-plug-egui 0.6.0 and the existing egui 0.37 Session editor. Context7 resolved
egui 0.37.0, then refused its documentation query because the monthly quota was
exhausted; installed egui 0.37.0 `Ui::colored_label` source was checked instead.
No dependency or download was added.
