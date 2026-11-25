param([ValidateRange(1, 65535)][int]$Port = 8080)

$ErrorActionPreference = 'Stop'
if (-not $env:AETHER_TOKEN -or $env:AETHER_TOKEN.Length -lt 32) {
    throw 'Set a local AETHER_TOKEN of at least 32 characters before starting.'
}
$root = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath (Join-Path $root 'web')
try {
    npm ci --no-audit --no-fund
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    npm run build
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}
Push-Location -LiteralPath $root
try {
    cargo run --locked --release --bin aether-lab -- serve $Port
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
