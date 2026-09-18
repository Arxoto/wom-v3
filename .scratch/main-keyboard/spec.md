# Spec: 主窗口按键交互

Label: wayfinder:spec
Status: settled

1–13 号 ticket 的决策拼成的说明书：按它写代码时不需要再做交互层面的决策。本 effort 只产这份文档，不写生产代码。

动手前按 `AGENTS.md` 读被改模块顶部的文档注释；术语以 `CONTEXT.md` 为准（本 effort 新增 `Selection` / `Preview` / `Item Action` / `Item Index` / `List Position`）。

## 0. 范围

- **覆盖**：按键 → `Selection` / `Preview` / 退场的整条路；动作的表示、下发与派发契约；检索、预请求、滚动的时序模型。Rust 与前端两侧都可以改。
- **Rust 侧最小改动**：`ItemDisplay` 带上 `Item Index`；动作表按 `ItemType` 写死并经元数据下发；新增派发动作与派发退场的命令；显示窗口时发一条事件。
- **不覆盖**：四类动作的真实平台执行（只保留 `copy` 把链路打通，见 §4.3）、鼠标点选、ghost 补全、动作菜单 UI、窗口销毁重建、视觉过渡动画。

## 1. 按键到意图

**唯一入口**：`window` 捕获阶段的单个 `keydown` 监听。真实 `<input>` 只负责收字符，不参与按键判定——预览打开、鼠标点过 body 之后按键仍要生效，入口不能挂在 input 上。

| 键 | 列表模式（Preview 关） | 预览打开 |
| --- | --- | --- |
| `↑` / `↓` | `select_prev` / `select_next` | `select_prev` / `select_next`（`Preview` 跟随 `Selection`） |
| `Enter` | 跑当前动作（默认动作） | 与列表模式一致 |
| `Shift+Enter` | 打开 `Preview` | 关闭 `Preview` |
| `ESC` | 隐藏主窗口 | 关闭 `Preview`（不隐藏），再按一次隐藏 |
| 其他（含 `←` / `→`） | 放行 | 放行 |

- `select_prev` / `select_next`：`Selection` 移动一行，边界 clamp 不环绕；两种模式都生效——`Preview` 跟随 `Selection`，预览开着时上下键直接换条目；按住连发限流 100ms（写成常量），首次按键立即响应。
- **`←` / `→` 不拦**：没法判断用户想移动光标还是换动作，所以这两个键完全放行——不产生意图、不 `preventDefault()`，光标照常移动。代价是本 effort 没有「切换 `Item Action`」的键位，每个条目只用它的默认动作（动作表第一个）；切换动作留给将来的动作菜单那条线。
- `run_action`：跑当前条目的默认 `Item Action`；没有可执行的动作（条目没有动作、或结果为空）则整个流程都不发生——不关 `Preview`、不隐藏窗口。
- `dismiss`：无条件隐藏主窗口（`ESC` 那条显式意图，见 §4.4）。
- `toggle_preview`：`Shift+Enter` 是开关。`Preview` 跟随 `Selection`，结果为空时自动关闭。
- **`preventDefault()` 只用在三个键上**：`Enter`、`Shift+Enter`、`ESC`。其余键保留浏览器默认行为——`↑` / `↓` 会把光标顶到首尾、`←` / `→` 会移动光标，都照旧。
- 未识别的键不产生意图、不 `preventDefault()`。
- **合成期间（IME 候选框打开）暂不做任何合成判断**，见 §6。

## 2. 状态与前端接缝

新增目录 `src/main/interaction/`（`src/main/item/` 已有先例）：

| 文件 | 职责 |
| --- | --- |
| `keys.ts` | 纯函数 `resolve_key(key, mod, ctx) → Intent \| null`；`Intent` 为 `select_prev / select_next / run_action / toggle_preview / dismiss`，`ctx` 只带 `{ preview_open }`（只有 `ESC` 要判它；合成判据待实测，见 §6） |
| `reducer.ts` | `MainState` + 纯函数 `reduce(state, action)`；动作来源只有 `typing` / `page_loaded` / `intent` 三类 |
| `search_session.ts` | 检索会话的时序：50ms 尾防抖、请求令牌、在飞预请求的记账。非 React 模块，两个 invoke 与两个派发回调由接线层注入，判据（已加载条数 / `Selection` / `total`）也从外面传 |
| `useMainInteraction.ts` | 唯一 React 接线层：window 级 keydown、合成事件、显示重置、动作 / 退场 / 检索三类 invoke 与聚焦 |

`MainState`（渲染用得上的都在这里，请求时序状态留在 hook 内部）：

| 字段 | 含义 |
| --- | --- |
| `input` | 输入框的值（受控） |
| `item_list` | 已加载的结果，分页追加 |
| `total` | 结果总数，用来判断还有没有下一页 |
| `selection` | `List Position`，即已加载列表里的行号 |
| `preview_open` | `Preview` 是否打开 |

不变量：

- **列表位置不是 `Item Index`**：`selection` 只表示行号；寻址条目一律用 `ItemDisplay.item_index`。
- **副作用不进 reducer**，也不放 effect：`index_main.tsx` 挂着 `React.StrictMode`，dev 下 reducer / state updater 会被调用两次，一次按键就会打两枪。
- **滚动位置是派生值**，不占状态（见 §5）。
- **空结果**：`item_list` 为空时，所有依赖条目的意图退化成无操作（判据是列表长度，不是 `selection` 的取值）；列表区显示一行占位文案「没有匹配的条目」，Tail 保持列表模式那套提示（见 [Selection 与 Preview 的行为](issues/06-selection-and-preview.md)）。
- **新结果回来**：`selection` 回到 0；当前动作恒为该 `ItemType` 动作表的第一个，不需要额外的状态。

组件 props：

| 组件 | props |
| --- | --- |
| `Head` | `{ value, ghost, on_change, input_ref }`（ref 归 hook，聚焦与全选要用；`ghost` 沿用现状的占位串，本 effort 不实现补全） |
| `Body` | `{ item_list, selection, item_n, show_preview, type_actions }`（滚动偏移自己现算） |
| `Item` | `{ item, action_id, is_selected }`（图标按 `action_id` 查；没有动作时传 `null`，整块不渲染） |
| `Tail` | `{ preview_open, action_desc }`（当前 `ItemType` + 动作对应的文案，先用动作名占位；条目没有动作时为 `null`，动作栏整块不渲染） |

2026-09-18 起，前端文件形状在交互接缝之外多了一层：`Tail` 与 `Head` / `Body` 同层，只管外壳布局（`Tail.css` 只剩 `.tail-box`），也不出现任何图标与顺序；形态由 `preview_open` 现算（不再由 `AppMain` 传枚举进来），`Tail.tsx` 按 `HintBarKind` 从一张说辞表里取 `esc_label` / `preview_toggle_label` 交给 `src/main/hint_bar/HintBar.tsx`——两套形态的骨架并成这一份，因为图标顺序本来就一致、差异只有文案。子组件（`SelectionHint` / `ActionHint` / `ActionName` / `PreviewToggleHint` / `EscHint` / `KeyEsc`）也在 `HintBar.tsx`，提示条自己的按键图标合在 `hint_icon.tsx`（类名 `.hint-icon`、`.hint-icon-enter`，尺寸在 `HintBar.css`）。列表行的动作图标不跟过来：它随消费者留在 `item/action_icons.tsx`（类名 `.item-action-icon`，尺寸在 `Item.css`）。

提示条的内容（同一天定）：左组是 `ESC → ↑ ↓ 🖱 选中条目`，ESC 出口两套形态都有，文案由形态给——列表模式「关闭界面」（对应 `dismiss`，隐藏主窗口）、预览模式「关闭预览」（只关预览）；右组是 `↵ 触发动作` 与 `⇧↵ 打开/关闭预览`，具体动作名不进右组，绝对定位在 tail 正中（`.hint-action-name`）。

动作名的归属（同一天定）：它由 `Tail` 直接渲染在 `.tail-box` 里，不进 `HintBar`——`HintBar` 收的只有 `esc_label` / `preview_toggle_label` / `has_action`，都是形态级输入，再配 `memo`，上下切条目时提示条整个跳过重渲染；动作名自己随 `Selection` 每次上下重渲染，那是它该做的事。`has_action` 只管「↵ 触发动作」显不显示（没有动作的条目连它一起不渲染）。

同一轮里动作名换了位置：具体动作名（「复制命令」等）绝对定位在 tail 正中（`.hint-action-name`，父元素 50% + 自身 -50%），不参与左右两组的分宽；右侧那栏只说「触发动作」，不再跟着当前动作变。

新增的前端表（放哪个文件是实现细节，两处都只放数据）：

- **图标表**：`ItemActionId` → svg。行内动作是图标不是文字，每行传该行的默认动作（动作表第一个）。建议放 `src/main/item/action_icons.tsx`。
- **文案字典**：`ItemType` + `ItemActionId` → 中文描述（同一个 `copy` 在 `File` 上是「复制完整路径」、在 `Web` 上是「复制链接」）。查不到键就显示键本身。建议放 `src/main/interaction/action_labels.ts`。

`src/core.tsx` 的镜像与封装：

```ts
type ItemType = "snip" | "sys" | "note" | "cmd" | "web" | "file" | "scan";
type ItemActionId = "copy" | "open_url" | "open_path" | "reveal" | "open_note";
interface ItemAction { id: ItemActionId, label_key: string }
type ItemTypeActions = Record<ItemType, ItemAction[]>;   // System 是空数组
interface ItemDisplay { the_type: ItemType, name: string, desc: string, item_index: number }

get_item_type_actions() → invoke("fetch_item_type_actions")
run_item_action(item_index, action)   // invoke 时载荷是 { itemIndex, action }
dismiss_main_window()
```

**命令参数名**：按 tauri 的默认约定——Rust 侧 `snake_case`，JS 侧 lowerCamelCase（tauri-macros 会做这个转换，见 `wrapper.rs:50` 与 `:461`，版本 2.5.5）。也就是 `item_index` 在 invoke 的载荷里叫 `itemIndex`，不给命令加 `rename_all`；前端封装的形参沿仓库的镜像写法用 `item_index`（`core.tsx` 的其余镜像字段也是这个风格），只有载荷键跟着 tauri 转。

## 3. 时序

**打字 → 检索**

- 变更后 50ms 尾防抖（`compositionend` 补的那次也走这里）；文本与上次发送相同就不重发（后端另有 `input_key` 缓存兜底）。
- 请求令牌只与「当前最新令牌」比相等，不比较大小；用有界环计数器（`% 256`，远大于同时在飞的请求数）。过期响应整包丢弃。
- `compositionstart` / `compositionend` 维护 lock：合成期间 input 路径不防抖、不检索；`compositionend` 自己补一次检索（读 input 当前值），同样进 50ms 防抖——连续提交候选（每次都是一份新文本）只在停手后发一次，不绕过防抖；靠上面那条「文本没变不重发」去重，所以 `compositionend` 与提交后那次 input 谁先谁后都得到同一结果。
- 不做「窗口显示时清 lock」的兜底（判为罕见情形，见 §6）。

**预请求**

- 每次 `↓` 移动之后判断：`selection >= 已加载条数 - 10`（余量写死 10，不跟随 `main_item_n`）且 `已加载条数 < total`，满足就请求下一页。
- `search_page` 的 `index` 参数是**结果集里的起始位置**，不是页码；预请求传已加载条数。
- 预请求是纯异步的列表追加：只影响 `item_list`，`Selection` 与滚动位置不因它的成败改变；失败静默，Rust 记 warn，前端不打日志也不提示；只要指针还在后 10 位，下一次 `↓` 会再试。
- 同一时间最多一页在飞（起始下标恒等于已加载条数，而它只在上一页落账之后才增长），所以用单槽记账就够：挡同一页的重复请求，新检索到来时清掉这一笔（同一套令牌规则）。

**显示 / 退场**

- 每次显示窗口：先强制关掉 `Preview`（唤出等于重新开始），再让 input 聚焦并全选内容；输入内容与已加载的结果都保留。
- 聚焦与全选由 Rust 的事件驱动（见 §4.5），前端挂载时也做一次，覆盖「启动即显示」时事件早于前端就绪的情况。
- `ESC` 一律隐藏，与 `main_window_mode` 无关——它是显式意图；配置描述的是失焦、触发动作这两条被动隐藏规则。
- 触发动作后：前端先关掉 `Preview`（`Selection` 与输入内容都不动），再跑动作；要不要隐藏窗口由 Rust 在动作里按 `main_window_mode` 决定。

## 4. Rust 侧契约

### 4.1 `ItemDisplay` 带 `Item Index`

`search::ItemDisplay` 增加 `item_index: usize`，值是该条目在设置文件加载出的整集里的下标（`ItemCollection::item_list` 的位置，就是 `CONTEXT.md` 的 `Item Index`）。前端的行号只是 `List Position`，不用来寻址。

### 4.2 `fetch_item_type_actions`

无入参，返回 `ItemType` 的字符串形式 → 该类型支持的动作列表（顺序即优先级，第一个是默认动作）：

| `ItemType` | 动作 | 依据 |
| --- | --- | --- |
| `snip` | `copy` | 注释「片段 仅允许复制」 |
| `sys` | 无（空数组） | 不挂动作 |
| `note` | `open_note` | 占位，不具体实现 |
| `cmd` | `copy` | 注释「可复制并自动打开终端、后台执行」，执行不在本 effort |
| `web` | `open_url`, `copy` | 注释「支持使用默认浏览器打开、复制连接」 |
| `file` | `open_path`, `reveal`, `copy` | 注释「支持默认方式打开、在文件夹中选中、复制完整路径」 |
| `scan` | `open_path`, `reveal`, `copy` | 与 File 同类行为 |

- 每条动作带 ASCII 的 `id` 与 `label_key`；`label_key` 按 `ItemType` + 动作分套（`action.file.copy`），中文一个字都不进 Rust。
- 不按平台剔除动作：桌面三平台上四类动作都有官方路径（见 [`research/item-action-apis.md`](research/item-action-apis.md)）；真遇到某个平台缺 API 时再说。
- 前端只在挂载时拉一次；配置重载会重建窗口，不需要热更新。

### 4.3 `run_item_action`

`run_item_action({ itemIndex, action })`：在 Rust 里按 `item_index` 取出条目、执行动作，随后按 `main_window_mode` 决定要不要隐藏窗口（`HideAndShow` 隐藏）。越界索引或未知动作当无操作并记 warn——不执行、也不隐藏，不 panic、不做版本校验（沿用仓库「开发期一律破坏性处理」的态度）；前端不提示失败。

**本 effort 只打通 `copy` 一条动作**（`snip` 条目的默认动作），其余动作只出现在表里。它需要新增 `tauri-plugin-clipboard-manager` 依赖并注册插件；`snip` 的内容不依赖真实文件系统的对象，验证链路时最省事。

### 4.4 `dismiss_main_window`

无参、**无条件隐藏**。它就是 `ESC` 那条显式意图，与 `main_window_mode` 无关。

`main_window_mode` 只管两条被动隐藏规则，判定点都在 Rust：失焦时隐藏（`HideAndShow`，已有实现），触发动作后隐藏（`HideAndShow`，由 §4.3 的命令在派发完动作之后自己处理，前端不参与判断）。

### 4.5 显示事件

`window_utils` 在显示分支（`show_main_window` / `show_hide_main_window` 的显示路径）向主窗口发一条事件（暂名 `main_shown`，无载荷）；前端监听它做「关 `Preview` + 聚焦全选」。创建窗口时事件可能早于前端就绪，那一步由前端挂载时兜底。

## 5. 滚动模型

可见行数 = `main_item_n`；滚动偏移是 `Selection` 的派生值，不占状态：

```text
offset = clamp(selection - (item_n - 2), 0, max(0, loaded_len - item_n))
```

效果就是 09 号 ticket 定的：高亮走到倒数第二行时再按 `↓`，列表整体上移一行、高亮停在同一屏幕行；只有下面确实没有更多结果时，高亮才继续下移到可见区最后一行。`↑` 对称。本 effort 只做最简单的滚动，不含过渡动画。

## 6. 已知待实测

**合成期间的 keydown**：候选框打开时 `Enter` / `↑` / `↓` / `←` / `→` / `ESC` 会不会到达页面，先按「不影响」处理——keydown 路径不做任何合成判断。实现时要在 keydown 入口留一条注释说明这个已知风险（`Enter` 会跑动作、`↑` / `↓` 会移动 `Selection`、`ESC` 会关预览或隐藏窗口），功能验收时一并确认：真到了页面且能可靠判出合成，再补最小判据（`isComposing`，必要时补 `keyCode === 229`）。结论接回 [01 号 ticket](issues/01-research-ime-composition.md)。

**lock 卡住的残余面**：如果 `compositionend` 没来（合成被放弃），lock 会停在「合成中」，检索停摆。判为罕见、不加兜底；注意纯英文输入与粘贴不发 composition 事件，所以那种情况下不会自愈。

**WebKit 尾随 229**：`compositionend` 之后是否补一个 `keyCode === 229`、`isComposing === false` 的 keydown，需要在 Mac 上验；真出现就把它补进判据。

## 7. 决策索引

| 票据 | 内容 |
| --- | --- |
| [01](issues/01-research-ime-composition.md) | IME 合成（待实测） |
| [02](issues/02-research-item-action-apis.md) | 四类动作的插件覆盖 |
| [03](issues/03-destination-and-scope.md) | 终点与范围 |
| [04](issues/04-key-layering-and-state-ownership.md) | 按键层与状态归属 |
| [05](issues/05-typing-search-and-ime.md) | 打字到检索的时序 |
| [06](issues/06-selection-and-preview.md) | `Selection` 与 `Preview` 的行为 |
| [07](issues/07-item-action-and-command.md) | `Item Action` 与触发命令 |
| [08](issues/08-dismiss-and-window-mode.md) | 退场与窗口模式 |
| [09](issues/09-paging-and-scroll.md) | 分页与滚动 |
| [10](issues/10-item-action-table-and-metadata.md) | 动作表与元数据 |
| [11](issues/11-frontend-module-shape.md) | 前端接缝的形状 |
| [12](issues/12-focus-and-select-all-trigger.md) | 显示时的聚焦与全选 |
| [13](issues/13-glossary-scroll-and-paging.md) | 术语 |
