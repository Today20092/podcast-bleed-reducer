# Smart Gate local development delivery

The verified Smart Gate preview is linked into the current user's plugin paths.
New/restored instances remain bypassed and unarmed. This makes verified files
available for host discovery; an actual FL scan, native editor and armed audio
are separate checks, not results of creating links.

| Format | Installed path |
| --- | --- |
| CLAP binary | `C:/Users/User/AppData/Local/Programs/Common/CLAP/CleanSlate Smart Gate/CleanSlate Smart Gate.clap` |
| VST3 bundle | `C:/Users/User/AppData/Local/Programs/Common/VST3/CleanSlate Smart Gate.vst3` |
| VST3 binary | `C:/Users/User/AppData/Local/Programs/Common/VST3/CleanSlate Smart Gate.vst3/Contents/x86_64-win/CleanSlate Smart Gate.vst3` |

Both format files have SHA256
`274eb182915506a685389cc9f0ca19b86e055859ab406f8b318f44efe4568e31`, plugin
source `2e3adbf12dd9c8f73105c4b99af7a81e3cf699c7`. They resolve through two
directory junctions to the preserved, fully inventoried preview:
`C:/Users/User/AppData/Local/CleanSlate/GuiPreviews/SmartGate/2e3adbf-274eb182/`.
The CLAP junction retains its sidecars/notices in its own product directory;
the VST3 junction retains the complete bundle. No shared runtime DLL was copied
into another product's folder. Preview/candidate bytes remain unchanged.

Canonical scoped source merged native preview/host regression `54dfcf5` at
`e68f35d`; delivery helper source is `648f016`. No new compiler or native host ran
in this delivery slice. The earlier normal format regression and standalone
loader proof are preserved separately; this slice adds installed-file evidence.

## Delivery checks and evidence

`scripts/link-smart-gate.ps1` accepts an explicit reviewed preview/hash, verifies
its complete inventory and default flags, validates both plugin binaries and
every destination before linking, and refuses unexpected existing items or
linked/non-directory installation parents. Existing exact links are idempotent.
Failure removes only newly created, still-matching junctions, never their targets.

PowerShell parser and diff checks pass. Public-script filesystem fixtures pass:
wrong reviewed hash rejected without creating destinations; conflicting existing
VST3 directory/sentinel preserved before any CLAP link; both links hash-verified;
repeat delivery creates no links; forced receipt-write failure removes only new
junctions and preserves preview bytes. Fixture:
`%TEMP%/gate-delivery-8b1bd93899c14b2a8028616bd93c0946/`.

The global CLAP destination rejected creation with Access denied. Its exact error
is retained; no unexpected installed item was replaced. The documented user
fallback then succeeded for both formats. Delivery receipt and before/after
Probe/Podcast path/type/target/binary-hash snapshots are under:
`%LOCALAPPDATA%/CleanSlate/DevelopmentInstalls/SmartGate/20261030-648f016-b0d527da2d3e48b2bc7cedc33da3cfba/`.
`installed.json` records both created junctions, exact targets, plugin/source
hashes, safe defaults and remaining limits. `other-products-before.json` and
`other-products-after.json` match. No DAW was closed, project saved, owner audio
changed, public release/tag created or Probe/Podcast installation changed.

## Host discovery and first use

The [official CLAP entry contract](https://raw.githubusercontent.com/free-audio/clap/main/include/clap/entry.h)
defines both Windows user/global directories and recursive `.clap` discovery.
The [official VST3 locations](https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical%2BDocumentation/Locations%2BFormat/Plugin%2BLocations.html)
include `%LOCALAPPDATA%/Programs/Common/VST3` for development and allow links.
Context7 quota was exhausted; these primary sources were read directly.

In FL Studio, use Options → File settings → Manage plugins and Find installed
plugins with Verify plugins. This follows [Image-Line's installation guide](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_externalplugins.htm).
Concrete scan folders are
`C:/Users/User/AppData/Local/Programs/Common/CLAP/` and
`C:/Users/User/AppData/Local/Programs/Common/VST3/`.
Image-Line's current Windows default-location list mentions global paths only;
actual discovery of these standard user locations remains unverified. If the
host omits them, retain the installed files and use an explicit supported scan
path rather than claiming discovery succeeded. No host scan setting was changed
by this slice.

Load **CleanSlate Smart Gate** on a development insert and keep Bypass enabled
for the initial editor/load check. Experimental processing needs both Bypass
off and explicit temporary Arm in the current editor; defaults/saved restoration
never supply arming consent. This is not loudness leveling or OBS-native support.
Native arming/readiness, host PDC, realtime deadlines and listening remain open.

## Rollback

No prior Gate link existed, so rollback removes only the two links created here.
The immutable preview remains available. This reviewed receipt-driven command
has not been executed on the actual installation:

```powershell
$receipt = Get-Content -Raw -LiteralPath "$env:LOCALAPPDATA/CleanSlate/DevelopmentInstalls/SmartGate/20261030-648f016-b0d527da2d3e48b2bc7cedc33da3cfba/installed.json" | ConvertFrom-Json
foreach ($link in $receipt.links) {
    if ($receipt.created_links -notcontains $link.Path) { continue }
    $current = Get-Item -LiteralPath $link.Path -Force -ErrorAction SilentlyContinue
    if (-not $current) { continue }
    if ($current.LinkType -ne 'Junction' -or
        [IO.Path]::GetFullPath([string]$current.Target) -ne [IO.Path]::GetFullPath($link.Target)) {
        throw "Refusing changed installed link: $($link.Path)"
    }
    Remove-Item -LiteralPath $link.Path -Force
}
```

Never recurse through a junction or delete its preview target to undo a link.
