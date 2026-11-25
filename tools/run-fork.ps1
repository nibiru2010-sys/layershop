# Starts the LayerShop fork from this project's own install.
#
#   app\photocraft.exe                 <- the executable (build.ps1 puts it there)
#   app\LayerShopData\                 <- that copy's preferences, logs, recovery
#   work\                              <- the only folder the control channel may read/write
#
# The MSI install under Program Files and its %APPDATA%\LayerShop settings are untouched, so
# both copies can run side by side.
#
# Usage:
#   pwsh -File tools\run-fork.ps1                 # normal start
#   pwsh -File tools\run-fork.ps1 -Control 7879   # start with the agent control channel
#   pwsh -File tools\run-fork.ps1 -Arguments @('some.psd')
param(
  [string[]]$Arguments = @(),
  [int]$Control = 0
)
$ErrorActionPreference = 'Stop'
$proj = Split-Path -Parent $PSScriptRoot
$localLayout = Join-Path $PSScriptRoot 'local-layout.ps1'
if (Test-Path -LiteralPath $localLayout) { . $localLayout }
if ($LayoutProjectRoot) { $proj = $LayoutProjectRoot }
$exe = Join-Path $proj 'app\photocraft.exe'
if (-not (Test-Path -LiteralPath $exe)) { throw "not built yet: $exe (run tools\build.ps1)" }
$data = Join-Path $proj 'app\LayerShopData'
$work = Join-Path $proj 'work'
New-Item -ItemType Directory -Force -Path $data, $work | Out-Null
$env:PHOTOCRAFT_CONFIG_DIR = $data

$list = @()
if ($Control -gt 0) {
  $list += @('--control', "$Control", '--control-token-file', (Join-Path $data 'agent-control.token'))
  $list += @('--automation-read-root', $work, '--automation-write-root', $work)
}
$list += $Arguments
Start-Process -FilePath $exe -ArgumentList $list
"started $exe"
