[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$generator = Join-Path $PSScriptRoot "generate_prototype_textures.py"
$launchers = @()

$bundledPython = Join-Path $env:USERPROFILE ".cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe"
if (Test-Path -LiteralPath $bundledPython -PathType Leaf) {
    $launchers += [pscustomobject]@{ Path = $bundledPython; Prefix = @() }
}

foreach ($name in @("python3", "python")) {
    $command = Get-Command $name -CommandType Application -ErrorAction SilentlyContinue
    if ($null -ne $command) {
        $launchers += [pscustomobject]@{ Path = $command.Source; Prefix = @() }
    }
}

$pyLauncher = Get-Command "py" -CommandType Application -ErrorAction SilentlyContinue
if ($null -ne $pyLauncher) {
    $launchers += [pscustomobject]@{ Path = $pyLauncher.Source; Prefix = @("-3") }
}

$python = $null
foreach ($launcher in $launchers) {
    $prefix = $launcher.Prefix
    & $launcher.Path @prefix --version *> $null
    if ($LASTEXITCODE -eq 0) {
        $python = $launcher
        break
    }
}

if ($null -eq $python) {
    throw "Python 3 was not found."
}

Push-Location $projectRoot
try {
    $prefix = $python.Prefix
    & $python.Path @prefix $generator
    if ($LASTEXITCODE -ne 0) {
        throw "Prototype texture generation failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}
