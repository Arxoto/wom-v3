# 按键层与状态归属

Type: grilling
Status: resolved

## Question

按键响应挂在哪一层？「按键 → 意图」的映射放在哪？交互状态归谁？列表滚动位置算不算状态？

## Answer

- **拦截层**：`window` / `document` 级的单个 `keydown` 监听（捕获阶段）作为唯一入口；真实 `<input>` 始终聚焦，只负责收字符。预览打开、鼠标点过 body 之后按键仍然要生效，所以入口不能挂在那个 input 上。
- **意图映射**：独立的纯模块 `(key, state) → Intent | null`；事件回调只做「DOM 事件 → 参数 → dispatch」的接线；reducer 消费 Intent 改状态。判定「现在是什么状态、该不该拦」与「状态怎么变」分开。
- **状态归属**：抽一个交互 hook，收拢输入值、`Selection`、`Preview`、已加载列表、请求时序；`Head` / `Body` / `Tail` 退化成展示组件（`Head` 收值 + onChange，`Body` 收列表 + `Selection` + `Preview` 开关，`Tail` 收 `Preview` 开关）。`AppMain` 只剩组装。
- **滚动位置**：派生值，不占状态。`偏移 = clamp(Selection - (可见行数 - 2), 0, max(0, 已加载条数 - 可见行数))`，这样不存在「高亮与滚动位置各说各话」的漂移。
