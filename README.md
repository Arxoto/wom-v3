# WOM

## How to dev

use [pnpm tauri dev (注意 node 和 pnpm 版本)](./build_scripts/run_dev.ps1)

## How to build&bundle

use [pnpm tauri build (注意 node 和 pnpm 版本)](./build_scripts/run_build.ps1)

如果因为网络原因打包失败（下载部分工具失败），可以手动下载

### v1.x

源码位置，注意分支换到对应依赖的版本

- [wix_path](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/msi.rs#L30)
  - [WIX_URL](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/msi/wix.rs#L35)
- [nsis_toolset_path](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/nsis.rs#L67)
  - [NSIS_URL](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/nsis.rs#L33)
- [NSIS_REQUIRED_FILES.ApplicationID](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/nsis.rs#L59)
  - [NSIS_APPLICATIONID_URL](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/nsis.rs#L38)
- [NSIS_REQUIRED_FILES.nsis_tauri_utils](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/nsis.rs#L60)
  - [NSIS_TAURI_UTILS_URL](https://github.com/tauri-apps/tauri/blob/tauri-build-v1.5.0/tooling/bundler/src/bundle/windows/nsis.rs#L36)

P.S.部分版本可能没有 `NSIS_APPLICATIONID_URL` ，具体看对应版本的源码，如果没有则跳过

手动解压

1. 下载 `WIX_URL` `NSIS_URL` `NSIS_APPLICATIONID_URL` `NSIS_TAURI_UTILS_URL`
1. 资源管理器进入 `%LocalAppData%`
1. 创建目录 `mkdir .\tauri\WixTools .\tauri\NSIS`
1. `%LocalAppData%\tauri\WixTools` <- 解压 `wix311-binaries.zip`
1. `%LocalAppData%\tauri\NSIS` <- 解压 `nsis-3.zip`
1. `%LocalAppData%\tauri\NSIS\Plugins\x86-unicode` <- 复制 `NSIS-ApplicationID.zip/ReleaseUnicode/ApplicationID.dll`
1. `%LocalAppData%\tauri\NSIS\Plugins\x86-unicode` <- 复制 `nsis_tauri_utils.dll`

### v2.x

源码位置，注意分支换到对应依赖的版本

- [wix_path](https://github.com/tauri-apps/tauri/blob/tauri-v2.10.3/crates/tauri-bundler/src/bundle/windows/msi/mod.rs#L64)
  - [WIX_URL](https://github.com/tauri-apps/tauri/blob/tauri-v2.10.3/crates/tauri-bundler/src/bundle/windows/msi/mod.rs#L39)
- [nsis_toolset_path](https://github.com/tauri-apps/tauri/blob/tauri-v2.10.3/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs#L87)
  - [NSIS_URL](https://github.com/tauri-apps/tauri/blob/tauri-v2.10.3/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs#L38)
- [NSIS_REQUIRED_FILES.nsis_tauri_utils](https://github.com/tauri-apps/tauri/blob/tauri-v2.10.3/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs#L70)
  - [NSIS_TAURI_UTILS_URL](https://github.com/tauri-apps/tauri/blob/tauri-v2.10.3/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs#L42)

记录 v2.10.3 对应的下载连接和哈希

- [WIX_URL](https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip)
  - WIX_SHA256 `6ac824e1642d6f7277d0ed7ea09411a508f6116ba6fae0aa5f2c7daa2ff43d31`
- [NSIS_URL](https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.11/nsis-3.11.zip)
  - NSIS_SHA1 `EF7FF767E5CBD9EDD22ADD3A32C9B8F4500BB10D`
- [NSIS_TAURI_UTILS_URL](https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll)
  - NSIS_TAURI_UTILS_SHA1 `75197FEE3C6A814FE035788D1C34EAD39349B860`

手动解压

1. 下载 `WIX_URL` `NSIS_URL` `NSIS_TAURI_UTILS_URL`
1. 资源管理器进入 `%LocalAppData%`
1. 创建目录 `mkdir .\tauri\WixTools314 .\tauri\NSIS`
1. `%LocalAppData%\tauri\WixTools314` <- 解压 `wix314-binaries.zip`
1. `%LocalAppData%\tauri\NSIS` <- 解压 `nsis-3.11.zip`
1. `%LocalAppData%\tauri\NSIS\Plugins\x86-unicode\additional` <- 复制 `nsis_tauri_utils.dll`
