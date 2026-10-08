param([int]$Jobs = 2)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$revision = '8642eaa5cc51efbc17ad0f3e433944ba858a873f'
$source = Join-Path $root '.native-build\NeMo-Speech.cpp'
$destination = Join-Path $root 'native-runtime'
if (-not (Test-Path -LiteralPath (Join-Path $source '.git'))) {
    git clone --no-checkout https://github.com/NVIDIA/NeMo-Speech.cpp.git $source
    if ($LASTEXITCODE -ne 0) { throw 'Native source checkout failed' }
}
git -C $source checkout --detach $revision
if ($LASTEXITCODE -ne 0) { throw 'Pinned native revision unavailable' }
git -C $source submodule update --init llama.cpp
if ($LASTEXITCODE -ne 0) { throw 'Pinned ggml dependency unavailable' }

# Upstream repeats PATH three times; deduplicate before vcvars reaches cmd's line limit.
$driver = Join-Path $source 'scripts\windows\build.ps1'
$original = '$env:Path = "$machine;$user;$process"'
$patched = '$env:Path = (($machine + '';'' + $user + '';'' + $process).Split('';'') | Where-Object { $_ } | Select-Object -Unique) -join '';'''
$content = [IO.File]::ReadAllText($driver)
if (-not ($content.Contains($original) -or $content.Contains($patched))) { throw 'Pinned PATH patch no longer matches' }
[IO.File]::WriteAllText($driver, $content.Replace($original, $patched))

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'Install Visual Studio 2022 C++ Build Tools and Windows SDK for development builds' }
$cmakeTools = Join-Path $vs 'Common7\IDE\CommonExtensions\Microsoft\CMake'
$env:PATH = (Join-Path $cmakeTools 'CMake\bin') + ';' + (Join-Path $cmakeTools 'Ninja') + ';' + $env:PATH
$env:VCPKG_MAX_CONCURRENCY = [string]$Jobs
$env:CMAKE_BUILD_PARALLEL_LEVEL = [string]$Jobs
$flags = @('-DNEMO_SPEECH_BUILD_ASR=OFF', '-DNEMO_SPEECH_BUILD_CLI=OFF',
    '-DNEMO_SPEECH_BUILD_MIC_CAPTURE=OFF', '-DGGML_NATIVE=OFF', '-DGGML_AVX512=OFF',
    '-DGGML_AVX2=ON', '-DGGML_FMA=ON', '-DGGML_F16C=ON', '-DGGML_OPENMP=OFF')
& $driver -Backend cpu -AsrOnly -Jobs $Jobs -CMakeArgs $flags
if ($LASTEXITCODE -ne 0) { throw 'Pinned native CPU build failed' }
$build = Join-Path $source 'build-cpu-asr'
cmake --install $build --prefix $destination
if ($LASTEXITCODE -ne 0) { throw 'Native SDK installation failed' }

# Package the complete installed CPU dependency closure, including MSVC runtime.
$bin = Join-Path $destination 'bin'
$runtime = Join-Path $vs 'VC\Redist\MSVC'
$redist = Get-ChildItem -LiteralPath $runtime -Directory |
    Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName 'x64\Microsoft.VC143.CRT') } |
    Sort-Object Name -Descending | Select-Object -First 1
if (-not $redist) { throw 'MSVC redistributable DLLs missing from Build Tools installation' }
$crt = Join-Path $redist.FullName 'x64\Microsoft.VC143.CRT'
Copy-Item -Path (Join-Path $crt '*.dll') -Destination $bin
$licenses = Join-Path $destination 'share\licenses'
New-Item -ItemType Directory -Force -Path $licenses | Out-Null
# vcpkg copyright files retain the actual notices of statically linked dependencies.
$vcpkgLicenses = Join-Path $build 'vcpkg_installed\x64-windows-static-md\share'
foreach ($dependency in Get-ChildItem -LiteralPath $vcpkgLicenses -Directory) {
    $copyright = Join-Path $dependency.FullName 'copyright'
    if (Test-Path -LiteralPath $copyright) {
        $folder = Join-Path $licenses ('vcpkg-' + $dependency.Name)
        New-Item -ItemType Directory -Force -Path $folder | Out-Null
        Copy-Item -LiteralPath $copyright -Destination (Join-Path $folder 'copyright')
    }
}
Copy-Item -LiteralPath (Join-Path $source 'LICENSE') -Destination (Join-Path $licenses 'NeMo-Speech.cpp-Apache-2.0.txt')
Copy-Item -LiteralPath (Join-Path $source 'llama.cpp\LICENSE') -Destination (Join-Path $licenses 'ggml-llama-MIT.txt')
Copy-Item -LiteralPath (Join-Path $vs 'Licenses\BuildTools\1033\ThirdPartyNotices.txt') -Destination (Join-Path $licenses 'MSVC-third-party-notices.txt')
Invoke-WebRequest -Uri 'https://visualstudio.microsoft.com/license-terms/vs2022-cruntime/' -OutFile (Join-Path $licenses 'MSVC-runtime-license.html')
$notices = Join-Path $PSScriptRoot '..\docs\runtime-notices.md'
Copy-Item -LiteralPath $notices -Destination (Join-Path $licenses 'CleanSlate-runtime-notices.md')
$binaryHashes = [ordered]@{}
foreach ($dll in Get-ChildItem -LiteralPath $bin -Filter '*.dll' | Sort-Object Name) {
    $binaryHashes[$dll.Name] = (Get-FileHash -LiteralPath $dll.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
}
$manifest = [ordered]@{ revision = $revision; recipe_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant(); binaries = $binaryHashes }
$manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $bin 'runtime-manifest.json') -Encoding utf8NoBOM
@("Native revision: $revision", "ggml gitlink: $(& git -C $source ls-tree HEAD llama.cpp)",
    "vcpkg: 9e593bb18ea69cc5095e012465dcd675a822ed0d (2026.07.29)",
    "MSVC installation: $vs", "MSVC CRT: $($redist.Name)", "CMake: $(& cmake --version | Select-Object -First 1)",
    "Ninja: $(& ninja --version)", "Flags: $($flags -join ' ')", "Jobs: $Jobs") |
    Set-Content -LiteralPath (Join-Path $licenses 'native-build-provenance.txt') -Encoding utf8NoBOM
Write-Output "Windows CPU runtime installed: $bin"
