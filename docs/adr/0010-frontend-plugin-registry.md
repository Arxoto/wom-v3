# 前端插件注册表：动作表以它为准，内置与 JS 插件走同一条注册路径

界面那一层原本把一个插件的三件事散在两处：类型名 → 图标在 `src/main/item/item_icons.tsx`、动作 id → 图标在 `src/main/item/action_icons.tsx`、`label_key` → 中文在 `src/main/interaction/action_labels.ts`，而「某类型有哪些动作、顺序如何」来自 Rust 下发的动作表。这一轮把前两件与第三件收进 `src/plugins/`：`registry.tsx`（注册表本体：注册、查找、动作解析）、`launcher/launcher.tsx`（launcher 插件的注册内容）、两个图标文件（`launcher/launcher_icons.tsx` / `launcher/launcher_action_icons.tsx`）。旧的三处随之删除。

## 动作表以**前端注册表**为准

上一轮是「挂载时调 `fetch_plugin_actions` 拉一次动作表」（ADR-0009 的接线）。现在条目有哪些动作、顺序与默认动作由前端注册表给出，Rust 只把条目自带的 `action_ids` 随检索结果一起下发；`fetch_plugin_actions` 命令与 `PluginRegistry::action_table()` 一并收掉——它们此后没有调用方。框架内部仍会拼一张动作表（`ActionTableView` / `PluginActionView` 只是注册表私有的内部类型），但只为算条目自带的 `action_ids`，不为下发而暴露。

这么定的理由是顺序与文案本来就成对出现：`label_key` 按「类型 + 动作」分套，而 Rust 侧的键是静态字面量（`plugin_impl_launcher/action.rs`），前端要显示它就必须自己持有「类型 → 有序动作 → 文案」这一整份。留一份 Rust 下发、再与注册表合并，只会换来一个「注册表认不出就跳过这个动作」的分支和一段挂载后的空窗（拉到之前行里不画动作图标）。

代价是**这份顺序在前端有了第二份写法**，与 `plugin_impl_launcher/action.rs` 的动作表必须一致；不一致时界面显示的默认动作会与 Rust 期望的不同（动作本身仍按 id 派发，Rust 不校验顺序，所以不会报错，只会「跑的不是显示的那个」）。launcher 的四个类型就这么一份，改一边时要改另一边。

## 内置与 JS 插件同一条路径，挂载点仍未定

`register` 是唯一的写入口，内置的 launcher 走的也是它：`index_main.tsx` 在渲染之前调 `register_launcher(registry)`。将来的 JS 插件调同一个 `register`（Q39）——两条路径分开的话，JS 插件能用的能力会永远只是内置能力的子集。

注册放在**入口**而不是某个组件的挂载副作用里：两个 HTML 入口各挂一个 React app，`StrictMode` 下挂载会跑两遍；同一个插件 id 重复注册不再生效，于是重复调用是安全的。

JS 插件将来怎么拿到这个单例（webview 全局 / 可 import 的模块 / 事件）**仍然没有定**，所以注册表**不往 `window` 挂任何东西**（spec §五）：现在挂上去就是一个没有调用方的全局副作用。挂载点随时能加，单例已经只在自己的模块里导出。

> 后续：JS 插件宿主落地时，挂载点定了下来——宿主往 `window.__WOM_PLUGIN__` 挂一个窄接口
> （`register` / `log` / `fail`），由它转调这里；**挂的是接口，不是注册表本身**。
> 情形与本条的预期一致（那时才出现调用方），改写记在 [ADR-0011](./0011-js-plugins-load-by-url-not-eval.md)。

## 两个已知边界

- `action_icon` 是**全表**查：行里只带动作 id，不带它属于哪个插件，所以两个插件用同一个动作 id 配不同图标时会撞在一起。动作 id 跨插件并不保证唯一（`copy` / `open_url` 这类通用名尤其），等真有 JS 插件注册进来再谈要不要加插件维度。
- 注册表是**每个 webview 一份**：两个入口各有自己的模块实例，所以两个入口都得自己调一次 `register`（配置窗口将来要用图标或文案时也要）。

> 后续：第一条边界**到了要谈的时候**——真有 JS 插件注册进来了。它从"已知边界"变成
> "必须解决"：查找要加上插件维度，连带改 `PluginItemDisplay` 的形状。形状要等第二个插件才定得下来，
> 所以这一轮只记录、不实现，见 [ADR-0011](./0011-js-plugins-load-by-url-not-eval.md)。
