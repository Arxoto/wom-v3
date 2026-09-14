# Map: 主窗口按键交互

Label: wayfinder:map
Status: open

## Destination

`.scratch/main-keyboard/spec.md`：一份「实现前不需要再决策」的 spec，定死主窗口按键响应的分层、状态归属、命令契约与滚动 / 预请求模型。

本 effort 只产决策，不写生产代码。

## Notes

- 仓库原则看 `AGENTS.md`：只写生产代码、测试留给用户；依赖单向流动（`commands` → `configs` / `window_utils` / `global_shortcut`）；`Config` 只有一份，派生值现算。
- 每个 session 都应带上：`codebase-design`（接缝与模块深度）、`grilling`（HITL 讨论）、`domain-modeling`（术语与 ADR）。
- 术语以 `CONTEXT.md` 为准：本次新增 `Selection` / `Preview` / `Item Action` / `Item Index`。
- 「翻页」是内部说法；用户可见的行为是「向下滚动」。
- 01–09 这几张 ticket 的答案在 charting 的 grilling 里已经问出来了。按「一个决策只住一处」的原则，把它们落成已解决的 ticket，map 只留索引与概要。

## Decisions so far

- [四类动作的插件覆盖](issues/02-research-item-action-apis.md)：四类动作都有官方路径；ACL 只拦前端 IPC，动作在 Rust 里执行时只需依赖 + 注册插件。
- [Destination 与范围](issues/03-destination-and-scope.md)：只产决策，spec 为终点；Rust 与前端都可改，四类动作的真实执行不在范围内。
- [按键层与状态归属](issues/04-key-layering-and-state-ownership.md)：window 级单个 keydown 入口 + 纯意图模块 + reducer + 一个交互 hook；滚动位置是 Selection 的派生值。
- [打字到检索的时序](issues/05-typing-search-and-ime.md)：合成期间所有键放行；50ms 尾防抖 + 有界环令牌丢弃过期响应；按住 ↑/↓ 限流。
- [Selection 与 Preview 的行为](issues/06-selection-and-preview.md)：新结果回到第一条、边界 clamp 不环绕；Preview 跟随 Selection，空结果自动关闭。
- [Item Action 与触发命令](issues/07-item-action-and-command.md)：动作表在 Rust 写死、经元数据下发、带 ASCII 标签键；一条通用 `run_item_action` 命令。
- [退场与窗口模式](issues/08-dismiss-and-window-mode.md)：不再销毁窗口；`Always` / `HideAndShow` 只决定失焦与触发动作后是否隐藏；ESC 一律隐藏。
- [分页与滚动](issues/09-paging-and-scroll.md)：预请求余量写死 10；高亮停在倒数第二行、列表整体上移，下面没有更多结果才移到末行。

## Not yet specified

<!-- 还没sharp到能开票的雾 -->

- 本 effort 的决策里有没有够格写 ADR 的（等决策集齐再判断）。
- 检索结果为空时主窗口的表现：是否给占位、Tail 的提示要不要变（目前只定了 Preview 关闭）。
- 预请求失败或长时间无响应时，用户可见的反馈是什么。

## Out of scope

<!-- 划出目的地的范围之外的，永不毕业 -->

- 四类动作的真实平台执行，以及所需的插件与权限（另起 effort）。
- 鼠标点击选中条目。
- ghost 补全建议（Head 里那条假占位串）。
- 动作菜单 UI（侧边挤占 / 覆盖那一套）。
- 窗口销毁与重建（被丢弃的旧设计）。
- 复杂的视觉过渡动画（下翻的 transform / opacity、Accent 条的拉长回缩）。
