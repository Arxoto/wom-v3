function Reset-Fnm {
    # 注入 fnm 环境变量
    fnm env | Out-String | Invoke-Expression
    # 切换正确的 Node 版本
    fnm use

    # 验证环境
    Write-Host "Current Node version: $(node -v)" -ForegroundColor Cyan
    Write-Host "Current pnpm version: $(pnpm -v)" -ForegroundColor Cyan
}
