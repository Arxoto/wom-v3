# 插件系统（第一轮：框架层 + launcher 插件）

## 背景与目标

现有的 `builtin_plugins/` **以 ItemType 为轴**组织：解析固定 5 个字段（`<->` 分隔，`parse_core.rs:10`）、
`Scan` 行经 WalkDir 展开成多条条目（`scans_helper.rs:21`）、每种类型的动作是一张写死的常量表
（`action.rs:108`）、整条链路挂在 `BuiltinStat` 这个 Tauri 托管状态上（`stat.rs:20`）。

这次要立一套支持插件的框架：**插件负责注册条目与实现动作，框架负责检索、前端交互与动作派发**。
框架与插件分层，未来还要能接前端插件（JS 注册）与自定义前端页面。

**第一轮的交货边界（硬约束）**

- 新代码放在两个**新的**顶层模块里，不改 `builtin_plugins/`。
- **不接入** Tauri 托管状态、不注册命令、不接线前端、不进检索链路。
- 唯一的例外是 `lib.rs` 里两行 `mod` 声明——不声明就编不进去。
- 因此这一轮它是**死代码**，验证方式是 `cargo build` 加解析相关的单元测试。

这样做的理由是"先立体系、再谈替换"：原地改造 `builtin_plugins` 会让每一次提交都同时面对
"新设计对不对"和"旧行为有没有被破坏"两个问题。见 `docs/adr/0008`。

---

## Language

- **Plugin Framework**：框架层。注册、检索、前端交互、动作派发。
- **Plugin**：插件。自己负责提供条目与实现动作。
- **Plugin Item**：插件注册进框架的条目。**不透明类型名 + 纯数据字段**。
- **Item Handle**：条目身份，`(plugin_id, address)`；地址区分"注册条目（注册序号）"与
  "结果行（行下标）"两段序号空间。
- **Plugin Action**：插件注册的动作，`(类型, 动作 id, label_key)`。
- **Plugin Context**：插件从宿主取能力的窄接口（唯一横切面）。
- **Launcher**：内置插件 launcher，支持 Sys / Cmd / Web / Scan 四种类型。
- **Launcher Item Source**：manifest 的一行，launcher 持久化层的中间表示。

---

## 一、框架层 `plugin_framework`

不依赖 `launcher`，不依赖 `builtin_plugins`，**不直接依赖 Tauri**（Q3）。

### 1.1 插件接口（Q2）

```rust
trait Plugin {
    fn id(&self) -> PluginId;                       // 稳定 ASCII 标识
    fn actions(&self) -> Vec<PluginAction>;         // 注册动作表（类型 → 动作 → label_key）
    fn init(&self, cx: &dyn PluginContext, registrar: &mut dyn ItemRegistrar)
        -> Result<(), PluginError>;
    fn run_action(&self, cx: &dyn PluginContext, item: &PluginItem, action_id: ActionId)
        -> ActionOutcome;
}
```

- 插件结构体**尽量无状态**，`init`/`run_action` 都取 `&self`（Q12）。
  框架要在派发时同时读表与调插件，`&self` 省掉一整层借用冲突。
  真需要缓存时插件自己用 `OnceLock`/`Mutex` 包自己的字段，框架不替它决定。
- 插件**不自己分配 handle**（Q30），只把条目推给 registrar。

### 1.2 条目注册（Q6 / Q9 / Q11 / Q12）

- 注册时机：`init` 里**一次性推完**。重载 = 重新 `init`，框架按 `plugin_id` **整块替换**。
- 框架持有**全量纯数据条目**，插件侧不做二次存储。
  这是 Q6 的裁定：**不采用**"Item 里嵌 `Arc<dyn Trait>`"那条路。
- 条目字段：`priority: i32`、`key_words: Vec<String>`、`name: String`、`desc: String`，
  外加一个**不透明**的 `the_type: String`（Q34）。
  框架对类型名的值永远不解释、不匹配、不列举，只透传。
- 句柄由框架分配：`(plugin_id, address)`，注册条目的地址是该插件内**从 0 递增的注册序号**
  （结果行是另一段序号空间，见第二轮 spec §3.4）（Q30）。
- 序列化仍是 `#[serde(tag = "the_type")]` 平铺（Q34），下发形状与现有 `common.rs:22` 完全一致，
  前端手写的镜像类型不需要任何改动。
- **不留**插件私有数据位（Q10）。将来真需要 per-item 绑定时再新增字段。

"Item 里嵌 `Arc<dyn Trait>`"被否掉的三条理由，记在这里备查：`Clone` 直接废掉（而 `ItemDisplay::new`
就是 `item.clone()`，`search.rs:103`）；序列化要被 `skip` 或手写；闭包捕获插件状态后，
插件就再也不能是值类型。而它唯一的收益（per-item 绑定）可以靠将来加字段解决。

### 1.3 注册表形状（Q21 / Q25 / Q26）

- `Vec<PluginBlock>`（**注册顺序即数组顺序**）配 `HashMap<PluginId, usize>` 反查下标。
- **不能用 `HashMap` 的迭代顺序**：哈希种子使顺序每次进程启动都可能不同，而我们要的是确定性排序。
  `HashMap` 只服务于"整块替换某插件的条目"。
- priority 在**注册时**做一次稳定排序（复刻 `load.rs:91`），检索时只做分组。

### 1.4 检索与排序（Q8 / Q26）

匹配算法原样沿用四种模式与优先级：**精确 > 前缀 > 包含 > 子序列**（`search.rs:111`）。

最终顺序的维度优先级：

```
匹配模式分组  >  priority  >  插件注册顺序  >  条目注册顺序
```

`priority` 之后是"注册顺序"而不是"文件内顺序"，这是与现有行为唯一的结构性差别，
也是新增插件维度必然带来的变化。分组、分页、`token` 语义全部照旧。

框架层**不做**打分插件（Q8 的 (b)），但把缝留着：排序维度是显式的、集中在一处。

### 1.5 动作（Q7 / Q13 / Q22）

- 动作表由**插件注册**，不再按 ItemType 写死常量表。
  现状里 `label_key` 必须按"类型 + 动作"分套（同一个 `copy` 在 Scan 上是"复制完整路径"、
  在 Web 上是"复制链接"，`action.rs:99`），类型轴正是这个要求的产物。
- 每条条目自带**有序的动作 id 列表**，第一个是默认动作。顺序即优先级。
- 返回 `ActionOutcome { Done, NoOp, Failed }`。
  `NoOp` 与 `Failed` 必须分开：现在"还没实现的动作不隐藏窗口"这条行为
  （`action.rs:219`）靠的就是这个区分。这一轮**不加**错误文案字段。
- 动作表**不加 revision**（Q22）：在可预期范围内，动作表相对插件与类型是固定的。
  前端维持"挂载时拉一次"的语义不变。

### 1.6 错误（Q18 / Q19）

三层错误，**不许合并**：

| 层 | 类型 | 归属 |
| --- | --- | --- |
| 框架级 | `PluginError`（init / action 两类） | 框架抽象 |
| 条目级 | `Box<dyn std::error::Error + Send + Sync>` | 插件私有，框架只 `to_string()` |

- 条目级失败**由插件自己消化**（Q19）：加载是插件自己的职责，框架只负责索引与前端交互。
  launcher 因此在 decode 阶段就地记日志，框架侧看不到"空行"与"字段不足"的区别——
  那是 launcher 的语义，不该被框架命名。
- 插件级隔离（Q14）：某个插件 `init` 失败时记 warn 并**跳过该插件**，其余插件照常注册，
  应用启动不受影响。一个坏插件不该让启动器起不来。
- **Rust 侧日志与错误串一律 ASCII**（仓库规则），只有注释与测试数据可以带中文。

### 1.7 Plugin Context（Q15）

这一轮只给**最小集合**：应用数据目录解析、写日志、动作结果回传。

剪贴板写入、打开路径/URL、事件下发**都不进去**——那是 launcher 的动作实现与"前端插件"
真正落地时的事。现在加进去就是为一个还没写的方法设计接口。

### 1.8 公开面（Q29）

只暴露两处：

1. `PluginRegistry`：构造 + 注册插件 + 检索 + 翻页 + 跑动作。
2. 动作表查询函数。

不暴露：注册表的块结构、条目地址的分配逻辑、`PluginError` 以外的内部类型。

### 1.9 失败隔离与诊断

沿用现有四分类的诊断风格（`load.rs:96`），但分类留在 launcher 内部，框架只收最终字符串。

---

## 二、launcher 插件 `plugin_impl_launcher`（Q37）

三个子模块：`persistence` / `action` / `init`。

### 2.1 支持的类型

**只支持 Sys / Cmd / Web / Scan 四种**（Q23）。

launcher 里自定义 `LauncherItemType`，**不复用** `builtin_plugins::base::ItemType`。三条理由：

1. 未来的 `Note`/`Snippets` 在插件体系里该是**独立插件**（各有各的持久化与动作），
   塞进 launcher 的枚举就把它们焊进了 launcher。
2. "框架不预设类型"这条决定要求类型名由插件定义，复用内建枚举等于把内建命名变成公共契约。
3. 新模块不依赖 `builtin_plugins`，复用会直接踩线。

类型名与现有一字不差：`"sys"` / `"cmd"` / `"web"` / `"scan"`。

### 2.2 `persistence` 子模块

- 常量：文件名 `launcher-manifest.txt`。
  放**本模块自己的**常量文件里，**不新增到** `src-tauri/src/constants.rs`——
  那个文件是旧体系的常量表。
- 目录：`app_data_dir()`（Q24），与现有内建设置文件同处。
- `LauncherItemSource`：manifest 一行的中间表示。**手写 `FromStr` + `Display`**，不用 serde derive（Q46）。

| 变体 | 字段 | 字段数 |
| --- | --- | --- |
| `Sys` | priority / key_words / name | **4**（无 desc） |
| `Cmd` | priority / key_words / name / desc | 5 |
| `Web` | priority / key_words / name / desc | 5 |
| `Scan` | priority 链 / key_words / name / desc(JSON) | 5 |

> **接入期的陷阱**：现有的 `parse_core` 对**所有**类型都要求 5 个字段（`ITEM_FIELD_COUNT`，
> `parse_core.rs:14`），所以现存的 manifest 里 `sys` 行是带第 5 个字段的（那份数据会被
> `into_system()` 当作 desc 收下）。新实现对 `sys` 只认 4 个字段，**接入时必须同时决定**：
> 是给 `sys` 保留一个被忽略的第 5 字段，还是让用户改写现有的 `sys` 行。
> 这不是笔误，是 Q46 的直接后果。

- `Scan` 的 `desc` 是**一行 JSON**，扫描配置（`file_types`/`file_suffix`/`black_list`/`max_depth`/
  `base`/`path`）都在里面（`parse_impl_scan.rs:55`）——照抄现状，这次不改文件语法（Q27）。
- `Scan` 的 priority 是**逗号分隔的链**（`1,2,,5`，空项沿用前一个，首个缺省 0），同样照抄（Q27）。
- `desc` 里的 JSON 含 `<`、`>` 时**不做转义**，照抄 `split_line` 行为。
- 字段数不符报"值不足"类错误。
- **迭代器**（Q47）：`LauncherItemSourceIter` 包住 `BufReader`，懒逐行产出
  `Option<Result<LauncherItemSource, LauncherParseError>>`。
  空行与坏行**由调用方决定**怎么处理，迭代器只如实产出错误。
- `Display` 这一半现在没有调用方（文件不落盘），但它是往返测试的前提，
  也是将来配置页写回 manifest 必须有的东西。
- manifest 不存在时：**创建父目录并写一个空文件**（Q45，复刻 `load.rs:51`）。
  因为是死代码，这次不会真的产生副作用；且"不存在"与"内容为空"合并成一种状态没有好处。

### 2.3 `action` 子模块

动作清单与顺序**原样收下**（Q33），不改语义：

| 类型 | 动作（顺序即优先级，第一个是默认动作） |
| --- | --- |
| Sys | **空**（每个命令的动作就是描述内容） |
| Cmd | `copy` |
| Web | `open_url`、`copy` |
| Scan | `open_path`、`reveal`、`copy` |

- `label_key` 的命名规范改为四段式：`action.<插件>.<类型>.<动作>`（Q35）。
  于是现有键迁移为 `action.launcher.scan.open_path` 这类形状。
- 框架**不强制**这个规范，只当不透明字符串透传；规范是 launcher 自己遵守的约定。
- 这一轮只实现 `copy`（Q36），其余三个动作留空：
  记 `ActionOutcome::NoOp` + warn，复刻 `action.rs:219` 的行为。

### 2.4 `init` 子模块

1. 用 `persistence` 的方法拿到 `LauncherItemSource` 迭代器。
2. 逐行转成框架的 Plugin Item。
   - `Scan` 走 `WalkDir` 展开：`follow_links(false)`、
     `file_types` 过滤、`file_suffix` 后缀过滤、`black_list` 含则排除、`max_depth` 直接交给 walkdir。
     **第 0 层是配置的 path 本身，不受过滤条件约束**（`scans_helper.rs:174`）。
     越界层级沿用链尾（`priority_at`，`scans_helper.rs:99`）。
   - 第 0 层沿用配置的 `name`，其余层级用文件名。
   - **`key_words` 始终是"配置关键字 ∪ 文件名"，且不去重**（Q31 / Q32）。
     这是**行为变更**，见下节。
3. 注册条目与动作表。

### 2.5 路径解析

`Scan` 的 `base` 用 Tauri 的 `BaseDirectory::from_variable` 解析（`scans_helper.rs:39`），
**不自己维护变量名映射表**——避免与 Tauri 的平台差异脱节。空 `base` 时 `path` 原样使用；
非空但认不出则报错，不退化成相对路径静默扫不到文件。

对应地，`PluginContext` 需要能提供路径解析能力；这是 Q15"最小集合"里
"应用数据目录解析"的扩展面。

---

## 三、已知行为变更（Q43）

**语法不变、语义变了**，这两件事必须分开记，否则将来读的人会以为什么都没变：

> 现有配置里凡是**配了** `key_words` 的 `Scan` 行，合并之后命中集合都会变大：
> 配置关键字与文件名都成为关键字，且每条扫出的文件都自带一个精确命中自己的关键字
> （因为它的 `name` 就是文件名）。

连带影响：`label_key` 从三段式变四段式后，前端文案表里那 8 处字面量会失配。
这一轮**不动前端**，所以两套键暂时并存，迁移进接入清单。

---

## 四、接入清单（不在本轮范围）

按依赖顺序：

1. `lib.rs` 加两行 `mod` 声明（本轮已加，但带临时 `allow(dead_code)`）。
2. 前端 `AppMain` 的 `label_key` 迁移到四段式。
3. `ItemDisplay` 显示投影改持 `Arc`（Q20 裁定 `Arc` 更优：每页省约 300 次 malloc，
   而 clone 方案每次翻页都产生一批冷的散落分配）。**推后**，但注册表签名按"将来会放 Arc"设计。
4. `PluginContext` 补齐剪贴板 / 打开路径 / 打开 URL，实现 `open_url`、`open_path`、`reveal`。
5. 动作表 revision（Q22 明确**不做**；将来若插件热重载成为常态再议）。
6. JS 插件宿主、自定义前端页面（见下节）。

---

## 五、前端接入规划（本轮不写一行前端代码，只留口子）

- 新增 `src/plugins/`：`registry.ts`（注册表 + 查找，纯数据无 React）、
  `launcher.ts`（launcher 的适配，默认导出注册函数）。
- 内置 Rust 插件与未来的 JS 插件走**同一条注册路径**（Q39）：
  内置的默认导出就是一次注册调用，JS 插件调同一个注册函数。
  两条路径分开的话，JS 插件能用的能力会永远只是内置能力的子集。
- 文案解析归属前端：Rust 只透传 `label_key`，前端注册表负责查文案。
- JS 插件将来怎么拿到那个单例（webview 全局 / 可 import 的模块 / 事件）**这一轮不定**，
  spec 只记三个候选。现在挂 `window` 会引入一个没有调用方的全局副作用。
- 现状下的前端注册点：`src/main/` 与 `src/AppMain.tsx` 一并纳入接入期改造。

---

## 六、模块布局与可见性（Q37）

```
src-tauri/src/
├── plugin_framework/          # 框架层，不依赖 launcher / builtin_plugins / Tauri
└── plugin_impl_launcher/      # launcher 插件
    ├── persistence/
    ├── action.rs
    └── init.rs
```

与现有 `builtin_plugins/` 平级。不要建一个 `plugins/` 父目录——
那只会凭空多一层，并把"框架 vs 插件"这个最重要的分界藏进目录名里。

- `plugin_framework`：只 `pub` 出 Q29 定下的两处入口，其余私有。
- `plugin_impl_launcher`：只 `pub` 出一个 `Plugin` impl 类型（接入时 `lib.rs` 唯一要碰的东西），
  三个子模块互相可见、对外私有。
- 两个模块顶部各加一行 `#![allow(dead_code)]` 并注明"接入前临时"，接入时删掉（Q17）。

---

## 七、验证

- `cargo build --manifest-path src-tauri/Cargo.toml` 干净（`dead_code` 已被临时 `allow` 压住）。
- `LauncherItemSource` 的解析单元测试：`FromStr`/`Display` 往返、字段数不符、
  priority 链、四类型分派。**这是本次唯一的测试**（Q44），其余留给接入那一轮。
- 前端一行不动，`pnpm exec tsc --noEmit` 不受影响。

---

## 八、被显式否掉的选项（备查）

| 选项 | 否掉的理由 |
| --- | --- |
| `Arc<dyn Trait>` 嵌进 Item | `Clone` 废掉、序列化要拆弹、闭包捕获把生命周期绑回插件；收益只有 per-item 绑定 |
| 注册回调 / handler map（Q2 的 b） | handler 生命周期漏进框架自有存储，没有收益 |
| 插件自派发、框架只转发（Q2 的 c） | 框架无法在不问遍插件的情况下回答"有哪些动作" |
| 运行期扫目录加载 dylib（Q4） | ABI 是个不该被顺手承诺的坑 |
| 插件自带 scorer（Q8 的 b） | 在第二个插件不存在时定 scorer trait，是对着想象的需求设计 |
| 复用 `builtin_plugins::base::ItemType`（Q23） | 会把内建命名变成公共契约，并踩 Q1 的边界 |
| 框架提供 `<->` 解析工具（Q28） | 提供了解析工具，框架就变相拥有解析的形态 |
| `HashMap` 迭代顺序当排序（Q25） | 哈希种子让顺序每次启动都可能不同，与确定性排序冲突 |
| 动作表带 revision（Q22） | 可预期范围内动作表是固定的，多余 |
| 改 manifest 语法为平铺字段（Q27） | 会让现有用户配置立刻失效，而这次没有迁移收益 |
| 条目地址由插件分配（Q30） | 稳定性与唯一性就成了插件的账，而那是框架该背的 |
