# Destination 与范围

Type: grilling
Status: resolved

## Question

这次 effort 的终点是什么形态？覆盖到哪里为止？能不能动 Rust 侧？术语与产物放哪？

## Answer

- **终点**：只产决策。结束时 `.scratch/main-keyboard/spec.md` 是一份「实现前不需要再决策」的 spec；本 effort 不写生产代码。
- **范围**：只覆盖「按键 → Selection / Preview / 退场」这条路，加上动作的表示、下发与派发契约。四类动作的真实执行、所需插件与权限不在范围内，另起 effort。范围里保留一条能验证链路的最小动作即可。
- **改动边界**：Rust 与前端都可以改。Rust 侧的最小改动是：搜索结果给每个 Item 带上 `Item Index` 与 `ItemType`；动作映射在 Rust 写死并经元数据下发；新增一条派发动作的命令。
- **术语**：`Selection` / `Preview` / `Item Action` / `Item Index` 已写进 `CONTEXT.md`。
- **产物位置**：`.scratch/main-keyboard/`。
