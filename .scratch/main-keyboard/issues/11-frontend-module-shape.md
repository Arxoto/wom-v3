# 前端接缝的形状

Type: grilling
Status: resolved

## Question

落到代码里的接缝长什么样：意图模块、reducer、交互 hook 各自的文件名与接口签名？谁持有 input 的 ref（聚焦与全选要用）？`Head` / `Body` / `Tail` 与 `Item` 的 props 形状分别是什么？`core.tsx` 里要新增哪些类型镜像与 invoke 封装？

## Answer

落定（2026-09-14），草案里的四个待拍点按下面执行：

- **文件**：新增 `src/main/interaction/`（`src/main/item/` 已有先例）——`keys.ts` 纯函数 `resolve_key(key, mod, ctx) → Intent | null`；`reducer.ts`；`useMainInteraction.ts` 唯一的接线层（window 级 keydown、50ms 防抖、请求令牌、预请求、动作 / 退场 / 检索三类 invoke）。
- `Intent`：`select_prev / select_next / run_action / toggle_preview / dismiss`；`ctx` 只带 `{ preview_open }`（合成判据待实测，见 [IME 合成期间的事件语义](01-research-ime-composition.md)）。
- `MainState { input, item_list, total, selection, preview_open }`；动作来源只有 `typing / page_loaded / intent` 三类。
- **列表位置不是 `Item Index`**：`ItemDisplay` 带 `item_index`（设置文件整集里的下标，`CONTEXT.md` 的 `Item Index` 条目与 [Destination 与范围](03-destination-and-scope.md) 都已这么写），`selection` 只是已加载列表里的行号。草案里那条「前端列表下标就是 `Item Index`」的不变量作废。
- **副作用只在事件回调里做**，不进 reducer、不放 effect：`index_main.tsx` 挂着 `React.StrictMode`，dev 下 reducer / state updater 会被调用两次（见 [Item Action 与触发命令](07-item-action-and-command.md) 的评论）。
- **可见窗口由 `Body` 现算**：hook 不产出 `visible_items`，滚动偏移是 `Selection` 的派生值（见 [按键层与状态归属](04-key-layering-and-state-ownership.md)）。
- **input 的 `ref` 归 hook**，`Head` 只收 `ref` 与值。
- 类型镜像：`core.tsx` 新增 `ItemType`（联合）、`ItemActionId`（`"copy" | "open_url" | "open_path" | "reveal" | "open_note"` 的字符串联合）、`ItemAction { id, label_key }`、`ItemTypeActions`（`Record<ItemType, ItemAction[]>`，`System` 是空数组）、`get_item_type_actions()`、`run_item_action(item_index, action)`、`dismiss_main_window()`；`ItemDisplay` 加 `item_index: number`。

props：

- `Head { value, ghost, on_change, input_ref }`
- `Body { item_list, selection, item_n, show_preview, type_actions }`
- `Item { item, action_id, is_selected }`（图标由 Item 按 `action_id` 查一张前端 svg 表；没有动作时传 `null`，整块不渲染。每行传该行的默认动作）
- `Tail { preview_open, action_desc }`（`action_desc` 是当前 `ItemType` + 动作解析出来的文案，先用动作名占位；条目没有动作时为 `null`，动作栏整块不渲染）

## Comments

- 2026-09-18：检索的时序从接线层拆到 `search_session.ts`——令牌 / 去重 / 在飞预请求 / 尾防抖互相咬合（令牌一前进就要清空在飞页，响应又要拿令牌判过期），散在 hook 的各个闭包里会变成一份看不见的共享状态。`useMainInteraction.ts` 仍是唯一 React 接线层，对外接口不变（见 [spec.md](../main-keyboard/spec.md) §2）。
- 2026-09-18：`Tail` 的 props 从 `hint: TailHint` 改成 `preview_open: boolean`——形态是 `preview_open` 的派生值，在派发点现算；提示条内容拆到 `src/main/hint_bar/`，图标收进 `src/main/icons/`（见 [spec.md](../main-keyboard/spec.md) §2）。
- 更早的草案（`action_prev` / `action_next` 切动作、`action_index` 状态、`action_labels` / `action_label` 这组 props、「前端列表下标就是 `Item Index`」那条不变量）已被上面的 Answer 逐条取代，细节不再保留。当时附带的四个待拍点——`ItemActionId` 用字符串联合、三个文件放新目录、可见窗口由 `Body` 现算、`ref` 归 hook——都按上面的 Answer 执行。
- 2026-09-14 讨论后改的两处：`label_key` 改成按 `ItemType` + 动作分套（见 [Item Action 映射表与元数据](10-item-action-table-and-metadata.md)），行内动作从文字改成图标、详细解释挪到 Tail。
- 2026-09-14 收口：行内图标固定是该行的默认动作（`←` / `→` 已放行，见 [Item Action 与触发命令](07-item-action-and-command.md)），无动作的条目动作栏整块留空。
