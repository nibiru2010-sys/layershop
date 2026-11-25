# Builds the LayerShop fork and installs the result as the app you run.
#
#   pwsh -File build.ps1              # release build + install into .\app
#   pwsh -File build.ps1 -Debug       # debug build + install (faster loop, slower app)
#
# Output (the one place the executable lives): <project>\app\photocraft.exe and
# <project>\app\photocraft-cli.exe. Settings used by that copy: .\app\LayerShopData
# (portable mode, see portable.txt).
#
# The project root is the folder above tools\, and cargo's artifacts land in <project>\target.
# To keep the artifacts elsewhere (or to work from another checkout) put the overrides in
# tools\local-layout.ps1 — gitignored, see local-layout.example.ps1.
param([switch]$Debug)
$ErrorActionPreference = 'Stop'
$proj = Split-Path -Parent $PSScriptRoot
$localLayout = Join-Path $PSScriptRoot 'local-layout.ps1'
if (Test-Path -LiteralPath $localLayout) { . $localLayout }
if ($LayoutProjectRoot) { $proj = $LayoutProjectRoot }
$app = Join-Path $proj 'app'
$env:CARGO_TARGET_DIR = if ($LayoutCargoTarget) { $LayoutCargoTarget } else { Join-Path $proj 'target' }
$env:CARGO_TERM_COLOR = 'never'
# Build paths never reach a published binary: Rust writes source locations into panic metadata,
# which would otherwise carry this machine's user folder and cargo cache into every release.
# The prefixes are derived at build time (never written down), so a build from any machine is
# clean. Set them only when unset, so an explicit RUSTFLAGS in the environment still wins.
if (-not $env:RUSTFLAGS) {
  $remap = @(
    "--remap-path-prefix=$env:USERPROFILE="
    "--remap-path-prefix=$($env:CARGO_TARGET_DIR)="
    "--remap-path-prefix=$proj="
  )
  $env:RUSTFLAGS = ($remap -join ' ')
}
New-Item -ItemType Directory -Force -Path $app | Out-Null

$profileArgs = if ($Debug) { @() } else { @('--release') }
Push-Location $proj
# Cargo writes warnings to stderr; with $ErrorActionPreference = 'Stop' Windows PowerShell turns
# that into a terminating NativeCommandError, so the native call runs with 'Continue' and its exit
# code decides.
$previous = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
  if ($Debug) {
    & cargo build -p photocraft -p photocraft-cli
  } else {
    & cargo build --release -p photocraft -p photocraft-cli
  }
  $code = $LASTEXITCODE
} finally {
  $ErrorActionPreference = $previous
  Pop-Location
}
if ($code -ne 0) { throw "cargo build failed: $code" }

$rel = if ($Debug) { Join-Path $env:CARGO_TARGET_DIR 'debug' } else { Join-Path $env:CARGO_TARGET_DIR 'release' }
foreach ($f in 'photocraft.exe', 'photocraft-cli.exe') {
  Copy-Item (Join-Path $rel $f) (Join-Path $app $f) -Force
}
if (-not (Test-Path (Join-Path $app 'portable.txt'))) {
  Copy-Item (Join-Path $proj 'packaging\windows\portable.txt') (Join-Path $app 'portable.txt')
}
$exe = Join-Path $app 'photocraft.exe'
"built: $(& (Join-Path $app 'photocraft-cli.exe') --version)"
"run:   $exe   (run-fork.ps1 starts it with the control channel)"
