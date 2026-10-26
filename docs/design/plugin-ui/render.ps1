param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$node=(Get-Command node -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
$magick=(Get-Command magick -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
& $node (Join-Path $PSScriptRoot 'generate.mjs')
if($LASTEXITCODE -ne 0){throw 'Fixture assertions failed'}
$dimensions=Get-Content (Join-Path $PSScriptRoot 'dimensions.json') -Raw | ConvertFrom-Json
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$results=@(foreach($fixture in $dimensions.psobject.Properties){
    foreach($scale in @(100,150,200)){
        $width=[int]($fixture.Value.width*$scale/100)
        $height=[int]($fixture.Value.height*$scale/100)
        $path=Join-Path $OutputDirectory "$($fixture.Name)-$scale.png"
        & $magick -background '#202226' -density (96*$scale/100) (Join-Path $PSScriptRoot "$($fixture.Name).svg") -resize "${width}x${height}!" -set comment "Original static CleanSlate design fixture; source generate.mjs; synthetic signal; scale $scale percent" $path
        if($LASTEXITCODE -ne 0){throw "Render failed: $path"}
        $size=& $magick identify -format '%w %h' $path
        if($size -ne "$width $height"){throw "Wrong output dimensions: $size"}
        [pscustomobject]@{file=[IO.Path]::GetFileName($path);width=$width;height=$height;sha256=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant()}
    }
})
if($results.Count -ne 12){throw 'Incomplete DPI matrix'}
$results | ConvertTo-Json | Set-Content (Join-Path $OutputDirectory 'manifest.json')
Write-Output 'PASS: twelve deterministic compact/full DPI renders, dimensions and hashes'
