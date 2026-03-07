#requires -Version 7.0

. "$PSScriptRoot/fnm_env_use.ps1"

Reset-Fnm

pnpm tauri build
