# 04 — 宿主侧命令：列出插件包、重扫、以及给前端的两个入口

Status: resolved
Category: enhancement

## 目标

命令层与 capability 接线：前端能问"有哪些插件包"，能触发一次重扫；
后面两个 issue 要用的入口在这里先占好位。

## 范围

1. `commands.rs` 新增（只做转发，实现留在不依赖 Tauri 的那一层）：
   - `plugin_list_packages` → `id` / `name` / `entry` 是否存在之类，供验证与重扫后核对；
   - `plugin_reload_packages` → 重扫 `Plugin Folder`（issue 03 的注册路径重跑一遍）。
2. `capabilities/default.json`：给新命令放行。**只加窗口 `main`**，不加配置窗口（spec §五）。
3. 临时托盘菜单项（spec §五，Q25 的 (b)）：调 `plugin_reload_packages`。
   **在代码注释里标明这是临时验证入口，交付时删除或转正。**

## 不在范围

`plugin_open_plugin_search` 与结果回程命令在 issue 06 加（它们依赖 webview 装载的时序）。

## Blocked by

03。

## Answer

四个命令都在 `commands.rs`，实现落在宿主侧（这一层**必须**知道 Tauri：
回程靠事件、状态靠 `State`，与 launcher 那种"实现留在不依赖 Tauri 的那一层"不同）：

| 命令 | 实现 |
| --- | --- |
| `plugin_list_packages` | `plugin_host::list_packages` → `[PackageInfo]`（`id` / `name` / 入口绝对路径 / `entry_exists`） |
| `plugin_reload_packages` | `plugin_host::reload_packages` → 重扫 `Plugin Folder`，新包注册、已有 id 换代理重 init、消失的包**保留** |
| `plugin_open_plugin_search` | `plugin_host::open_plugin_search`（async）：去程事件 + 等回程，返回 `ItemSearchPage` |
| `plugin_report_search_results` | `plugin_host::report_search_results`：把结果行交给等着的 `recv()` |

临时托盘项："Reload Plugin Packages"（菜单 id `reload_packages`），代码注释里标了它是
issue 04 的临时验证入口，转正或删除由接口定稿时决定。

**`capabilities/default.json` 没有改动**，与 spec §五 的预期不同——查了一遍原因：
Tauri v2 的 capability 放行的是**插件命令**（`core:*` / `opener:*` / `log:*` …），
应用自己的命令（`search`、`run_item_action`、`plugin_search` …）根本不走 ACL。
现有这份文件里也没有任何一条应用命令，而它们一直可用，所以新命令同样不需要条目。
加一条不存在的权限标识反而会让构建失败。
