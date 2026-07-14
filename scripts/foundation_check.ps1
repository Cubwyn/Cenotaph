param(
    [switch]$SkipClippy
)

# Compatibility entry point. The canonical whole-project check now has a name
# that reflects its runtime, content, save, and developer-tool coverage.
$projectCheck = Join-Path $PSScriptRoot "project_check.ps1"
& $projectCheck -SkipClippy:$SkipClippy
