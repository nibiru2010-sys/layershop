# Package, publish and update: how a build becomes the production copy
#
# Layout this assumes:
#   <repo>                          source checkout (dev copy is app\ + portable.txt)
#   %ProgramFiles%\LayerShop        THE PRODUCTION COPY (no portable.txt: settings in %APPDATA%)
#   github.com/nibiru2010-sys/layershop   the fork the app reads releases from
#
# Build a release      : tools\package.ps1                 (builds, zips into dist\)
# Publish it           : tools\package.ps1 -Publish        (creates the GitHub release)
# Install / update     : tools\update.ps1                  (detects a newer release, installs it)
# Only check           : tools\update.ps1 -Check           (exit 1 when an update is waiting)
#
# The app itself also notices: its update card reads the fork's release list, so a published
# release shows up in Help while this script is what installs it (LayerShop's in-place updater
# is deliberately off).
#
# The install folder and the cargo artifact folder can be overridden in tools\local-layout.ps1
# (gitignored, see local-layout.example.ps1); the defaults are portable.

$Root = Split-Path -Parent $PSScriptRoot
$Repo = 'nibiru2010-sys/layershop'
$Dist = Join-Path $Root 'dist'
$InstallDir = Join-Path $env:ProgramFiles 'LayerShop'
$CargoTarget = Join-Path $Root 'target'
$localLayout = Join-Path $PSScriptRoot 'local-layout.ps1'
if (Test-Path -LiteralPath $localLayout) { . $localLayout }
if ($LayoutInstallDir) { $InstallDir = $LayoutInstallDir }
if ($LayoutCargoTarget) { $CargoTarget = $LayoutCargoTarget }

function Get-WorkspaceVersion {
    $manifest = Get-Content -Raw (Join-Path $Root 'Cargo.toml')
    if ($manifest -notmatch '(?m)^\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"') {
        throw 'Cargo.toml: no [workspace.package] version'
    }
    $Matches[1]
}

function Get-ExeVersion([string] $Exe) {
    if (-not (Test-Path $Exe)) { return $null }
    (Get-Item $Exe).VersionInfo.FileVersion
}

function Compare-Semver([string] $A, [string] $B) {
    # -1, 0 or 1: A before, same as, or after B. `1.2.3`, `1.2`, `v1.2.3`.
    $norm = { param($v) $v.Trim().TrimStart('v', 'V') -replace '-.*$', '' }
    $parse = {
        param($v)
        $parts = (& $norm $v).Split('.')
        @([int]$parts[0], [int]($parts[1]), [int]($parts[2]))
    }
    $pa = & $parse $A
    $pb = & $parse $B
    for ($i = 0; $i -lt 3; $i++) {
        if ($pa[$i] -lt $pb[$i]) { return -1 }
        if ($pa[$i] -gt $pb[$i]) { return 1 }
    }
    0
}
