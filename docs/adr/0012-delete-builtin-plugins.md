# 删除 builtin_plugins：插件体系成为唯一的检索与动作路径

ADR-0008 让内建体系（`builtin_plugins`）与插件体系并行存在，ADR-0009 把界面切到 `plugin_*`
命令、内建那一套原样保留。此后前端又把动作表搬进自己的插件注册表（ADR-0010），内建的
`search` / `search_page` / `run_item_action` / `fetch_item_type_actions` /
`fetch_scan_base_options` 就再没有调用方：命令仍在 Rust 侧注册、`load_stat` 仍在 `setup` 里跑、
托盘重载仍会重读 `builtin_plugins.txt`，但加载出的 `BuiltinStat` 没有任何消费者——
整条内建数据流只剩副作用。

这一轮把 `builtin_plugins` 整个删除（模块根加 14 个子文件），连带它的命令与注册、`BuiltinStat`
运行时状态、只服务于它的 `SETTING_FILE_NAME` 常量，以及前端 `core.tsx` 里已无人使用的
`ScanBase` / `ScanBaseOption` 镜像类型。托盘菜单的插件重载此后只重读 launcher 的 manifest
（后来的 `re_packages` 那项是另一回事：它重扫整个 `Plugin Folder`，新包注册、消失的包移除，
见 `plugin_host::reload_packages`）。
**ADR-0008 / 0009 的"并行、稳定后再替换"到此结束，本 ADR 取代它们。**

## 没有任何已实现的行为随删除消失

内建体系里已经跑起来的那部分，每一项在插件侧都有接棒对象：检索与运行期状态在
`plugin_framework` 的 `PluginRegistry`，manifest 的解析与扫描在 `plugin_impl_launcher`，
动作的顺序、图标与文案在前端插件注册表。删除只是把已经没人走的那条旧路拆掉。

真正没有接棒对象、一起删掉的是三处占位或过期描述：

- `ItemType` 的 `note` / `snip` 两个类型，以及 `action.rs` 的 `open_note` 动作与 note 动作表；
- `action_systems.rs` 整个文件（系统命令的实现，一直是 TODO）；
- `persistence/scan_base.rs` 的十四项 `ScanBase` 枚举与配置页下拉。

前两件不是删功能，而是把位置让给将来的插件：笔记、片段、系统命令各自成为插件时再回来，
未实现的内容已经转移到 `src/AppMain.tsx` 的 todo list，不在 Rust 侧留半截类型。第三件的
"变量名 → 目录"只保留宿主那侧的 `PluginContext::resolve_base`，映射由 Tauri 负责；
配置页下拉这一轮不再提供。

## 代价

内建的检索算法测试与解析测试随模块一起删除（`search.rs` 五个、`parse_core.rs` 一个），
新框架目前没有对应的测试覆盖。`docs/adr/0008` / `0009` 保留原文记录当时的状态，由本 ADR 取代。
