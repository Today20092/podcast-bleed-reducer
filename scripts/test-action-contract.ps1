$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$source = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../src/button_action.rs'))
$fixtureRoot = Join-Path ([IO.Path]::GetTempPath()) ("cleanslate-action-contract-" + [Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null
$fixture = Join-Path $fixtureRoot 'contract.rs'
$metadata = Join-Path $fixtureRoot 'contract.rmeta'
$diagnostics = Join-Path $fixtureRoot 'diagnostics.txt'
try {
    $example = @"
mod contract { include!(r#"$source"#); }
struct Message;
struct NewAction;
fn activate(action: impl contract::ButtonAction<Message>) -> Message { contract::message(action) }
fn omitted_registration() { activate(NewAction); }
"@
    [IO.File]::WriteAllText($fixture, $example)
    & rustc --edition 2024 --crate-name action_contract --crate-type lib --emit metadata $fixture -o $metadata 2> $diagnostics
    if ($LASTEXITCODE -eq 0) { throw 'Unregistered action unexpectedly passed the button activation contract.' }
    $errorText = [IO.File]::ReadAllText($diagnostics)
    if ($errorText -notmatch 'E0277' -or $errorText -notmatch 'ButtonAction') {
        throw "The omitted registration failed for an unrelated reason: $errorText"
    }
    $registration = @'
impl contract::ButtonAction<Message> for NewAction {
    fn into_message(self) -> Message { Message }
}
'@
    [IO.File]::WriteAllText($fixture, $example + [Environment]::NewLine + $registration)
    & rustc --edition 2024 --crate-name action_contract --crate-type lib --emit metadata -A dead_code $fixture -o $metadata 2> $diagnostics
    if ($LASTEXITCODE -ne 0) { throw ([IO.File]::ReadAllText($diagnostics)) }
    # Supplement the typed boundary by rejecting direct native-button construction.
    $appSource = [IO.File]::ReadAllText((Join-Path $PSScriptRoot '../src/app.rs'))
    if ($appSource -match 'iced::widget::button\s*\(|\bButton::new\s*\(') {
        throw 'Application buttons must use gui_controls::button and its typed action contract.'
    }
    Write-Output 'Action contract passed: omitted registration rejected (E0277); explicit registration accepted.'
} finally {
    foreach ($artifact in @($fixture, $metadata, $diagnostics)) {
        if (Test-Path -LiteralPath $artifact) { Remove-Item -LiteralPath $artifact -Force }
    }
    Remove-Item -LiteralPath $fixtureRoot -Force
}
