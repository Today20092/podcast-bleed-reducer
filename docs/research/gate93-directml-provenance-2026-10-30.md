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

Still required: inspect the newly built Gate DLL's imports, establish the exact
MSVC runtime inventory and its official provenance, then prove candidate-local
loading with isolated CWD, sanitized environment and observed module paths.
Existing ABI hosts are unarmed and inherit development search paths. Neither
this filesystem staging check nor those ABI checks demonstrate armed gating,
native editor operation, DAW compatibility or release readiness.
