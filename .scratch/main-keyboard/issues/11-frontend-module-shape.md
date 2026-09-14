# 前端接缝的形状

Type: grilling
Status: open

## Question

落到代码里的接缝长什么样：意图模块、reducer、交互 hook 各自的文件名与接口签名？谁持有 input 的 ref（聚焦与全选要用）？`Head` / `Body` / `Tail` 与 `Item` 的 props 形状分别是什么？`core.tsx` 里要新增哪些类型镜像与 invoke 封装？

## Answer

待解决。已定的只是形状的**模式**（见 [按键层与状态归属](04-key-layering-and-state-ownership.md)），这一票把它钉成具体文件与签名。

## Comments

- 一份**未经确认的草案**（此前由一名 agent 提出，直接记在这里免得丢）：新增 `src/main/interaction/` 三个文件——`keys.ts`（纯函数 `resolve_key(key, mod, ctx) → Intent | null`，`Intent` 为 `select_prev / select_next / action_prev / action_next / run_action / toggle_preview / dismiss`，`ctx` 只带 `{ composing, preview_open }`）、`reducer.ts`（`MainState { input, item_list, total, selection, action_index, preview_open }` + `reduce(state, action)`，动作来源只有 `typing` / `page_loaded` / `intent` 三类）、`useMainInteraction.ts`（唯一的接线层：window 级 keydown、50ms 防抖、请求令牌、预请求、三个 invoke）。`Head` 收 `{ value, on_change, input_ref }`，`Body` 收 `{ item_list, selection, item_n, show_preview, action_labels }`，`Item` 收 `{ item, action_label, is_selected }`。草案里还提了一条不变量：**前端列表下标就是 `Item Index`**（只要分页按 index 顺序追加不跳页，`selection` 既是列表位置也是寻址参数，不需要第二套映射）。

- 该草案附带四个待拍的点：① `core.tsx` 里 `ItemActionId` 用字符串联合还是 `string`；② 三个文件放新目录 `src/main/interaction/` 还是散在 `src/main/`；③ 可见窗口由 `Body` 现算还是 hook 算好 `visible_items` 传下去；④ input 的 `ref` 归 hook 还是归 `Head`。
