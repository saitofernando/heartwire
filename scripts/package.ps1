# Build heartwire.exe and the Windows installer. Windows only.
# Run from any directory: pwsh -NoProfile -File scripts/package.ps1
# Requires NSIS 3 (makensis on PATH or in its default install folder).
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path

cargo build --release --locked -p heartwire --manifest-path (Join-Path $projectRoot 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'cargo build falhou / cargo build failed' }

$manifest = Get-Content -LiteralPath (Join-Path $projectRoot 'Cargo.toml') -Raw
if ($manifest -notmatch '(?m)^version = "([^"]+)"') { throw 'Versão não encontrada / Version not found' }
$version = $Matches[1]

$makensis = (Get-Command makensis -ErrorAction SilentlyContinue).Source
if (-not $makensis) {
    $makensis = @(
        (Join-Path ${env:ProgramFiles(x86)} 'NSIS\makensis.exe'),
        (Join-Path $env:ProgramFiles 'NSIS\makensis.exe')
    ) | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
}
if (-not $makensis) { throw 'NSIS (makensis) não encontrado / NSIS (makensis) not found' }

$outDir = Join-Path $projectRoot 'target\release\bundle\nsis'
New-Item -ItemType Directory -Path $outDir -Force | Out-Null
# The script is UTF-8 without BOM; without this, accented text turns into mojibake.
& $makensis /V2 /INPUTCHARSET UTF8 "/DVERSION=$version" "/DROOT=$projectRoot" "/DOUTDIR=$outDir" (Join-Path $projectRoot 'crates\heartwire\installer\heartwire.nsi')
if ($LASTEXITCODE -ne 0) { throw 'makensis falhou / makensis failed' }
Write-Output "Instalador / Installer: target/release/bundle/nsis/HeartWire_${version}_x64-setup.exe"
