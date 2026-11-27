# Packs target\release\lurt.exe into an MSIX package for the Microsoft Store, which signs
# it itself. Build first (cargo build --release), then:
#
#   powershell -File packaging\msix.ps1 -IdentityName <name> -Publisher <CN=...> -PublisherDisplayName <name>
#
# The three values come from Partner Center > your app > Product identity. Without them the
# package gets placeholders: good for checking that it packs, not for uploading.
param(
  [string]$IdentityName = 'Lurt.Placeholder',
  [string]$Publisher = 'CN=Placeholder',
  [string]$PublisherDisplayName = 'ladroxd',
  # The app name you reserved in Partner Center.
  [string]$DisplayName = 'Lurt'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$exe = Join-Path $root 'target\release\lurt.exe'
if (-not (Test-Path $exe)) { throw "No $exe. Run: cargo build --release" }

$cargo = Get-Content (Join-Path $root 'Cargo.toml') -Raw
if ($cargo -notmatch '(?m)^version = "(\d+)\.(\d+)\.(\d+)"') { throw 'No version in Cargo.toml' }
# The Store refuses versions starting with 0 and keeps the fourth number for itself,
# so 0.8.6 is packaged as 1.8.6.0 (and a later 1.0.0 as 2.0.0.0).
$version = '{0}.{1}.{2}.0' -f ([int]$Matches[1] + 1), $Matches[2], $Matches[3]

# A fresh folder for each run, so nothing stale ends up in the package.
$out = Join-Path $root 'target\msix'
$stage = Join-Path $out ('stage-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force (Join-Path $stage 'images') | Out-Null
Copy-Item $exe $stage

Add-Type -AssemblyName System.Drawing
$logo = [System.Drawing.Image]::FromFile((Join-Path $root 'assets\lurt.png'))
foreach ($l in @(@('StoreLogo', 50), @('Square44x44Logo', 44), @('Square150x150Logo', 150))) {
  $bmp = New-Object System.Drawing.Bitmap $l[1], $l[1]
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.InterpolationMode = 'HighQualityBicubic'
  $g.DrawImage($logo, 0, 0, $l[1], $l[1])
  $bmp.Save((Join-Path $stage "images\$($l[0]).png"), [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
}
$logo.Dispose()

$manifest = Get-Content (Join-Path $PSScriptRoot 'AppxManifest.xml') -Raw -Encoding UTF8
$values = @{ IdentityName = $IdentityName; Publisher = $Publisher; PublisherDisplayName = $PublisherDisplayName; DisplayName = $DisplayName; Version = $version }
foreach ($k in $values.Keys) { $manifest = $manifest.Replace("{$k}", [System.Security.SecurityElement]::Escape($values[$k])) }
[System.IO.File]::WriteAllText((Join-Path $stage 'AppxManifest.xml'), $manifest)

$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\makeappx.exe" |
  Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1
if (-not $makeappx) { throw 'makeappx.exe not found: install the Windows SDK' }
$msix = Join-Path $out "lurt-$version.msix"
& $makeappx.FullName pack /d $stage /p $msix /o
if ($LASTEXITCODE -ne 0) { throw "makeappx failed ($LASTEXITCODE)" }
Write-Host "Packed $msix"
