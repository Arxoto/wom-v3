#requires -Version 7.0

$RepoRoot = Split-Path -Parent $PSScriptRoot

fnm env | Out-String | Invoke-Expression

Push-Location $RepoRoot
try {
    fnm use
}
finally {
    Pop-Location
}
