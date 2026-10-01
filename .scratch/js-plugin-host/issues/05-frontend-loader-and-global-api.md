# 05 — 前端：`<script src>` 装载插件、全局窄接口、插件注册

Status: resolved
Category: enhancement

## 目标

把 `Plugin Package` 的 JS 入口装进 webview，并把它的类型、动作图标与文案表交进前端注册表。

## 硬约束

**不许 `eval` / `new Function`**（ADR-0011）。装载方式：宿主拼出绝对路径 →
`convertFileSrc` → 注入 `<script src>`。取文件与执行都是 webview 自己的行为。

## 范围

1. **装载器**：给定 `PluginId`，读它的 `entry`，注入 `<script src>`，返回一个
   **"装载完成"的 Promise**（见下）。
2. **全局窄接口**（spec §3.2，同时改写 ADR-0010:19）：
   ```js
   window.__WOM_PLUGIN__ = { register(spec), log(text), fail(text) }
   ```
   **只挂这一个对象，不挂 registry 本身**；`register` 转调 `src/plugins/registry.tsx` 的 `register`。
3. **插件如何交出搜索函数**：`register(spec)` 的 spec 里除了 ADR-0010 的
   `types` / `labels`，再加一个搜索函数（形状由本 issue 定，写进 spec 的补充）。
4. **时序不变量（最要紧的一条）**：装载完成后**先等已注册、再允许发起搜索**。
   `label_key` 与图标必须在结果行渲染之前到位（spec §2.4）。实现上用一个 Promise
   把 `open_search` 串在后面。
5. **失败隔离**：`<script src>` 的 `onerror`、插件抛错、`register` 传了非法 spec，
   三种都要记日志 + 丢弃该插件，**主窗口不受影响**。
6. **懒加载**：只在用户第一次进入某个插件的搜索页时才装载（spec §3.5 / Q25(a)）。
   启动时**不**装载任何插件代码。

## 验收

用一个手写的探针插件，验证：装载成功、注册进前端注册表（图标与文案生效）、
抛错时主窗口照常。

## Blocked by

04（要能知道有哪些包、`entry` 在哪）。

## Answer

落在 `src/plugins/host.ts`（前端一半）与 `src/plugins/js_host.tsx` / `js_host_icons.tsx`
（插件条目那一条注册）：

1. **装载器**：`ensure_loaded(plugin_id, entry)`，每个插件一个 Promise；
   `inject_plugin_script` 用 `convertFileSrc(entry)` + `<script src>`，**没有 eval / new Function**
   （ADR-0011）。"已注册"的判据是脚本 onload 之后 `registry.has(plugin_id)` 为真——
   文件到手不算装好。装载失败**不记账**，下次进搜索页重试。
2. **全局窄接口**：`window.__WOM_PLUGIN__ = { register, log, fail }`，只挂这一个对象；
   `register` 转调 `src/plugins/registry.tsx` 的 `register`，而且**先过一道校验**
   （`id` 非空字符串、`types` 是数组、`labels` 是对象），非法 spec 整份丢掉并记日志。
3. **搜索函数的形状**（spec 的补充）：`register` 的 spec 里多了两个字段
   `search(keyword) => rows` 与 `run(row, action_id)`。它们不进界面读的那张表——
   注册表把 `types` / `labels` 与这两个函数分开存。行形状是
   `{ the_type, name, desc, action_ids }`（镜像 Rust 侧 `plugin_impl_js::SearchRow`）。
4. **时序不变量**：`on_search_request` 先 `await ensure_loaded(...)`，装好之后才调插件的
   `search`，拿到行才回传。Rust 收到回程时图标与文案一定已经在注册表里。
5. **失败隔离**：`onerror`、`search` 抛错、非法 spec 三条都只记日志并让这一次搜索"没有结果"，
   主窗口不受影响。
6. **懒加载**：入口只装 `window.__WOM_PLUGIN__` 与两个事件监听，不装载任何插件代码；
   第一次进某个插件的搜索页才注入它的脚本。

图标形状上补了一条 spec 没定的口子：JS 插件手上没有 React，交不出 `ReactNode`，
所以注册表接受 `ReactNode | string`（字符串按图片地址画成 `<img>`，探针用 data URI）。
`plugins/plugin_icon.tsx` 是唯一画图标的地方。
