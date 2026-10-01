# 01 — 打通传递路径：`Plugin Folder` 的打包与两态定位

Status: ready-for-human
Category: enhancement

## 目标

让磁盘上真的出现一个 `plugins/` 目录，并**实测** `resolve_resource("plugins")` 在
`tauri dev` 与 `tauri build` 两态各解析到什么绝对路径。

这条必须**第一个做**：spec §1.1 与 §3.1 都以"两态都能定位到 `Plugin Folder`"为前提，
它没确认之前后面每一步都建在未验证的假设上。

## 范围

1. 仓库根新建 `plugins/`，放进一个探针 `Plugin Package`（本 issue 只放占位文件，
   内容由 issue 02 起填）。
2. `src-tauri/tauri.conf.json`：
   - `bundle.resources` 声明 `"../plugins/": "plugins/"`；
   - `app.security.assetProtocol.enable = true`，scope 覆盖 `$RESOURCE/plugins/**/*`。
3. 一个临时的诊断路径：把 `resolve_resource("plugins")` 的结果写进日志（`tauri dev` 与
   `tauri build` 各跑一次），确认：
   - dev 态解析到哪（预期在 target 目录下）；
   - bundle 态解析到哪（macOS 的 `Contents/Resources/`、Windows/Linux 的安装根附近）；
   - asset protocol 的 scope 对**两态**都命中。
4. 实测"打包后加插件"：`tauri build` 出来的安装目录里手工加一个新目录，确认路径可读
   （issue 02 的列目录还没写，所以本 issue 只确认**路径与可读**，不确认被扫到）。

## 验收

- 两态的解析路径写进 issue 的 `## Answer`。
- scope 命中与否有明确结论（命中/不命中，以及不命中时怎么改）。
- `cargo build` 干净。

## 不在范围

列目录、读清单、代理插件、任何 JS 装载（issues 02–07）。

## Answer

### 结论：**实测没做到，两态路径这一条还没有答案**

`cargo build` 与 `tauri dev` / `tauri build` 在本会话里**跑不起来**，所以 §验收 的三条
（两态路径、scope 命中、打包后加插件）一条都还没有实测结论。下面是卡在哪、以及已经做完的部分。

### 卡点：shell 写不了 `src/` 与 `src-tauri/`

```
cargo build --manifest-path src-tauri/Cargo.toml
error: failed to write D:\develop\code\rust\wom-v3\src-tauri\Cargo.lock
Caused by: failed to open: ...Cargo.lock
Caused by: 拒绝访问。 (os error 5)
[exit code: 1]
```

逐目录实测（`Set-Content` 探针，已全部清理）：

| 路径 | shell 写入 |
| --- | --- |
| `wom-v3\`（仓库根） | 可以 |
| `wom-v3\plugins\` | 可以 |
| `wom-v3\src\` | **拒绝** |
| `wom-v3\src\plugins\` | **拒绝** |
| `wom-v3\src-tauri\` | **拒绝** |
| `wom-v3\src-tauri\target\` | **拒绝** |
| `%TEMP%` | **拒绝** |

`src-tauri` 上的显式权限是 `Everyone:(I)(CI)(DENY)(DC)` 加一条 `JM\Jesus:(F)`，
但 shell 里的进程拿的是受限令牌，写不进去。**DSH 自己的写文件工具能写这些目录**
（本轮所有 Rust / JSON / TS 改动都是这样落地的），只有 shell 里跑的程序不行——
所以"改代码"没问题，"编译与实测"被挡住。

`.scratch/acl-report/` 里记着上一次对 `src\main\item` 的同类修复以 `refused` 结束，原因是
`D:\` 缺 `WRITE_DAC`。按流程**没有重跑**那个修复脚本，等人工处理权限。

一个连带影响：`cargo build` 失败发生在读清单阶段，所以本 issue 里**连一次日志都没打出来**。

### 已经做完的（代码都在工作树里，等权限一到就能跑）

1. **仓库根 `plugins/probe/`**：探针 `Plugin Package` 占位——`manifest.yml`（`id: probe`、
   `keywords: probe,wom-probe`、`entry: index.js`、`icon: icon.png`、两个 `actions`、
   一个 `types`）、`index.js`（只留一个 `window.__WOM_ISSUE01_PROBE__ = "executed"` 标记）、
   `icon.png`（16x16）。
2. **`src-tauri/tauri.conf.json`** 两处：
   - `bundle.resources` = `{ "../plugins/": "plugins/" }`；
   - `app.security.assetProtocol` = `{ "enable": true, "scope": ["$RESOURCE/plugins/**/*"] }`。
3. **`src-tauri/Cargo.toml`**：`tauri` 的 features 显式加上 `protocol-asset`。
   这一条**比 spec §五 多了一处改动**，理由是实测出来的：
   - `AssetProtocolConfig::enable` 只被 `tauri-utils` 的 `Config::features()` 消费，
     也就是**只有 tauri CLI 构建时才补上这个 feature**；
   - `cargo build` / `cargo test` 不走 CLI，`Manager::asset_protocol_scope()` 直接被
     `#[cfg(feature = "protocol-asset")]` 挡掉，asset protocol 也根本不注册；
   - 在 CLI 的原生二进制里确实能搜到 `protocol-asset` 字符串（偏移 13712255），
     说明 CLI 那条路是通的——显式写一遍只是让两条路都一样，而不是绕开它。
   - 代价：`Cargo.lock` 要加一条 `http-range 0.1.5`（本地 registry 已有源码与 `.crate`，
     更新不需要联网）。
4. **临时诊断路径**（issue 01 结束后删除，代码注释里都标了）：
   - `plugin_host::plugins_dir`：`resolve_resource("plugins")` + 一条
     `plugin folder: path=... exists=...` 日志（ASCII）；
   - `plugin_probe.rs` + 命令 `plugin_probe_entry_path`：报出探针入口绝对路径，
     并现算一次 `asset_protocol_scope().is_allowed(entry)`，记一条
     `probe asset scope: entry=... allowed=...`；
   - `src/plugins/probe_asset.ts`：前端把那个路径经 `convertFileSrc` 注入一次
     `<script src>`，按 `onload` / `onerror` 记一条 `[issue01]` 日志——
     这一条同时验证"文件真被 asset protocol 送出"与"cargo build 不带
     `protocol-asset` 时会 403"两件事；
   - release 态的日志目标加了 `Webview`：打包态没有 stdout，前端记的那两条只能靠日志文件。
5. `pnpm exec tsc --noEmit` **干净**（用 fnm 里的 node v22.23.2 直接跑，
   因为本会话 PATH 里没有 `node`）。

### 环境侧的两个额外发现

- **PATH 里没有 `node`**：`pnpm` 在，`node` 不在。本轮起是手动拼的
  `C:\Users\Jesus\scoop\apps\fnm\current\node-versions\v22.23.2\installation\node.exe`
  加 `node_modules\typescript\bin\tsc`。这与 `AGENTS.md` 说的"Node 环境由用户切好"不一致，
  但没有另找 Node、也没有调 `fnm`。
- **留下一个删不掉的探针文件**：`src-tauri\dsh-acl-probe.tmp`（16 字节，内容是
  `acl write probe`，被 git 标成未跟踪）。这是我为了确认"DSH 能写、shell 不能写"造的，
  同一个权限问题让它删不掉。**请人工删掉**，不要当成仓库内容提交。

### 下面这些仍然是**推断**，不是实测，权限恢复后要逐条验

- 开发态 `resolve_resource("plugins")` → `src-tauri/target/debug/plugins`：
  依据是 `tauri-utils::platform::resource_dir` 在 Windows 上直接返回
  "可执行文件所在目录"，而 `tauri-build` 把 `bundle.resources` 拷到 `OUT_DIR` 往上第三级
  （即 `target/<profile>`）。
- 打包态（Windows）→ 安装目录（`wom.exe` 旁边）的 `plugins\`。
- scope 对两态都命中：`is_allowed` 会先 canonicalize 再按 glob 匹配，
  而 `push_pattern` 会同时插入原样与去掉 verbatim 前缀（`\\?\`）两种写法。
- 打包后新加目录可读：scope 是构建期 glob，`**/*` 不是文件快照。

## Comments

### 权限恢复后的复测（后续一次会话）

状态从 `blocked-on-environment` 改成 `ready-for-human`：环境（写权限）已经恢复，
开发态那三条也都实测过了，剩下的只有打包态实测一步。

- **写权限已恢复**：`src/`、`src-tauri/` 都能写，`cargo build` / `cargo test` /
  `pnpm build` 都跑得起来；上面那条 `src-tauri/dsh-acl-probe.tmp` 与
  `src-tauri/src/plugin_probe.rs`（issue 01 的临时诊断模块）都已删除。
- **开发态落点已实测**：`cargo build` 之后 `bundle.resources` 的 `../plugins/` 被
  `tauri-build` 拷进 `src-tauri/target/debug/plugins/`，探针包（`manifest.yml` /
  `index.js` / `icon.png`）在里面。Windows 上 `resolve_resource` 解的是可执行文件所在目录，
  所以开发态的 `Plugin Folder` 就是 `target/debug/plugins`。这一条从"推断"变成"实测"。
- **运行时返回值也实测了**（`pnpm tauri dev` 起来看日志）：

  ```
  [plugin_host][INFO] plugin folder: path=\\?\D:\develop\code\rust\wom-v3\src-tauri\target\debug\plugins exists=true
  [plugin_host][INFO] js plugin ready: probe
  [plugin_framework][INFO] plugin registered: probe
  ```

  即开发态解析到 `src-tauri/target/debug/plugins`（带 `\\?\` verbatim 前缀，与
  `push_pattern` 两种写法都插入的推断一致），目录存在，探针包被扫到、代理注册成功。
- **asset protocol 的 scope 也实测命中了**（开发态）：探针插件是经
  `convertFileSrc(entry)` + `<script src>` 装载的，它**真的执行了**——插件的搜索结果页画了出来、
  结果行的动作也在插件里跑出了日志（见 issue 07 的 Answer）。清单里的 `icon.svg` 同样经
  asset URL 画到了条目上。
- **还没实测**：打包态落点、打包后往安装目录加一个目录能不能被扫到。这两条要出安装包，
  见 issue 07 的 Answer。
- 上一轮提到的 `src-tauri/tauri.conf.json`（`bundle.resources` + `assetProtocol`）与
  `src-tauri/Cargo.toml`（显式 `protocol-asset`）两处改动保持不变。

