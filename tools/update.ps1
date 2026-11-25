# Install or update the production copy from the fork's releases.
#
#   tools\update.ps1                 detect a newer release and install it, then start the app
#   tools\update.ps1 -Check          report only; exit 1 when an update is waiting, 0 when not
#   tools\update.ps1 -InstallDir ... another install folder (default %ProgramFiles%\LayerShop)
#   tools\update.ps1 -Prerelease     also install releases tagged as pre-releases
#
# Detection is the same public release list the app's own update card reads
# (github.com/nibiru2010-sys/layershop). Installation is a copy: close the app, keep the previous
# build as a backup folder, unpack the release zip over the install folder, start it again.
# Settings are untouched: production runs without portable.txt, so they live in %APPDATA%\LayerShop.

[CmdletBinding()]
param(
    [switch] $Check,
    [string] $InstallDir,
    [switch] $Prerelease,
    [switch] $NoStart
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'release-common.ps1')
# release-common.ps1 resolves the install folder (default %ProgramFiles%\LayerShop, overridable
# through tools\local-layout.ps1); an explicit -InstallDir still wins.
if (-not $InstallDir) { $InstallDir = $script:InstallDir }

$installedExe = Join-Path $InstallDir 'photocraft.exe'
$installed = Get-ExeVersion $installedExe

$releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases?per_page=30" -Headers @{ 'User-Agent' = 'LayerShop-Update' } -TimeoutSec 30
if (-not $Prerelease) { $releases = @($releases | Where-Object { -not $_.prerelease } | Where-Object { -not $_.draft }) }
$candidates = @()
foreach ($release in $releases) {
    $tagVersion = $release.tag_name -replace '^layershop-', '' -replace '^[vV]', ''
    if ($tagVersion -notmatch '^\d+\.\d+(\.\d+)?') { continue }
    $asset = @($release.assets | Where-Object { $_.name -like 'LayerShop-*-win-x64.zip' }) | Select-Object -First 1
    if (-not $asset) { continue }
    $candidates += [pscustomobject]@{ Version = $tagVersion; Tag = $release.tag_name; Url = $asset.browser_download_url; Size = $asset.size }
}
if (-not $candidates) { Write-Host 'no packaged releases on the fork yet'; exit 0 }

$newest = $candidates | Sort-Object -Property @{ Expression = { $v = $_.Version.Split('.'); [int]$v[0] * 1000000 + [int]$v[1] * 1000 + $(if ($v.Count -gt 2) { [int]$v[2] } else { 0 }) } } | Select-Object -Last 1

if (-not $installed) {
    Write-Host "no production copy at $InstallDir (nothing installed yet)"
} else {
    Write-Host "installed: $installed    newest release: $($newest.Version) ($($newest.Tag))"
}
$available = (-not $installed) -or ((Compare-Semver $newest.Version $installed) -gt 0)
if (-not $available) { Write-Host 'up to date'; exit 0 }
Write-Host "update available: $installed -> $($newest.Version)"
if ($Check) { exit 1 }

$work = Join-Path ([IO.Path]::GetTempPath()) "layershop-update-$($newest.Version)"
if (Test-Path $work) { Remove-Item -Recurse -Force $work }
New-Item -ItemType Directory -Force -Path $work | Out-Null
$zip = Join-Path $work 'release.zip'
Write-Host "downloading $($newest.Url)"
Invoke-WebRequest -Uri $newest.Url -OutFile $zip -UseBasicParsing -TimeoutSec 600
$staged = Join-Path $work 'staged'
Expand-Archive -LiteralPath $zip -DestinationPath $staged -Force

$stagedExe = Join-Path $staged 'photocraft.exe'
$stagedVersion = Get-ExeVersion $stagedExe
if (-not $stagedVersion) { throw "the release zip has no photocraft.exe" }
if ((Compare-Semver $stagedVersion $newest.Version) -ne 0) {
    throw "the release zip's executable says $stagedVersion, but the release is $($newest.Version)"
}

# Close the running app, so its files can be replaced. Only this install is closed: the dev copy
# (another folder) and the MSI's copy keep running. A document with unsaved changes makes the app
# ask, which needs the person at the machine: say so instead of killing it.
function Get-InstalledApp {
    @(Get-Process photocraft -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $installedExe })
}
if ((Get-InstalledApp).Count -gt 0) {
    Write-Host 'closing the running LayerShop...'
    Get-InstalledApp | ForEach-Object { $_.CloseMainWindow() | Out-Null }
    $deadline = (Get-Date).AddSeconds(20)
    while ((Get-InstalledApp).Count -gt 0 -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 400 }
    if ((Get-InstalledApp).Count -gt 0) {
        throw 'LayerShop is still running (unsaved changes?): save or discard them, then run this again'
    }
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
if (Test-Path $installedExe) {
    $backup = Join-Path $InstallDir ('backup-' + $(if ($installed) { $installed } else { 'old' }))
    if (Test-Path $backup) { Remove-Item -Recurse -Force $backup }
    New-Item -ItemType Directory -Force -Path $backup | Out-Null
    foreach ($file in 'photocraft.exe', 'photocraft-cli.exe') {
        $from = Join-Path $InstallDir $file
        if (Test-Path $from) { Copy-Item $from $backup -Force }
    }
    Write-Host "previous build kept in $backup"
}

# Copy over the install, leaving anything else in the folder (settings are not here anyway).
Get-ChildItem -LiteralPath $staged | ForEach-Object { Copy-Item $_.FullName (Join-Path $InstallDir $_.Name) -Force }
Remove-Item -Recurse -Force $work

$now = Get-ExeVersion $installedExe
Write-Host "installed $now into $InstallDir"
if (-not $NoStart) {
    Start-Process -FilePath $installedExe -WorkingDirectory $InstallDir
    Write-Host 'started'
}
