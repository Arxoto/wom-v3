# 前端插件：清单写 `html` 就是无代码插件，条目的动作直接开页面

`Plugin Package` 原本一律要有 JS 入口（`index.js`，见 ADR-0011）：Rust 侧读清单建索引，
webview 侧起 Worker 装载插件代码，插件自己跑搜索、注册结果行的图标与文案。对"只想挂一个
页面"的插件，这一整套（清单里的 `types` / `actions`、一份 `index.js`、一个 Worker）都是过路费。

这一轮加**前端插件**：`manifest.json` 声明 `"type": "html"` 并给出 `"html": "<包内相对路径>"`，
这个包就是前端插件——不需要 JS 入口，条目只有一条、动作只有一个，触发它直接打开那一页。

## 包的形态由 `type` 显式声明

`"type": "js"` 是原来的 JS 插件，`"type": "html"` 是前端插件（清单为什么是 JSON、`type`
为什么是必填，见 [ADR-0014](./0014-manifest-json-and-explicit-package-type.md)）。两种包共用同一个扫描
（`plugin_package::scan`）与同一套清单解析，只是注册的代理不同：JS 包注册
`plugin_proxy_js::JsPlugin`，前端包注册新加的 `plugin_proxy_html::HtmlPlugin`。

前端插件的条目类型由宿主定（`html_plugin`），动作是 `open_html`；清单里的
`entry` / `types` / `actions` 对它没有意义，被忽略。`keywords` / `name` / `desc` / `icon`
照旧——它也照旧靠主检索被搜到，所以 ADR-0011 那条"插件能不能被搜到不依赖它的代码能否执行"
在这里是最强形式：前端插件根本没有代码可执行。

## 打开页面复用既有的那条路

`HtmlPlugin::run_action` 与命令 `plugin_open_html_window` 都落到 `plugin_window::open`——JS
插件经 `self.__WOM_PLUGIN__.open_window` 走的也是它（前端插件按自己的包目录解析路径，
不必经由宿主那本账）。包内相对路径的约束（`resolve_in_package`）、"一个包一个窗口、再开一次
是换页 + 聚焦"全部原样复用，没有新的 capability。动作回 `ActionOutcome::Done`，主窗口照
`main_window_mode` 隐藏，与别的动作一致。

## 窗口装承载页，插件页面在 iframe 里打开

到 asset URL 的那一步一度是 Rust 手写的（`plugin_window::asset_url` + `percent_encode`：自己挑
平台前缀、自己按 `encodeURIComponent` 转义），与"宿主只给路径、前端转 asset URL"这条既有分工
相悖——条目图标就是这么走的。现在窗口装的是应用自己的 `index_iframe.html`：宿主只把页面
**绝对路径**记到 `PluginWindowPages`，新增命令 `plugin_window_page` 让承载页按自己的窗口标签
取回它，再由前端 `convertFileSrc` 拼 iframe 的 `src`。Rust 侧不再有任何 URL 拼装与转义。

代价：多一个前端入口与一条命令；插件页面从此在 iframe 里而不是顶层（`window.close()` 这类
顶层假设不再成立），顶层导航守卫也从"只认 asset"改成放行应用页与 `*.localhost`。

> 后续：这一节被推翻了（`763c63f`）：窗口改回**直接装插件页面**——`plugin_window::open` 借主窗口的
> `convert_file_src` 把包内路径换算成 asset URL，用 `WebviewUrl::CustomProtocol` 直接装它，
> 外框占位改由 `initialization_script` 注入；`index_iframe.html`、`PluginWindowPages` 与
> `plugin_window_page` 命令一并删除，插件页面回到顶层。上面"打开页面复用既有的那条路"结尾的
> "没有新的 capability"同时作废：插件窗口从 [ADR-0015](./0015-command-acl-and-plugin-window-api.md)
> 起有自己的 capability（`capabilities/plugin.json`，窗口 `plugin-*`）。

## 代价

- 前端插件是"一条条目 + 一个动作"的最小形态：它不能注册多条结果行，也没有 Plugin Search
  Page。真需要那些的包仍然走 JS 插件。
- 条目图标的兜底与动作的中文由前端注册表给（`src/plugins/html_host/register.tsx`），
  中文不住 Rust。
- 窗口这一轮仍是系统原生边框，与 ADR-0011 的 `open_window` 同一条。
