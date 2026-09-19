# Map: 主窗口按键交互

Label: wayfinder:map
Status: resolved

## Destination

`.scratch/main-keyboard/spec.md`：一份「实现前不需要再决策」的 spec，定死主窗口按键响应的分层、状态归属、命令契约与滚动 / 预请求模型。

本 effort 只产决策，不写生产代码。

## Notes

- 仓库原则看 `AGENTS.md`：只写生产代码、测试留给用户；依赖单向流动（`commands` → `configs` / `window_utils` / `global_shortcut`）；`Config` 只有一份，派生值现算。
- 每个 session 都应带上：`codebase-design`（接缝与模块深度）、`grilling`（HITL 讨论）、`domain-modeling`（术语与 ADR）。
- 术语以 `CONTEXT.md` 为准：本次新增 `Selection` / `Preview` / `Item Action` / `Item Index`。
- 「翻页」是内部说法；用户可见的行为是「向下滚动」。
- 01–09 这几张 ticket 的答案在 charting 的 grilling 里已经问出来了。按「一个决策只住一处」的原则，把它们落成已解决的 ticket，map 只留索引与概要。
- 01–13 全部 resolved，`spec.md` 已定稿（`Status: settled`）；交互接缝写进 ADR-0006（`docs/adr/0006-window-level-keydown-entry.md`）。
- 目的地的 `spec.md` 已达成，本 effort 的决策到此为止。剩下唯一未完成的动作是**功能验收时实测 IME 合成期间的 keydown**（见 Not yet specified）——它是实现之后的验证，不是还没定的决策。

## Decisions so far

- [四类动作的插件覆盖](issues/02-research-item-action-apis.md)：四类动作都有官方路径；ACL 只拦前端 IPC，动作在 Rust 里执行时只需依赖 + 注册插件。
- [Destination 与范围](issues/03-destination-and-scope.md)：只产决策，spec 为终点；Rust 与前端都可改，四类动作的真实执行不在范围内。
- [按键层与状态归属](issues/04-key-layering-and-state-ownership.md)：window 级单个 keydown 入口 + 纯意图模块 + reducer + 一个交互 hook；滚动位置是 Selection 的派生值；只有 `Enter` / `Shift+Enter` / `ESC` 调 `preventDefault()`。
- [打字到检索的时序](issues/05-typing-search-and-ime.md)：合成期间所有键放行；50ms 尾防抖 + 有界环令牌丢弃过期响应；按住 ↑/↓ 限流；检索关键字取第一个空格之前的内容，输入清空到空串不检索并把结果置空。
- [Selection 与 Preview 的行为](issues/06-selection-and-preview.md)：新结果回到第一条、边界 clamp 不环绕；鼠标 hover 只保留 CSS 效果、不写 Selection；Preview 跟随 Selection，空结果自动关闭并给一行占位（输入为空时「输入关键字开始搜索」，有输入没匹配时「没有匹配的条目」）；打开时 input 用 `readOnly`，↑/↓ 照常移动 Selection，`←` / `→` 放行给光标。
- [Item Action 与触发命令](issues/07-item-action-and-command.md)：动作表在 Rust 写死、经元数据下发、带 ASCII 标签键；一条通用 `run_item_action` 命令；`←` / `→` 放行给光标，本 effort 不切动作。
- [退场与窗口模式](issues/08-dismiss-and-window-mode.md)：不再销毁窗口；`Always` / `HideAndShow` 只决定失焦与触发动作后是否隐藏；ESC 一律隐藏；每次显示先强制关掉 Preview 再聚焦全选 input。
- [分页与滚动](issues/09-paging-and-scroll.md)：预请求余量写死 10、Selection 落到已加载列表后 10 位就发（失败静默，Rust 记日志）；高亮停在可见区往下 60% 那一行、列表整体上移，下面没有更多结果才移到末行。
- [前端接缝的形状](issues/11-frontend-module-shape.md)：新增 `src/main/interaction/`（纯意图 / reducer / 接线 hook 三个文件）；hook 持有 input ref；副作用只在事件回调里做（绕开 StrictMode 双调用）；滚动偏移由 `Body` 现算；`ItemDisplay` 带 `item_index`，列表位置不是 `Item Index`；行内动作是图标（选中行跟着 ←/→ 换），详细文案在 Tail。
- [显示时的聚焦与全选](issues/12-focus-and-select-all-trigger.md)：Rust 在显示分支发事件，前端监听后聚焦并全选；挂载时再做一次，覆盖「启动即显示」。
- [滚动与翻页的术语](issues/13-glossary-scroll-and-paging.md)：滚动 / 翻页 / 预请求不进术语表；新增 `List Position`，与 `Item Index` 分开。
- [Item Action 映射表与元数据](issues/10-item-action-table-and-metadata.md)：命令 `fetch_item_type_actions` 返回 `ItemType` → 动作表；文案键按 `ItemType` + 动作分套；`System` 无动作、`Note` 只有占位的 `open_note`；不做平台剔除。
- [IME 合成期间的事件语义](issues/01-research-ime-composition.md)：lock（`compositionstart` / `compositionend`，只管 input 路径）；`compositionend` 自己补一次检索，靠「关键字没变不重发」去重；不做显示时清 lock 的兜底；keydown 暂不做合成判断，待实测。

## Not yet specified

<!-- 还没sharp到能开票的雾 -->

- 功能验收时一并测：合成期间 `Enter` / `↑` / `↓` / `ESC` 会不会到达 window 级 keydown、要不要补判据（含 WebKit 的尾随 229）。

## Out of scope

<!-- 划出目的地的范围之外的，永不毕业 -->

- 四类动作的真实平台执行，以及所需的插件与权限（另起 effort）。
- 鼠标点击选中条目。
- ghost 补全建议（Head 里那条假占位串）。
- 动作菜单 UI（侧边挤占 / 覆盖那一套）。
- 窗口销毁与重建（被丢弃的旧设计）。
- 复杂的视觉过渡动画（下翻的 transform / opacity、Accent 条的拉长回缩）。
