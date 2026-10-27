[CmdletBinding()]
param([Parameter(Mandatory)][string]$LauncherExe)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$version = (Get-Content launcher/Cargo.toml -Raw | Select-String '(?m)^version = "([^"]+)"').Matches[0].Groups[1].Value
if (-not $version) { throw 'Versi launcher tidak dapat dibaca.' }
$stage = 'build/package'
if (Test-Path $stage) { throw 'Folder build/package sudah ada. Gunakan clean workspace untuk packaging.' }
New-Item -ItemType Directory -Force "$stage/runtime", "$stage/licenses", 'dist' | Out-Null
Copy-Item -LiteralPath $LauncherExe -Destination "$stage/warriors-launcher.exe"
$runtime = 'build/windows-game/bin/Release'
Copy-Item "$runtime/TheWarriors.exe" "$stage/runtime/"
Copy-Item "$runtime/*.dll" "$stage/runtime/"
Copy-Item 'docs/RELEASE_NOTES.md' "$stage/README.md"
Copy-Item 'launcher/assets/OFL.txt' "$stage/licenses/Anton-OFL.txt"
Copy-Item 'tools/PS2Recomp/LICENSE' "$stage/licenses/PS2Recomp-LICENSE.txt"
Copy-Item 'build/windows-game/_deps/raylib-src/LICENSE' "$stage/licenses/raylib-LICENSE.txt"
Copy-Item 'licenses/FFmpeg-LGPL-2.1.txt' "$stage/licenses/FFmpeg-LGPL-2.1.txt"
$vendorLicenses = 'build/windows-game/ThirdParty/ffmpeg-prefix/src/ffmpeg_external/licenses'
if (Test-Path $vendorLicenses) { Copy-Item $vendorLicenses "$stage/licenses/ffmpeg-vendor" -Recurse }

$files = Get-ChildItem $stage -File -Recurse
if ($files | Where-Object { $_.Name -match '(?i)\.iso$|\.wad$|\.snd$|\.bik$|^SLUS_|^SYSTEM\.CNF$|\.cpp$|\.csv$' }) {
    throw 'Paket berisi input build atau aset game yang tidak boleh ikut rilis.'
}
$zip = "dist/The-Warriors-$version-Windows-x64-experimental.zip"
Compress-Archive -Path "$stage/*" -DestinationPath $zip
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content "$zip.sha256" "$hash  $(Split-Path $zip -Leaf)" -Encoding ascii
Write-Output "Paket siap: $zip"
