# 03 — 代理 `Plugin`：让每个 `Plugin Package` 在框架里有一个块

Status: resolved
Category: enhancement

## 目标

每个合法 `Plugin Package` 在 `PluginRegistry` 里占据一个插件块，且**在 `setup` 里同步完成**——
框架的公开面**一行不扩**。

## 为什么需要代理

`PluginRegistry::run_action` 是按 `ItemHandle` 找到插件块、调那个块里的 Rust `Plugin` 对象
（`plugin_framework/mod.rs:581`）。没有代理，插件注册的条目一被触发就会走到一个不存在的插件上。
代理与 `index.js` **共用同一个 `PluginId`**（清单的 `id`）。

## 范围

1. 新模块（放哪见 spec §六的布局约束：与 `plugin_framework` / `plugin_impl_launcher` **顶层平级**，
   **不要**新建 `plugins/` 父目录）。
2. 一个 `Plugin` 实现，字段最小：`PluginId` + 清单解析出来的内容。
   - `id()` → 清单的 `id`；
   - `init()` → 推入**它唯一的那一条 `Plugin Item`**（`id` / `name` / `desc` / `keywords` / `icon`
     来自清单）。走现成的 `ItemRegistrar`，**不新增框架入口**。
   - `actions()` → 清单 `types` × `actions` 的笛卡尔积，`label_key` 按
     `action.<插件 id>.<结果类型名>.<动作 id>` 四段式推导；
   - `run_action()` → 本 issue **只做到记日志 + 返回 `NoOp`**，真正的派发在 issue 06。
3. `plugin_host::create_registry` 里逐个注册代理（在 `LauncherPlugin` 之后）。
4. 插件级隔离照旧：某个包坏掉只 warn + 跳过（第一轮的 Q14），其余照常。

## 不在范围

任何 JS、任何前端、`open_search`（issue 06）。

## Blocked by

02。

## Answer

代理是 `src-tauri/src/plugin_impl_js/mod.rs` 的 `JsPlugin`（与 `plugin_framework` /
`plugin_impl_launcher` 顶层平级的新模块，没有新建 `plugins/` 父目录）。框架的公开面**没有新增入口**：

- `id()` / `actions()` 取清单：动作表 = 插件条目自己的一个动作（类型 `js_plugin`，
  动作 `open_search`，文案键 `action.js_host.js_plugin.open_search`）加上清单里
  `types` × `actions` 的笛卡尔积，`label_key` 按四段式 `action.<插件 id>.<类型>.<动作>` 推导。
- `init()` 推**唯一的那一条** Plugin Item（`js_plugin` 类型，关键字与名字、描述来自清单，
  图标是清单 `icon` 拼出的绝对路径），走现成的 `ItemRegistrar`。
- 插件级隔离照旧：`init` 失败由框架记 warn 并跳过，其余插件照常。

清单在 `init` 里**重读**，所以"重扫 = 重新 `init`"这条现成语义就够用：名字、关键字、
动作改了都能生效，不必换实现。

为了运行期重扫能注册新包，框架新增了一个入口 `PluginRegistry::register_or_reload_plugin(&self)`：
已存在的 `id` 换实现并重新 `init`，没有的直接注册。它与装配期的 `register_plugin(&mut self)`
（重复注册仍然跳过）**只差"已存在时怎么办"**；取 `&self` 是因为注册表 `manage` 进 Tauri 之后
宿主手上只有 `State`。`place_plugin` 是两者共用的落位逻辑。

`run_action` 在这一轮记日志 + 返回 `NoOp` 的"只记日志"约定**已按 issue 06 提前落地**为
发一条动作请求给 webview（见 issue 06 的 Answer）；插件条目自己那一条仍然回 `NoOp`
（打开搜索页由前端直接发起）。
