# 插件框架与 builtin_plugins 并行存在，稳定后再替换

`builtin_plugins` 以 ItemType 为轴组织（解析、扫描、动作表、运行期状态都挂在这个轴上），
要改成插件体系就得把轴本身换掉。我们**不原地改造**它，而是在 `src-tauri/src/plugin_framework/`
（框架层）与 `src-tauri/src/plugin_impl_launcher/`（第一个插件）里另起一套，
不接 Tauri 托管状态、不注册命令、不接线前端，等稳定后再另起一轮规划完成替换。

理由是两道问题不该同时出现：原地改造会让每一次提交都同时面对"新设计对不对"与
"旧行为有没有被破坏"。并行存在期间仓库里会有两套 Item 与两套检索，这是**有意**的，
不是没清理干净——`plugin_impl_launcher` 因此连 `builtin_plugins` 的一个类型都不复用
（包括那个六变体的 `ItemType`，哪怕 launcher 只需要其中四个）。

> 后续：内建体系已整体删除，并行期结束，见 [ADR-0012](./0012-delete-builtin-plugins.md)。
