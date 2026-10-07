# 应用命令按窗口角色发 ACL，插件窗口直接拿 `__TAURI__`

## 洞在哪

`src-tauri/permissions/` 为空时，应用**没有自己的 ACL 清单**，而 Tauri 只在
"命令属于某个插件、或应用有清单、或请求来自非本地来源"这三种情况下才查 ACL
（`tauri::Webview::on_message`）。于是应用自己的命令（`fetch_config`、`save_config`、
`plugin_*` …）**一条都不查**：任何本地 webview，包括插件包自己那一页，都能调它们。

同一份清单也是"插件窗口能不能用 `__TAURI__`"的前提：`__TAURI__` 由
`app.withGlobalTauri` 在编译期决定装不装，而装上了也还得有匹配窗口的 capability，
否则每个 `__TAURI__.*` 的调用都会被 ACL 挡掉。

## 决定一：应用命令写成权限，按窗口角色发

`src-tauri/permissions/` 下写两条应用权限，一条一扇窗：

- `main-window`：主面板用到的 11 条（检索、派发、显示/隐藏/读配置）；
- `config-window`：配置窗口用到的 5 条（读配置、存配置、注册快捷键、重建主窗口）。

`capabilities/default.json`（窗口 `main`）与 `capabilities/config.json`（窗口 `config`）
各自引用自己那一条。**只有 `main` 能跑检索与动作，只有 `config` 能写配置**——
这正是 capability 文档里说的"按窗口需要分权，压缩次要窗口被攻破的后果"。

粒度选"一扇窗一条权限"而不是"一条命令一条权限"：命令本身就是按窗口划分的，
一条命令一条会多出十几个只被引用一次的文件，还得再写一层 set 把它们拼回窗口。
真出现"同一扇窗里要更细的开关"时再拆。

## 决定二：`app.withGlobalTauri` 打开，插件窗口拿到 `__TAURI__`

插件包的那一页是插件作者自己写的 HTML，不经过 Vite，手上没有 `@tauri-apps/api`，
只能吃全局注入的那一份。所以 `tauri.conf.json` 打开 `app.withGlobalTauri`，
注入包含 core 与已注册插件（log / notification / opener / clipboard-manager /
global-shortcut）的 `window.__TAURI__`。

`withGlobalTauri` 是应用级开关，主窗口与配置窗口会一起拿到这个全局；它们本来就用
npm 包走 `__TAURI_INTERNALS__`，多一份 `__TAURI__` 只是多一个名字。

## 决定三：插件窗口的 capability

新增 `capabilities/plugin.json`，`windows: ["plugin-*"]`（glob 匹配
`plugin_window` 拼出的 `plugin-{PluginId}` 标签），`local: true`，
权限是 `core` 的 app / event / image / path / resources / webview / window 七份 default，
加上 `core:window:allow-close`、`log:default`、`notification:default`、`opener:default`。

三条边界：

1. **不给 `core:default` 整包**：它含 `core:menu:default` 与 `core:tray:default`，
   而菜单与托盘是跨窗口的应用级界面，只活到插件页关掉为止的一页不该去动它们
   （其余几份 default 逐个列出，正是为了漏掉这两份）。
2. **不给任何应用命令**：插件页要的是通用 API，不是宿主的检索、动作或配置。
3. **只认本地**：`local: true`，插件页导航到外部站点后这份能力不再适用
   （`plugin_window` 的导航守卫本来就只放行应用页与 `*.localhost`，这是第二道）。

代价：插件页可以自己关掉自己的窗口（`allow-close` 是故意给的），但拿不到剪贴板
（本应用连主窗口都还没发剪贴板权限）；要按插件分发更细的能力，得等有插件真提出需求。

## 与 ADR-0013 的关系

ADR-0013 结尾那句"没有新的 capability"到此作废——插件窗口这一轮起有专属 capability。
（那一篇里"窗口装承载页、插件页在 iframe 里"的整节也已经与代码不符：`plugin_window`
现在直接装插件页并在 `initialization_script` 里注入外框占位。这一段漂移与本 ADR 无关，
留待单独收。）
