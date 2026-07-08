param(
    [switch]$SkipClippy
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptRoot "..")

function Invoke-FoundationStep {
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
    Invoke-FoundationStep "Format check" { cargo fmt --check }

    if (-not $SkipClippy) {
        Invoke-FoundationStep "Clippy" { cargo clippy -- -D warnings }
    }

    Invoke-FoundationStep "Tests" { cargo test }
    Invoke-FoundationStep "Content validation" { cargo run -- validate }

    Write-Host ""
    Write-Host "Foundation check passed."
}
finally {
    Pop-Location
}
