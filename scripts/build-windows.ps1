Param(
  [string]$Configuration = "Release"
)

# Build GFV on Windows and zip the executable for distribution.
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent
Set-Location $repoRoot

Write-Host "Building ($Configuration)..."
cargo build --release

$exe = Join-Path $repoRoot "target\release\gfv.exe"
if (-not (Test-Path $exe)) { throw "Missing $exe" }

$ver = Select-String -Path (Join-Path $repoRoot 'Cargo.toml') -Pattern '^version\s*=\s*"(.*)"' | Select-Object -First 1 | ForEach-Object { $_.Matches[0].Groups[1].Value }
$outDir = Join-Path $repoRoot 'releases'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$zip = Join-Path $outDir ("gfv-$ver-windows-x86_64.zip")

Write-Host "Packaging $exe -> $zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path $exe -DestinationPath $zip

Write-Host "Computing SHA256..."
$hash = (Get-FileHash -Algorithm SHA256 $zip).Hash
Set-Content -Path (Join-Path $outDir ("SHA256SUMS-$ver-windows.txt")) -Value ("$hash  $(Split-Path -Leaf $zip)")

Write-Host "Done: $zip"

