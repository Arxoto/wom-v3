# 滚动与翻页的术语

Type: task
Status: resolved

## Question

「滚动 / 翻页 / 预请求」这组名字要不要进 `CONTEXT.md`，怎么定义才不会把实现细节塞进术语表？（已经定了的口径：翻页是内部说法，用户可见的行为是向下滚动。）

## Answer

- 「滚动 / 翻页 / 预请求」**不进** `CONTEXT.md`：滚动是通用交互词，翻页与预请求是实现细节（`PAGE_SIZE`、余量 10 都是代码里的常量），术语表只收有歧义的领域词。
- 本轮唯一值得进的是把「整集里的位置」与「结果列表里的第几行」分开：新增 `List Position`，Selection 落在已加载结果列表的第几行、滚动偏移是它的派生值，并写明它不是 `Item Index`。已写入 `CONTEXT.md`。
- 2026-09-19（依代码订正）：Answer 里「滚动偏移是它的派生值」不再成立——偏移是 `Body` 自己的状态，`List Position` 只说 `Selection` 落在已加载列表的第几行（见 [spec.md](../spec.md) §5）。`CONTEXT.md` 的 `List Position` 条目没写派生关系，不用改。
