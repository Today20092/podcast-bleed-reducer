# Gate93 DirectML candidate provenance

On 2026-10-30, downloaded the official
[Microsoft.AI.DirectML 1.15.4 NuGet](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4)
from `https://www.nuget.org/api/v2/package/Microsoft.AI.DirectML/1.15.4`
into a new isolated temporary evidence folder. No compiler, host, model or
installer ran, and the existing runtime cache was not modified.

The archive is 202,292,617 bytes, SHA256
`4e7cb7ddce8cf837a7a75dc029209b520ca0101470fcdf275c1f49736a3615b9`.
Before extraction, all archive entries were checked for rooted paths, parent
traversal, alternate-stream names and Unix symlink attributes. Extracted only
`bin/x64-win/DirectML.dll`, `LICENSE.txt`, `LICENSE-CODE.txt` and
`ThirdPartyNotices.txt`. Their archive paths, byte counts and SHA256 hashes are
pinned in [the reviewed inventory](../licenses/directml-1.15.4-runtime.json).

The official x64 Windows DirectML DLL is 18,527,776 bytes, SHA256
`9c9e6d822561c6c41b90e6994b3e8857cf1d66dbfb1e0c4c799c7c89b4e92da1`.
This matches the previously recorded cached DLL hash exactly. The package helper
compares supplied receipt metadata and every file against this observed
inventory before staging; an arbitrary DLL with a self-consistent forged receipt
does not qualify.

The archive's `LICENSE.txt` agrees with the
[version-specific terms](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4/License).
Section 1(a) describes Windows/Xbox use and distribution within the stated ML
applications/services and games. Sections 1(c) and 3(c) address accompanying
third-party terms and preservation of notices; section 3(e) restricts sharing
beyond the grant. All three official notice files are preserved unchanged in the
candidate. This records the observed terms and bytes for internal candidate
staging; it does not claim completion of the product's release acceptance.

Local evidence is retained at
`C:/Users/User/AppData/Local/Temp/cleanslate-directml-provenance-543dd8c6-4a74-448a-997e-10a43eb173da/`,
including the archive, flat `runtime/` input and `runtime-receipt.json`. These
machine-local files are not required hidden test state: the fixture requires an
explicit `-VerifiedRuntimeDirectory`, and the small pinned inventory travels
with the source. No vendor binary or archive is committed.

## Actual native artifact and MSVC continuation

The actual Gate DLL and both existing ABI hosts subsequently compiled from clean
`2e3adbf12dd9c8f73105c4b99af7a81e3cf699c7`, using the timeout-only wrapper and
pinned Rust 1.99 LLD for each final target. The preserved Gate DLL SHA256 is
`274eb182915506a685389cc9f0ca19b86e055859ab406f8b318f44efe4568e31`.
DLL and VST3-host builds completed normally. CLAP-host Cargo succeeded, but its
wrapper retained one observed VCTIP descendant after Cargo exited 0. Following
coordinator authorization, only that accounted PID was terminated after checking
its exact path, parent and creation time. The wrapper then finalized empty. The
separate intervention record prevents treating this as normal completion;
final-target LLD does not prevent dependency-library MSVC helpers.

Static import inspection identified DirectML and four MSVC runtime DLLs, with
Windows/UCRT dependencies. Inspection of those four runtimes identified no
additional CRT siblings. Using the existing desktop packaging discovery policy,
`vswhere` selected complete VS Build Tools 2022 version 17.14.37328.6, with highest
redist directory `VC/Redist/MSVC/14.44.35112/x64/Microsoft.VC143.CRT`.
All four files report version 14.44.35211.0 and Valid Microsoft Authenticode
signatures. Exact hashes and signer thumbprints are pinned in
[the MSVC inventory](../licenses/msvc-14.44.35211-runtime.json).
They are read from that explicit input directory, never from a system DLL lookup.
The package helper requires the preserved plugin's actual source commit and
expected SHA256 independently of its packaging commit; both identities appear
in the candidate receipt. Changed plugin bytes fail before candidate publication.

The [official REDIST list](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)
includes unmodified files under the VC/redist subtree, excluding debug_nonredist,
subject to the originating license. The
[Build Tools terms](https://visualstudio.microsoft.com/license-terms/vs2022-ga-diagnosticbuildtools/)
describe build/verification rights and separately governed Microsoft components.
The existing MSVC notice retains these links and conditions. Internal candidate
staging is recorded here; release licensing acceptance is not claimed.

Native artifacts, wrapper receipts, static import text, signatures, installation
metadata and the explicit CLAP helper intervention are retained under
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261030-native-2e3adbf/`.
No native host, model or installer has run in this slice. Both baseline host
executables import VCRUNTIME140.dll at startup but do not import DirectML or the
other three Gate CRTs. This preload must be accounted for in independent tests.

Still required: prove candidate-local loading with isolated CWD, sanitized
environment and observed module paths, including preloaded host dependencies.
Existing ABI hosts are unarmed and inherit development search paths. Neither
this filesystem staging check nor those ABI checks demonstrate armed gating,
native editor operation, DAW compatibility or release readiness.
