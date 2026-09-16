# 05: 动作表下发与行内动作

**What to build:** 每种 `ItemType` 支持哪些 `Item Action` 由 Rust 写死并下发；列表每行显示该条目默认动作的图标，Tail 显示当前条目的动作文案。没有动作的条目，动作栏整块不渲染。

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] 新增无参命令，返回 `ItemType` → 动作列表；顺序即优先级，第一个是默认动作
- [ ] 映射表按 spec §4.2 写死；动作带 ASCII 的 `id` 与 `label_key`
- [ ] `label_key` 按 `ItemType` + 动作分套（同一个 `copy` 在 `File` 与 `Web` 上文案不同）
- [ ] 不按平台剔除动作
- [ ] 前端挂载时拉一次动作表；类型镜像补齐
- [ ] 每行显示该行默认动作的图标，没有动作时动作栏整块不渲染
- [ ] Tail 显示当前 `ItemType` + 动作的中文描述，查不到键就显示键本身
- [ ] 本票据不引入切换动作的键位与菜单

依据：[spec.md](../main-keyboard/spec.md) §2 的 props 与新增表、§4.2，以及 [Item Action 映射表与元数据](../main-keyboard/issues/10-item-action-table-and-metadata.md)。
