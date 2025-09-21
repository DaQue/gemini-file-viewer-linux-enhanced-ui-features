Param(
  [string]$Configuration = "Release"
)

# Build GFV on Windows and zip the executable for distribution.
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent
Set-Location $repoRoot

Write-Host "Refreshing icon assets from icon_pack.zip..."
$iconPackZip = Join-Path $repoRoot 'icon_pack.zip'
if (-not (Test-Path $iconPackZip)) { throw "Missing $iconPackZip" }

$outDir = Join-Path $repoRoot 'assets\icons'
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

try {
  Expand-Archive -Path $iconPackZip -DestinationPath $tempDir -Force

  Add-Type -AssemblyName System.Drawing
  $pngFiles = Get-ChildItem -Path $tempDir -Filter 'icon_*.png'
  $available = @{}
  foreach ($file in $pngFiles) {
    if ($file.BaseName -match 'icon_(\d+)') {
      $size = [int]$matches[1]
      $available[$size] = $file.FullName
    }
  }

  if ($available.Count -eq 0) { throw "icon_pack.zip did not contain icon_*.png files" }

  $largestSize = ($available.Keys | Measure-Object -Maximum).Maximum
  $largestPath = $available[$largestSize]
  $baseBitmap = [System.Drawing.Image]::FromFile($largestPath)

  try {
    foreach ($size in 16,24,32,48,64,96,128,256,384,512) {
      $dest = Join-Path $outDir ("icon_{0}.png" -f $size)
      if ($available.ContainsKey($size)) {
        Copy-Item -Force -Path $available[$size] -Destination $dest
      } else {
        $scaled = New-Object System.Drawing.Bitmap $size, $size
        $graphics = [System.Drawing.Graphics]::FromImage($scaled)
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $graphics.DrawImage($baseBitmap, 0, 0, $size, $size)
        $graphics.Dispose()
        $scaled.Save($dest, [System.Drawing.Imaging.ImageFormat]::Png)
        $scaled.Dispose()
      }
      Write-Host "  icon_$size.png -> $dest"
    }
  } finally {
    $baseBitmap.Dispose()
  }

  $icoSource = Join-Path $tempDir 'icon.ico'
  if (Test-Path $icoSource) {
    Copy-Item -Force -Path $icoSource -Destination (Join-Path $outDir 'icon.ico')
  } else {
    Write-Warning "icon_pack.zip missing icon.ico"
  }
} finally {
  if (Test-Path $tempDir) {
    Remove-Item -Recurse -Force $tempDir
  }
}

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
