# 07: 预请求下一页

**What to build:** 结果超过一页时，往下按到后段会静默续上下一页，指针与滚动位置都不受影响。

**Blocked by:** 02

**Status:** resolved

- [x] 每次 `↓` 移动后判断：`Selection` 落在已加载列表的后 10 位，且已加载条数小于总数，就请求下一页
- [x] 请求的起始位置是结果集里的起始位置（不是页码），传已加载条数
- [x] 新一页追加到已加载列表末尾，`Selection` 与滚动位置都不动
- [x] 同一页的重复请求被挡住；新检索到来时在飞的请求整体作废
- [x] 失败静默：Rust 记 warn，前端不打日志也不提示；指针还在区间内时下一次 `↓` 会再试
- [x] 余量写死 10，不跟随 `main_item_n`

依据：[spec.md](../main-keyboard/spec.md) §3 的「预请求」、[分页与滚动](../main-keyboard/issues/09-paging-and-scroll.md)。
