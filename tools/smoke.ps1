# End-to-end check of THIS fork against the local ComfyUI server, through the running app.
# Start the app first:  pwsh -File tools\run-fork.ps1 -Control 7879
# Then:                 pwsh -File tools\smoke.ps1
#
# It creates a document, renders with the Create preset, renders the result with the Restyle
# preset, checks that each render landed on a new layer, and saves the images the server
# returned into work\smoke\. Layer placement is asserted from the app's own document state.
param(
  [int]$Port = 7879,
  [string]$Server = 'http://127.0.0.1:8188',
  [string]$Prompt = 'A red ceramic mug on a wooden table, soft daylight, photorealistic',
  [string]$EditPrompt = 'Change the mug from red to cobalt blue. Keep the table, the light and the composition.',
  [int]$Width = 768,
  [int]$Height = 512
)
$ErrorActionPreference = 'Stop'
$proj = Split-Path -Parent $PSScriptRoot
$localLayout = Join-Path $PSScriptRoot 'local-layout.ps1'
if (Test-Path -LiteralPath $localLayout) { . $localLayout }
if ($LayoutProjectRoot) { $proj = $LayoutProjectRoot }
$ctl = Join-Path $proj 'tools\ls-ctl.ps1'
$out = Join-Path $proj 'work\smoke'
New-Item -ItemType Directory -Force -Path $out | Out-Null

function Cmd([string]$id, [hashtable]$params) {
  $json = @{ command = $id; params = $params } | ConvertTo-Json -Depth 12 -Compress
  $reply = & $ctl -Port $Port -Method engine.execute -Params $json -TimeoutSec 900 | ConvertFrom-Json
  if (-not $reply.ok) { throw "$id failed: $($reply.error)" }
  return $reply.result
}

function Save-LastRender([string]$name) {
  $h = (Invoke-WebRequest "$Server/history?max_items=1" -UseBasicParsing -TimeoutSec 20).Content | ConvertFrom-Json
  $entry = $h.PSObject.Properties | Select-Object -First 1
  $img = @($entry.Value.outputs.PSObject.Properties.Value.images) | Where-Object { $_.type } | Select-Object -First 1
  if (-not $img) { throw "no image in the server's last history entry" }
  $url = "$Server/view?filename=$([uri]::EscapeDataString($img.filename))&type=$($img.type)"
  $path = Join-Path $out $name
  Invoke-WebRequest $url -OutFile $path -UseBasicParsing
  return $path
}

"document: " + ((Cmd 'file.new' @{ width = $Width; height = $Height; resolution = 72 }) | ConvertTo-Json -Compress)
$before = (Cmd 'document.inspect' @{}).layers.Count

$made = Cmd 'ai.generate' @{ prompt = $Prompt; model = 'ls-qwen21-create'; seed = 4242 }
if (-not $made.layer) { throw 'create placed no layer' }
"create:   layer $($made.layer), mode $($made.mode), $($made.renderSize -join 'x')"
$createPng = Save-LastRender 'create.png'

$restyle = Cmd 'ai.generate' @{ prompt = $EditPrompt; model = 'ls-qwen21-edit'; mode = 'img2img'; seed = 777 }
if (-not $restyle.layer -or $restyle.layer -eq $made.layer) { throw 'restyle placed no new layer' }
"restyle:  layer $($restyle.layer), mode $($restyle.mode), $($restyle.renderSize -join 'x')"
$restylePng = Save-LastRender 'restyle.png'

$after = (Cmd 'document.inspect' @{}).layers.Count
if ($after -le $before) { throw "no layers were added ($before -> $after)" }
"layers:   $before -> $after"
"files:    $createPng"
"          $restylePng"
