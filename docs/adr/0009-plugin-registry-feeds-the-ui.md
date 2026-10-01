# 插件体系接入应用：界面改读插件注册表，内建体系整体保留

`plugin_framework` / `plugin_impl_launcher` 不再只是"编得进去的模块"：`plugin_host.rs` 用 Tauri 实现 `PluginContext`（剪贴板、打开路径/URL、在文件夹中选中、应用数据目录、路径变量解析），注册表在 `setup` 里托管进应用，`commands.rs` 加一组 `plugin_*` 命令，前端的检索、动作表与动作派发整体改走这一组。界面因此只显示 launcher manifest 的条目——`snip` / `note` 这类内建类型要等各自成为插件才会回来。

内建那一套（`builtin_plugins` 的类型、检索、动作表、命令，以及 `core.tsx` 里对应的封装与镜像类型）**一行不删**：两套并行是 ADR-0008 的决定，这一轮只换"谁给界面供数"，替换留给后面单独一轮。代价是过渡期里仓库有两套 Item、两套检索、两套命令，以及两套文案键（内建的三段式与 launcher 的四段式并存于 `action_labels.ts`）。

一个具体的接线取舍：动作表下发的是"类型名 → 有序动作数组"，而不是 `BTreeMap<ActionId, …>`。按 id 排序会把 launcher 的 `scan` 排成 `copy`、`open_path`、`reveal`，默认动作就错了；顺序是优先级，必须由注册顺序决定。
