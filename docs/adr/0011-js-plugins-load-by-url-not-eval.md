# JS 插件不 eval：清单承担索引，代理承担派发，插件代码按需装载

JS 插件入口以 `<script src>`（asset protocol）注入 webview，**宿主从头到尾不接触插件源码字符串**，
不用 `eval` / `new Function`。理由是领域惯例：uTools、Raycast、Alfred、Flow Launcher、Wox
五家主流启动器**没有一家**在宿主界面里对插件源码求值——它们要么给插件一份自己的文档
（uTools / Raycast），要么把插件放进另一个进程（Alfred / Flow / Wox）。附带收益是 CSP 将来能
收紧到 `script-src asset:` 而不必开 `unsafe-eval`（`csp: null` 现在把这条依赖掩盖着，
一旦收紧，eval 那条路会静默坏掉）。

## 条目与关键字在清单里声明

`manifest.yml` 声明 `id`、`keywords` 与那一行 `Plugin Item` 的内容。这样 Rust 在应用 `setup`
里**同步**读完清单就能建索引，`Plugin::init` 推的还是现成的 `ItemRegistrar`，
**框架的公开面不需要新增入口**，也不需要任何"异步注册口"。于是
**插件能不能被搜索到不依赖它的代码能否成功执行**——清单坏了跳过那个包，代码坏了只是结果页为空。
uTools 的 `plugin.json` 里的 `features[].cmds` 是同一个形状。

## 一个 `Plugin Package` 在 Rust 侧有一个代理

`PluginRegistry::run_action` 按 `ItemHandle` 找到插件块、调那个块里的 Rust `Plugin` 对象。
所以每个包在 Rust 侧有一个占据插件块的**代理**实现，与 `index.js` **共用同一个 `PluginId`**，
否则它注册的条目一旦被触发，框架会走到一个不存在的插件上。

## 重开 ADR-0010 的两处

- ADR-0010:19 写着"注册表不往 `window` 挂任何东西"，理由是当时没有调用方。现在有调用方了：
  宿主挂一个窄接口 `window.__WOM_PLUGIN__ = { register, log, fail }` 给插件，
  **不挂 registry 本身**。
- ADR-0010:23 那条"`action_icon` 是全表查、跨插件同动作 id 会撞，等真有 JS 插件注册进来再谈"
  **到了要谈的时候**：它从"已知边界"变成"必须解决"。本轮只记录、不实现
  （没有第二个插件时定不下形状）。

## 代价

插件跑在宿主自己的文档与全局里：能摸 DOM、改 `window`、覆盖宿主界面。真正的隔离要么走 Worker
（插件没有 DOM，图标与文案必须全变纯数据，与 ADR-0010 的 `ReactNode` 注册形状冲突），
要么走独立进程，都不是本轮的范围。本轮唯一要装的插件是我们自己写的探针。
