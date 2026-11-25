# Minimal client for LayerShop's control channel (TCP, JSON lines, token auth).
# Start the app with `tools\run-fork.ps1 -Control 7879` first.
#
#   pwsh -File tools\ls-ctl.ps1 -Method engine.commands
#   pwsh -File tools\ls-ctl.ps1 -Method engine.execute -Params '{"command":"file.new","params":{"width":512,"height":384}}'
param(
  [int]$Port = 7879,
  [string]$TokenFile,
  [Parameter(Mandatory = $true)][string]$Method,
  [string]$Params = '{}',
  [int]$TimeoutSec = 300
)
$ErrorActionPreference = 'Stop'
$proj = Split-Path -Parent $PSScriptRoot
$localLayout = Join-Path $PSScriptRoot 'local-layout.ps1'
if (Test-Path -LiteralPath $localLayout) { . $localLayout }
if ($LayoutProjectRoot) { $proj = $LayoutProjectRoot }
# The control token the app writes into its portable data folder (run-fork.ps1 -Control).
if (-not $TokenFile) { $TokenFile = Join-Path $proj 'app\LayerShopData\agent-control.token' }
$token = (Get-Content -Raw $TokenFile).Trim()
if ($token -notmatch '^[0-9a-f]{64}$') { throw "not a 64-hex token: $TokenFile" }

$client = New-Object System.Net.Sockets.TcpClient
$client.Connect('127.0.0.1', $Port)
$stream = $client.GetStream()
$stream.ReadTimeout = $TimeoutSec * 1000
$writer = New-Object System.IO.StreamWriter($stream, (New-Object System.Text.UTF8Encoding($false)))
$writer.NewLine = "`n"; $writer.AutoFlush = $true
$reader = New-Object System.IO.StreamReader($stream, (New-Object System.Text.UTF8Encoding($false)))

function Send-Json($obj) { $writer.WriteLine(($obj | ConvertTo-Json -Depth 20 -Compress)) }
function Read-Json {
  $line = $reader.ReadLine()
  if ($null -eq $line) { throw 'connection closed before a reply' }
  return ($line | ConvertFrom-Json)
}

Send-Json @{ id = 'auth'; method = 'auth'; params = @{ token = $token } }
$auth = Read-Json
if (-not $auth.ok) { throw "auth failed: $($auth.error)" }

Send-Json @{ id = 1; method = $Method; params = ($Params | ConvertFrom-Json) }
$reply = Read-Json
$client.Close()
$reply | ConvertTo-Json -Depth 20
