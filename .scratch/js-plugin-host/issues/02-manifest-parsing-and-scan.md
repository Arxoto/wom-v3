# 02 — 清单的解析与 `Plugin Folder` 的扫描

Status: resolved
Category: enhancement

## 目标

在 Rust 侧把 `Plugin Folder` 扫成一份合法 `Plugin Package` 的列表，坏包**跳过并记 warn**，
不 panic、不中断启动。

## 范围

1. **清单解析器**（手写 `key: value` 行格式，spec §1.3）：
   - `#` 起头为注释，空行忽略，值不做引号与转义；
   - 字段与缺省值见 spec §1.3 的表；`id` 与 `keywords` 必填；
   - 非 ASCII 的 `id` 判为非法（`PluginId` 是稳定 ASCII 标识）；
   - 错误是**这个包的错误**，不是框架的错误（第一轮的 Q18/Q19 分层）。
2. **目录扫描**：`resolve_resource("plugins")` → 列子目录 → 逐个找 `manifest.yml`。
   目录不存在时不让应用起不来（记 warn）。
3. **重扫**：提供一个"重新扫一遍"的函数（issue 04 把它挂到临时托盘项上）。
4. **单测**（与用户对齐后再写，见 AGENTS.md）：合法清单、缺 `id`、缺 `keywords`、
   注释与空行、值里带空格、重复 `id`。

## 不在范围

不注册任何插件（issue 03 才做）、不碰前端。

## Blocked by

01（路径得先能解析对）。

## Answer

落在 `src-tauri/src/plugin_impl_js/`：

- `manifest.rs`：手写 `key: value` 解析（`PackageManifest::parse`）。`#` 起头是注释、空行忽略、
  按**第一个** `:` 切开、键与值两侧去空白、值内部原样、同名键后写的赢；认不出的键忽略。
  `id` 必填且必须是稳定 ASCII（字母数字与 `-` `_` `.`），`keywords` 必填且不能全是空项，
  `entry` 缺省 `index.js`，`name` 缺省 `id`；逗号列表去空白、丢空项、去重。
  错误类型 `ManifestError`（`BadLine(行号)` / `MissingId` / `BadId` / `MissingKeywords`），
  诊断文本是 ASCII。
- `package.rs`：`read_package(dir)` 读一个包，`scan(folder)` 列子目录逐个读。
  坏包**跳过并记 warn**（目录读不了、清单打不开或坏掉、`id` 重复），目录不存在也只是一条 warn；
  结果按 `id` 字典序排定，与 `read_dir` 的返回顺序无关。
- 重扫：宿主侧 `plugin_host::reload_packages` → `sync_js_packages`（issue 03/04 用）。

对 issue 01 的依赖落在宿主组装里：扫描用的是 `resolve_resource("plugins")`，路径不对时
只是扫出 0 个包（一条 warn），不会让应用起不来。

`cargo build` 干净。单测按 AGENTS.md 留给用户（本 issue 的"单测"一条未做）。
