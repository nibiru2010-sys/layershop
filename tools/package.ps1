# Build the production package, and optionally publish it as a release of the fork.
#
#   tools\package.ps1                 build + zip into dist\LayerShop-<version>-win-x64.zip
#   tools\package.ps1 -Publish        ... and create github.com/nibiru2010-sys/layershop's
#                                     release "layershop-v<version>" with that zip attached
#   tools\package.ps1 -SkipBuild      zip what is already in the cargo target dir
#
# The zip holds what a Program Files install needs: the two executables, the licences and the
# notice. No portable.txt (production keeps its settings in %APPDATA%\LayerShop), and no
# LayerShopData. Bump [workspace.package] version in Cargo.toml before publishing: a release
# whose tag is not newer than the installed build is not offered as an update.

[CmdletBinding()]
param(
    [switch] $Publish,
    [switch] $SkipBuild
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'release-common.ps1')

$version = Get-WorkspaceVersion
$zipName = "LayerShop-$version-win-x64.zip"
$zipPath = Join-Path $Dist $zipName
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (-not $SkipBuild) {
    Write-Host "Building $version (release)..."
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    Push-Location $Root
    try {
        $env:CARGO_TARGET_DIR = $CargoTarget
        # A published binary must not carry build-machine paths. Rust embeds source locations in
        # panic metadata, so remap the user folder, the cargo cache and the checkout to nothing.
        # Derived at build time, never written down anywhere.
        if (-not $env:RUSTFLAGS) {
            $remap = @(
                "--remap-path-prefix=$env:USERPROFILE="
                "--remap-path-prefix=$CargoTarget="
                "--remap-path-prefix=$Root="
            )
            $env:RUSTFLAGS = ($remap -join ' ')
        }
        cargo build --release -p photocraft -p photocraft-cli 2>&1 | ForEach-Object { if ($_ -match '^(error|warning: unused)') { Write-Host $_ } }
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
    } finally {
        Pop-Location
        $ErrorActionPreference = $previous
    }
}

$release = Join-Path $CargoTarget 'release'
$stage = Join-Path $Dist "stage-$version"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force -Path $stage | Out-Null

Copy-Item (Join-Path $release 'photocraft.exe') $stage -Force
Copy-Item (Join-Path $release 'photocraft-cli.exe') $stage -Force
foreach ($file in 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE') {
    $from = Join-Path $Root $file
    if (Test-Path $from) { Copy-Item $from $stage -Force }
}

$built = Get-ExeVersion (Join-Path $stage 'photocraft.exe')
if ($built -ne $version) {
    throw "the executable reports version '$built', but Cargo.toml says '$version' (bump Cargo.toml, or the update check will never offer it)"
}

if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zipPath -CompressionLevel Optimal
Remove-Item -Recurse -Force $stage
Write-Host ("package: {0} ({1:N1} MB)" -f $zipPath, ((Get-Item $zipPath).Length / 1MB))

if (-not $Publish) {
    Write-Host "next: tools\package.ps1 -Publish   or   tools\update.ps1"
    return
}

# The token comes from the Windows user environment; it is never written to disk or printed.
if (-not $env:GH_TOKEN) {
    $env:GH_TOKEN = [Environment]::GetEnvironmentVariable('GIT_TOKEN_CLASSIC', 'User')
}
if (-not $env:GH_TOKEN) { throw 'no GitHub token: set GIT_TOKEN_CLASSIC in the user environment' }

$tag = "layershop-v$version"
$notes = @"
LayerShop fork build $version.

Portable: unzip anywhere and run photocraft.exe.
Production: run tools\update.ps1 from the fork checkout, or unzip over your Program Files
LayerShop folder (after closing the app).

Built from branch fw-layershop-comfy-presets at $(git -C $Root rev-parse --short HEAD).
"@
$notesPath = Join-Path $Dist "notes-$version.md"
[IO.File]::WriteAllText($notesPath, $notes, (New-Object System.Text.UTF8Encoding($false)))

$previous = $ErrorActionPreference
$ErrorActionPreference = 'Continue'   # gh writes its "not found" to stderr, which Stop turns fatal
$existing = gh release view $tag --repo $Repo --json tagName 2>$null
$exists = $LASTEXITCODE -eq 0
$ErrorActionPreference = $previous
$previous = $ErrorActionPreference
$ErrorActionPreference = 'Continue'   # gh writes progress to stderr
if ($exists) {
    Write-Host "release $tag exists: uploading the asset over it"
    gh release upload $tag $zipPath --repo $Repo --clobber
} else {
    gh release create $tag $zipPath --repo $Repo --title "LayerShop $version (fork build)" --notes-file $notesPath
}
$published = $LASTEXITCODE
$ErrorActionPreference = $previous
if ($published -ne 0) { throw "publishing $tag failed with exit code $published" }
Write-Host "published: https://github.com/$Repo/releases/tag/$tag"
Write-Host 'install it: tools\update.ps1'
