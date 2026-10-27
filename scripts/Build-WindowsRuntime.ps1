[CmdletBinding()]
param([string]$LocalElf)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
if (-not $IsWindows) { throw 'Jalankan build ini di Windows dengan PowerShell 7.' }
Set-Location (Split-Path $PSScriptRoot -Parent)
$root = (Get-Location).Path
$game = Get-Content config/game.json -Raw | ConvertFrom-Json

function Invoke-Checked([string]$File, [string[]]$Arguments) {
    & $File @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$File gagal dengan exit code $LASTEXITCODE" }
}

$private = Join-Path $root ("build/private/" + [Guid]::NewGuid().ToString('N'))
$elf = Join-Path $private $game.serial
New-Item -ItemType Directory -Force $private | Out-Null
try {
    if ($LocalElf) {
        Copy-Item -LiteralPath $LocalElf -Destination $elf
    } else {
        if (-not $env:PRIVATE_GAME_ELF_URL) { throw 'Secret PRIVATE_GAME_ELF_URL belum dikonfigurasi.' }
        $uri = [Uri]$env:PRIVATE_GAME_ELF_URL
        if ($uri.Scheme -ne 'https') { throw 'Input ELF privat harus melalui HTTPS.' }
        try { Invoke-WebRequest -Uri $uri -OutFile $elf } catch { throw 'Pengunduhan input ELF privat gagal. Periksa secret atau masa berlaku URL.' }
    }
    if ((Get-FileHash $elf -Algorithm SHA256).Hash.ToLowerInvariant() -ne $game.elf_sha256) {
        throw 'Hash ELF privat tidak cocok dengan versi game yang didukung.'
    }
    if (-not (Test-Path tools/PS2Recomp/.git)) {
        Invoke-Checked git @('clone', '--no-checkout', 'https://github.com/ran-j/PS2Recomp.git', 'tools/PS2Recomp')
        Invoke-Checked git @('-C', 'tools/PS2Recomp', 'checkout', $game.ps2recomp_revision)
    }
    $revision = (& git -C tools/PS2Recomp rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $revision -ne $game.ps2recomp_revision) { throw 'Revision PS2Recomp tidak cocok.' }
    Invoke-Checked cmake @('-S', 'tools/PS2Recomp', '-B', 'build/recompiler', '-G', 'Visual Studio 17 2022', '-A', 'x64', '-DPS2X_BUILD_RUNTIME=OFF', '-DPS2X_BUILD_ANALYZER=OFF', '-DPS2X_BUILD_TEST=OFF', '-DPS2X_BUILD_STUDIO=OFF')
    Invoke-Checked cmake @('--build', 'build/recompiler', '--config', 'Release', '--target', 'ps2_recomp', '--parallel', '2')
    $generated = Join-Path $root 'build/generated'
    $map = Join-Path $root 'config/functions.csv'
    $config = (Get-Content config/warriors.toml.in -Raw).Replace('@ELF@', $elf.Replace('\', '/')).Replace('@OUTPUT@', $generated.Replace('\', '/')).Replace('@MAP@', $map.Replace('\', '/'))
    $configPath = Join-Path $private 'warriors.toml'
    Set-Content $configPath $config -Encoding utf8NoBOM
    Invoke-Checked 'build/recompiler/ps2xRecomp/Release/ps2_recomp.exe' @($configPath)
    Invoke-Checked cmake @('-S', 'runtime', '-B', 'build/windows-game', '-G', 'Visual Studio 17 2022', '-A', 'x64', "-DWARRIORS_GENERATED=$generated")
    Invoke-Checked cmake @('--build', 'build/windows-game', '--config', 'Release', '--target', 'ps2EntryRunner', '--parallel', '2')
} finally {
    if (Test-Path $private) { Remove-Item -LiteralPath $private -Force -Recurse }
}
