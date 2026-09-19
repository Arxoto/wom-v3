#requires -Version 7.0

. "$PSScriptRoot/fnm_env_use.ps1"

Push-Location $RepoRoot
try {
    pnpm tauri dev
}
finally {
    Pop-Location
}
