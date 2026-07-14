param(
    [switch]$SkipClippy
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptRoot "..")

function Invoke-ProjectStep {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "== $Name =="
    $global:LASTEXITCODE = 0
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

Push-Location $repoRoot
try {
    Invoke-ProjectStep "Format check" { cargo fmt --check }

    if (-not $SkipClippy) {
        Invoke-ProjectStep "Clippy (all targets)" { cargo clippy --all-targets -- -D warnings }
    }

    Invoke-ProjectStep "Tests" { cargo test }
    Invoke-ProjectStep "Project doctor" { cargo run -- doctor }

    Write-Host ""
    Write-Host "Project check passed."
}
finally {
    Pop-Location
}
