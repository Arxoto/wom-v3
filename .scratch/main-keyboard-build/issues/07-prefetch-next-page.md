# 07: 预请求下一页

**What to build:** 结果超过一页时，往下按到后段会静默续上下一页，指针与滚动位置都不受影响。

**Blocked by:** 02

**Status:** resolved

- [x] 每次 `↓` 判断：`Selection` 落在已加载列表的后 10 位，且已加载条数小于总数，就请求下一页（判据取的是移动前的 `Selection`，见 Comments）
- [x] 请求的起始位置是结果集里的起始位置（不是页码），传已加载条数
- [x] 新一页追加到已加载列表末尾，`Selection` 与滚动位置都不动
- [x] 同一页的重复请求被挡住；新检索到来时在飞的请求整体作废
- [x] 失败看得见：Rust 记 warn，前端把失败打到控制台（有账要收的捕获里自己打，没有要收的就不捕获、交给未处理的拒绝自动打），不向用户提示；指针还在区间内时下一次 `↓` 会再试
- [x] 余量写死 10，不跟随 `main_item_n`

依据：[spec.md](../main-keyboard/spec.md) §3 的「预请求」、[分页与滚动](../main-keyboard/issues/09-paging-and-scroll.md)。

## Comments

- 2026-09-19 核对代码（`src/main/interaction/search_session.ts` 的 `prefetch` 与 `useMainInteraction.ts` 的 `prefetch_next_page` / `select_next` 分支）：后四条成立（起始位置传已加载条数、纯追加不动 `Selection` 与滚动、在飞单槽去重与令牌作废、余量写死 10）。第 1 条与第 5 条不成立，先取消勾选：
  - `b7dc7c4`（精简 search_session）连同三阶段一起删掉了 `page_start >= ctx.total` 的守卫，`PrefetchContext.total` 现在没人读——列表已经加载到底时，`↓` 仍会每次发一次 `search_page`（后端返回空页，前端只是空追加）。
  - `deps.search_page(...)` 只有 `then`、没有 `catch`：请求失败时 `prefetch_in_flight` 停在 `true`，同一轮里再也发不出下一页；「失败后下一次 `↓` 会再试」没有落地。
  - 同一处还有一处与 spec 的表述差一行：判据用的是移动前的 `Selection`（`prefetch_next_page()` 在 `dispatch` 之前调用），实际触发点是已加载列表的后 9 位。
- 同日修订（依讨论）：`page_start >= ctx.total` 不是死代码——结果总数不超过一页时首屏就把全部结果取回（`loaded_count === total`），指针走到后 10 位后每次 `↓` 都在打空枪；还没有结论时 `loaded_count` 与 `total` 都是 0，同样会发一次。守卫已补回。
- 同日修订（依讨论）：「不重试」指不重发失败的那一次——预请求的失败回调只把在飞记账放开（先比令牌，落单的失败不能动新的一轮），下一次 `↓` 按同一判据重新发。失败都要在控制台留痕：有账要收的（预请求要放开在飞记账）在捕获里自己 `console.error`；没有账要收的（第一页检索，只是停在上一份结论上）干脆不捕获，交给未处理的拒绝自动打印。两处都不向用户提示，也不撤 `last_sent`。
