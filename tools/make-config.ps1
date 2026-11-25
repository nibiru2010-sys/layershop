# Builds THIS fork's settings folder: points it at a local ComfyUI server and installs the
# LayerShop-only custom-workflow presets from a folder of API graphs.
# Run it after tools\build.ps1 when the graphs change; it keeps any built-in model entries the
# app added (only presets whose id starts with `ls-` are replaced).
#
# Defaults: the app's portable data folder, the ComfyUI server on http://127.0.0.1:8188 and the
# graphs in <project>\workflows\api-wks\layershop. Override any of them through
# tools\local-layout.ps1 (gitignored, see local-layout.example.ps1) or the parameters below.
param(
  [string]$ConfigDir,
  [string]$SourcePreferences = "$env:APPDATA\LayerShop\preferences.json",
  [string]$GraphDir,
  [string]$ComfyUrl
)
$ErrorActionPreference = 'Stop'
$proj = Split-Path -Parent $PSScriptRoot
$localLayout = Join-Path $PSScriptRoot 'local-layout.ps1'
if (Test-Path -LiteralPath $localLayout) { . $localLayout }
if ($LayoutProjectRoot) { $proj = $LayoutProjectRoot }
if (-not $ConfigDir) { $ConfigDir = Join-Path $proj 'app\LayerShopData' }
if (-not $GraphDir) { $GraphDir = if ($LayoutGraphDir) { $LayoutGraphDir } else { Join-Path $proj 'workflows\api-wks\layershop' } }
if (-not $ComfyUrl) { $ComfyUrl = if ($LayoutComfyUrl) { $LayoutComfyUrl } else { 'http://127.0.0.1:8188' } }
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null

$create = Get-Content -Raw (Join-Path $GraphDir 'layershop_qwen21_create.api.json')
$edit   = Get-Content -Raw (Join-Path $GraphDir 'layershop_qwen21_edit.api.json')
foreach ($pair in @(@('create', $create), @('edit', $edit))) {
  if (-not $pair[1].Contains('%prompt%')) { throw "$($pair[0]) graph has no %prompt% placeholder" }
  $pair[1] | ConvertFrom-Json | Out-Null
}

function New-CustomModel([string]$Id, [string]$Name, [string]$Workflow) {
  [pscustomobject][ordered]@{
    alpha = $false; apiModel = ''; cfg = 1.0; checkpoint = ''; clip = ''; clip2 = ''
    clip3 = ''; clip4 = ''; controlnet = ''; enabled = $true; enhancer = ''
    family = 'custom'; guidance = 0.0; id = $Id; imageSize = '1K'; inpaint = $null
    lora = ''; loraStrength = 1.0; loras = @(); megapixels = 1.0; name = $Name
    negativePrompt = ''; provider = 'comfy'; quality = 'auto'; refineStrength = 0.35
    refiner = ''; refinerClip = ''; refinerVae = ''; sampler = 'euler'; scheduler = 'simple'
    seed = -1; shift = 3.0; steps = 25; unet = ''; unet2 = ''; vae = ''; workflow = $Workflow
  }
}

# Start from this copy's own settings when it has them, else from the installed app's.
$live = Join-Path $ConfigDir 'preferences.json'
$prefs = if (Test-Path $live) { Get-Content -Raw $live | ConvertFrom-Json } else { Get-Content -Raw $SourcePreferences | ConvertFrom-Json }
$ai = $prefs.integrations.ai
$ai.comfyUrl = $ComfyUrl
$ai.comfyAutostart = $false
$kept = @($ai.models | Where-Object { $_.id -notlike 'ls-*' })
$ai.models = @($kept) + @(
  (New-CustomModel 'ls-qwen21-create' 'Qwen Image 2.1 - Create (ours)' $create),
  (New-CustomModel 'ls-qwen21-edit'   'Qwen Image 2.1 - Restyle (ours)' $edit)
)
$prefs.integrations.ai = $ai

[IO.File]::WriteAllText($live, ($prefs | ConvertTo-Json -Depth 40), (New-Object System.Text.UTF8Encoding($false)))

$check = Get-Content -Raw $live | ConvertFrom-Json
"config: $live"
"comfyUrl: $($check.integrations.ai.comfyUrl)  autostart: $($check.integrations.ai.comfyAutostart)"
foreach ($m in $check.integrations.ai.models) {
  "model: $($m.name) | id=$($m.id) family=$($m.family) prompt=$($m.workflow.Contains('%prompt%')) image=$($m.workflow.Contains('%image%'))"
}
