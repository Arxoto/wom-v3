# 06 — `Plugin Search Page`：第二份结果页与 `Item Handle` 寻址

Status: resolved
Category: enhancement

## 目标

插件条目的动作触发一次 `Plugin Search`，把结果装进一份**新的 `item_list`** 显示出来，
并且结果行的动作能被派发。

## 范围

### Rust

1. `plugin_open_plugin_search(plugin_id, keyword)`：确保插件已装载（走 issue 05 的时序），
   调插件的搜索函数，拿回结果行，走**既有的投影**下发（框架不新增投影路径）。
2. `plugin_report_search_results`：插件执行完搜索后把结果行回传。
3. **寻址改用 `Item Handle`**（spec §3.4）：`ItemHandle` 下发到前端
   （`PluginItemDisplay` 带上它，**主列表那一层也一并带上**）。
   动作派发改用 handle；`Item Index` 仍然供主列表翻页与缓存，**两者并存**。
4. 代理的 `run_action` 真的派发到插件（issue 03 里它只记日志）。

### 前端

5. reducer 加一份与 `conclusion` 并列的状态：`plugin_search: { plugin_id, item_list,
   selection, action_index } | null`。**主列表的 `conclusion` 在结果页打开期间保持不变。**
6. 插件条目的 `Enter` 与 `⇧+Enter` **都**进入结果页（Q21）；非插件条目的 `Preview` 行为**不变**。
7. `ESC` 与退格都返回主列表；返回后 `Selection` 回到那行插件条目，结果页的选中位置**不保留**（Q24）。
8. 结果页为空（没搜到 / 装载失败 / 抛错）显示成**空页**，不弹错误。

## 文档改动（同 PR）

- `CONTEXT.md` 的 `Preview` 定义**放宽**为"显示当前项延伸内容的侧栏"（spec §4.1）。
- 记一条"必须显式重开 ADR-0010:23 的 `action_icon` 全表查"（spec §六）——**只记录，不实现**。

## Blocked by

03（代理要能派发）、05（装载器与注册）。
与 05 可并行开工（接口形状在 spec 里已定），集成时收敛。

## 验收

spec §八 的端到端 1–4 条。

## Answer

### Rust

1. `plugin_open_plugin_search(plugin_id, keyword)`（`plugin_host::open_plugin_search`，async）：
   查包 → 登记一个回程通道 → 发事件 `plugin_search_request`（带入口绝对路径与关键字）→
   `recv().await` → 投影成 `ItemSearchPage`。
   **去程与回程走的是事件 + `plugin_report_search_results` 命令**，不是"Rust 直接调 JS"：
   Tauri 没有 Rust → JS 的直接调用面，事件是它这一侧唯一的路。一次搜索只往返一次。
2. `plugin_report_search_results`：把行交给等着的 `recv()`；没有在飞的请求（用户已经退出
   搜索页）就记 warn 丢掉。
3. **寻址改用 `Item Handle`**：`ItemHandle` 现在 `Serialize` / `Deserialize`，
   `PluginItemDisplay` 带上 `handle`（两条投影路径都带：框架的 `project` 与宿主的结果页投影）。
   命令 `plugin_run_item_action` 的参数从 `item_index` 换成 `handle`，`PluginRegistry::run_action`
   也按 handle 查块与行。`item_index` 仍在，只服务主列表的翻页与预请求。
4. 代理的 `run_action` 真的派发：发 `plugin_action_request` 事件（带插件 id、行下标、动作 id）
   → 前端从自己报上去的那一份里取回那一行，调插件的 `run(row, action_id)`。
   **这条路是单向的**（`Plugin::run_action` 是同步的，等不了 webview），
   所以发出去就算 `Done`；插件的失败由它自己记日志，这一轮不加"动作失败原因"这条回程（Q6）。

   为了走到这一步，框架补了一处（实现中发现的结构性问题）：**结果行不在注册块里**，
   框架按 handle 找条目时会落空。所以 `Plugin` trait 多了一个带默认实现的
   `resolve_item(&handle) -> Option<PluginItem>`——框架只在自己手上找不到时才问插件，
   注册过的条目照旧走原来的路（launcher 不受影响）。结果行的 `local_id` 因此排在注册条目之后
   （插件条目占 `0`，结果行从 `RESULT_LOCAL_ID_BASE = 1` 起），两套句柄不会撞。
   前端只认事件里的 `row_id`（行下标），不必知道这个号段。

### 前端

5. reducer 里多了一份与 `conclusion` 并列的 `plugin_search`（`plugin_id` / `item_list` /
   `selection` / `total` / `action_indices`）；主列表的 `conclusion` 在这一页打开期间**不动**。
6. 插件条目的 `Enter` 与 `⇧+Enter` 都进搜索页（`keys.ts` 里按 `the_type === JS_PLUGIN_ITEM_TYPE`
   判定）；非插件条目的 `Preview` 行为不变。
7. `ESC` 与退格都返回主列表（退格只在搜索页里是一个意图，别的时候归输入框）；
   返回后主列表的 Selection 原样；结果页的选中位置不保留（每次进都从头开始）。
   窗口再次显示时也回到主列表。
8. 结果页为空显示成空页（不弹错误），装载失败与搜索抛错都归到这一条。

### 与 spec 的两处偏离（已确认形状）

- **结果行的 `action_ids` 由插件给，不是按类型查动作表**：spec §3.3 的行形状里就带它，
  而"按类型查表"做不到 issue 07 的"结果行可以没有动作"（清单的 `types` × `actions` 会让
  每个声明过的类型都带上全部动作）。清单那一份仍然是**派发时的白名单**
  （`has_action` 用它挡）与 `label_key` 的来源。结果页因此走宿主自己那条投影，
  不经过框架的 `project`（注册条目才走那条）。
- 结果页的 `token` 与三个分组边界都是 0：它不是主列表的延续，不翻页、不分匹配组。

### 文档（同一批改动里）

- `CONTEXT.md` 的 `Preview` 已按 spec §4.1 放宽（此前一轮就改好了）。
- ADR-0010 的两条"已知边界"补了后续指针：`window` 挂载点由 ADR-0011 定下；
  `action_icon` 全表查**到了要谈的时候**——这一轮只记录、不实现（等第二个插件才定得下形状）。

### 验证

`cargo build` / `cargo test`（35 通过）/ `pnpm build`（tsc + vite）都干净。
端到端四条的**真跑**还没做，见 issue 07。
