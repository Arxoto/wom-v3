# JS 插件宿主（第二轮：`Plugin Folder` / `Plugin Package` / `Plugin Search`）

## 背景与目标

第一轮立了 `plugin_framework` 与 `plugin_impl_launcher`，接入轮把它接进了应用（ADR-0009），
前端注册表轮把动作顺序、图标与文案收进了 `src/plugins/`（ADR-0010）。
三轮之后插件体系已经能跑，但**只有 Rust 编译期注册的插件能进来**：全仓唯一的注册调用是
`plugin_host.rs:93` 的 `register_plugin(Box::new(LauncherPlugin::new()))`，
没有目录扫描、没有清单、没有任何动态装载。

这一轮要验证的是**框架能吃下另一类形态的插件**：写在磁盘上、在**打包之后**还能加进去、
由 JS 实现、会失败、会晚于应用启动才被需要。

**成功判据（Q1）**：

1. 往打包后的 `Plugin Folder` 里加一个 `Plugin Package` 目录，**不重新编译**；
2. 它的关键字命中主检索，框架为它显示一行 `Plugin Item`；
3. 触发那行的动作，打开 `Plugin Search Page`，里面是**调用插件自己的搜索**得到的结果行；
4. 结果行按插件自带的信息画出图标与中文文案，它的动作能被派发执行；
5. 插件代码抛错 / 清单坏掉 / 目录不存在时，**主窗口与主检索照常工作**。

**交货边界（Q2）**：守 ADR-0008 的并行线。这轮**只加"JS 插件宿主"这一层**，
不碰 `builtin_plugins`，不合流两套体系，不顺手统一托盘重载与命令层。

---

## Language

沿用 spec 第一轮的词，新增四个（已同步进 `CONTEXT.md`）：

| 词 | 英文 | 中文 |
| --- | --- | --- |
| 打包后那个顶层目录 | `Plugin Folder` | 插件目录 |
| 目录里的一个插件（一目录 = `manifest.yml` + `index.js`） | `Plugin Package` | 插件包 |
| 插件注册的那一行（关键字命中时框架显示的行） | `Plugin Item`（定义不变） | 插件条目 |
| 插件实时搜索出来的那一批行 | `Plugin Search Result` | 插件搜索结果 |
| 装那批行的那一页 | `Plugin Search Page` | 插件搜索页 |
| 插件执行搜索这件事 | `Plugin Search` | 插件搜索 |

**`Plugin Item` 的定义不改**：它仍然是"启动时注册进框架、框架持有全量"的那个东西。
`Plugin Search Result` **不是** `Plugin Item`——它不注册进框架、不由框架持有，是每次查询现算的，
生命周期也不同（Q10 的裁定）。两者在界面上都由框架画行，这是它们唯一相同的地方。

`Plugin` 一词保持它的 Rust 含义（实现了 `Plugin` trait 的对象）。JS 插件在 Rust 侧**有一个代理**
实现了它，见 §2。**不把 JS 插件也叫 `Plugin`**。

---

## 一、`Plugin Folder` 与 `Plugin Package`

### 1.1 位置

`Plugin Folder` 由 `bundle.resources` 声明并随安装包分发：

```json
"bundle": {
  "resources": { "../plugins/": "plugins/" }
}
```

源目录在仓库根的 `plugins/`；打包后落在 `$RESOURCE/plugins/`。运行期用
`resolve_resource("plugins")` 定位。

**开发态与打包态的落点不同**（macOS 是 `Contents/Resources/`，Windows/Linux 在安装根附近），
这是本轮第一件要实测的事，见 §八 验证。

**目录内容是运行期可变的**：`bundle.resources` 决定"这个目录随包分发"，不决定"里面有哪些插件"。
往 `$RESOURCE/plugins/` 里手工加一个目录即可被扫到，无需重新编译。**这是必须支持的**（Q3）。

### 1.2 一个 `Plugin Package`

```
plugins/
└── <目录名>/
    ├── manifest.yml     # 必填：索引与声明
    ├── index.js         # 必填（`entry` 可改名）：插件实现
    └── icon.png         # 选填：清单里引用的图片
```

目录名**不是**身份，`manifest.yml` 的 `id` 才是（Q11）。目录名只用于定位文件。

### 1.3 `manifest.yml` 的格式与字段

**手写 `key: value` 行格式**，不用 YAML 解析器（Q11）：一行一条 `key: value`，
`#` 起头为注释，空行忽略，值不做引号/转义处理。仓库里没有 YAML 依赖，
而 launcher 的 `LauncherItemSource` 已经证明"手写行格式"这个仓库养得起。

| 键 | 必填 | 缺省 | 含义 |
| --- | --- | --- | --- |
| `id` | **是** | — | `PluginId`，稳定 ASCII；非法或重复→跳过该包 |
| `name` | 否 | `id` | 插件条目的名字 |
| `desc` | 否 | 空 | 插件条目的描述 |
| `keywords` | **是** | — | 逗号分隔；**主检索据此匹配出插件条目**（见 §1.3 末） |
| `icon` | 否 | 空 | 包内相对路径的图片 |
| `entry` | 否 | `index.js` | 包内相对路径的 JS 入口 |
| `actions` | 否 | 空 | 逗号分隔的 `ActionId` **有序**列表（第一个是默认动作）；空表示结果行没有动作 |
| `types` | 否 | 空 | 逗号分隔的**结果行类型名** |

**为什么 `keywords` 必须在清单里**（Q16，本轮最关键的一条决定）：
框架要在**主检索**里命中插件条目，就必须在那之前知道每个插件认哪些关键字。
放在清单里，Rust 在 `setup` 里同步读完即可建索引，于是
**"插件能不能被搜到"不依赖它的代码能否成功执行**——这是"坏插件不影响主窗口"的最强形式。
uTools 的 `plugin.json` 里的 `features[].cmds` 是同一个形状。

**动作的 `label_key` 不进清单**，按四段式约定推导：
`action.<插件 id>.<结果类型名>.<动作 id>`（沿用 Q35 的规范）。理由见 §2.4。

---

## 二、Rust 侧：代理插件

### 2.1 为什么必须有一个代理

`PluginRegistry::run_action` 是按 `ItemHandle` 找到插件块、调**那个块的 Rust `Plugin` 对象**
（`plugin_framework/mod.rs:581`）。所以：

> **一个 `Plugin Package` 在 Rust 侧必须有一个占据了插件块的代理 `Plugin` 实现，
> 否则它注册的条目一旦被触发，框架会走到一个不存在的插件上。**
> 代理与 `index.js` **必须共用同一个 `PluginId`**（`manifest.yml` 的 `id`）。

这是本轮新发现的结构性约束，也是"JS 插件"在架构上必然有两半的原因。

### 2.2 注册时机与框架公开面

宿主在 `setup` 里、`create_registry` 期间就同步完成：

1. 用 `resolve_resource("plugins")` 定位 `Plugin Folder`，列子目录；
2. 逐个读 `manifest.yml`；
3. 为每个合法的包注册一个代理 `Plugin`；
4. 代理的 `init` 里同步推入**它唯一的那一条 `Plugin Item`**（来自清单的
   `id` / `name` / `desc` / `keywords` / `icon`）。

**因此框架的公开面不需要新增入口。** 代理自己就是一个 `Plugin`，
它的 `init` 走的就是现成的 `ItemRegistrar`——第一轮定的"插件在 `init` 里一次性推完条目"
这条不变量**完全不用动**。

条目集在启动后不再变动（除重扫清单，见 §2.5），所以 `search()` 那条"只在关键字变化时重建结果"
的缓存（`mod.rs:553`）**不需要任何失效机制**。

### 2.3 代理的 `actions()`

代理的动作表就是清单里 `types` × `actions` 的笛卡尔积，`label_key` 按 §1.3 的四段式推导。
**框架侧的动作表因此只由清单决定**，与插件代码是否装载成功无关。

### 2.4 结果行的图标与文案由插件在装载时注册

清单能给出"有哪些类型、哪些动作"，但**给不出图标与中文**：

- 图标是 `ReactNode`（ADR-0010 的注册形状），清单里放不下；
- 中文**只住在前端**（AGENTS.md），Rust 侧一律 ASCII，清单也不例外。

所以插件代码装载时，通过 ADR-0010 的**同一个 `register`**（Q8/Q39）补上这两样：
结果类型与动作的图标、以及 `label_key` → 中文的文案表。

**这里有一个时序依赖，是本轮的核心不变量（写进实现要求）**：

> **`label_key` 与图标在"结果页渲染出来"之前必须已经注册好。**
> 因此插件的装载必须发生在**结果行下发之前**，而不是与它并行。

具体做法见 §3.3：宿主先等插件报告"已注册"，再向它发起搜索。

### 2.5 重扫

宿主提供一个"重扫 `Plugin Folder`"的动作：重新列目录、重读清单、对已存在的
`PluginId` 整块替换条目（`reload_plugin` 的现成语义），对新出现的包注册，对消失的包**保留
还是移除**——**这一轮不做移除**（见 §7 开放问题）。

---

## 三、运行期：装载、搜索与派发

### 3.1 装载方式：不 eval，插件代码跑在 Worker 里

**宿主不对插件源码求值**（Q7/Q15 的最终裁定）。装载走 asset protocol，执行放在一个
**经典 Worker** 里，与宿主文档隔开：

1. 宿主用 `resolve_resource("plugins")` + `manifest.yml` 的 `entry` 拼出绝对路径；
2. 用 `convertFileSrc` 转成 asset URL，随搜索请求发给前端；
3. 前端起一个同源的经典 Worker，Worker 里用 `importScripts(asset URL)` 取插件代码。

经典 Worker 是这一条的关键：`importScripts` 取脚本走的是经典脚本那条 no-cors 路，跨源的
asset URL 能直接拿；`new Worker(assetUrl)` 会撞同源规则，`{ type: "module" }` + `import()`
会撞 CORS。代价是 Worker 文件本身不能有 import/export——TypeScript 会给只有类型引用的模块
补一句 `export {}`，经典 Worker 遇到它直接语法报错，所以宿主与 Worker 共用的消息类型声明成
全局的（见 `src/plugins/plugin_worker_protocol.d.ts`）。

需要 `app.security.assetProtocol.enable = true` 且 scope 覆盖 `$RESOURCE/plugins/**/*`。
scope 是**构建期 glob，不是快照**，所以打包后新加进 `$RESOURCE/plugins/` 的文件照样命中。

**为什么不用 eval**（这是领域惯例，不是洁癖）：uTools、Raycast、Alfred、Flow Launcher、Wox
五家**没有一家**在宿主界面里对插件源码求值——它们要么给插件一份自己的文档（uTools/Raycast），
要么把插件放到另一个进程（Alfred/Flow/Wox）。`importScripts` 至少保住了"宿主不碰源码"这一条，
Worker 又把插件与宿主文档隔开。代价见 §3.5。

### 3.2 宿主交给插件的窄接口

按 Q8：**往 Worker 的全局挂一个窄接口对象，不挂 registry 本身**。

```js
self.__WOM_PLUGIN__ = {
  register(spec),   // { id, types: [...], labels: {...} } → 转调 registry.register
  log(text),
  fail(text),
}
```

命名带 `__WOM_PLUGIN__` 前缀，避免与插件自己的东西撞车。`register` 收到的 `search` / `run`
是两个函数，**留在 Worker 里**（函数过不了结构化克隆）；过线的只有类型 / 图标 / 文案这些纯数据，
由宿主转调 `registry.register`。

Worker 里没有 `window`，所以这里不留兼容别名：插件按 `self.__WOM_PLUGIN__` 写。

> **这一条改写了 ADR-0010:19**（原文："JS 插件将来怎么拿到这个单例……仍然没有定，
> 所以注册表不往 `window` 挂任何东西"）；改写记在 ADR-0011 里。

插件装载时调 `register` 交出**它的类型、动作图标与文案表**（§2.4）；
它还要交出**一个搜索函数**（见 §3.3）。

### 3.3 `Plugin Search` 的完整流程

```
用户在主列表选中「插件条目」，触发动作（Enter 或 ⇧+Enter 等价，见 §4.1）
        │
        ▼
前端 plugin_open_plugin_search(plugin_id, keyword)   ← 新命令
        │
        ▼
Rust：这个插件装载了吗？
        ├─ 没有 → 让前端装载（起 Worker + importScripts），等插件报告「已注册」
        └─ 有   → 直接用
        │
        ▼
Rust → 前端：调用插件注册的搜索函数（带上关键字）
        │
        ▼
插件返回 Plugin Search Result 列表（纯数据：name / desc / icon / 动作 id 列表）
        │
        ▼
Rust 走既有的投影：按结果行类型名查动作表（清单给的）→ 下发
        │
        ▼
前端建一个新的 item_list：Plugin Search Page
```

**"先等已注册、再发起搜索"是硬要求**，不是优化：`register` 递交的文案表与图标必须在
结果行渲染前到位（§2.4）。实现上用一个"装载完成"的 Promise 把 `open_search` 串在后面。

### 3.4 寻址与派发：`Item Handle`（Q23）

`item_index`（"条目在整集里的下标"）**对结果行不成立**——结果行不在那一集里。
所以结果行的寻址改用 `Item Handle`：

- `Item Handle` 已经是这个仓库为"不依赖条目在整集里的位置就能寻址"而造的身份
  （`CONTEXT.md`：*so an item can be acted on without knowing its position in the loaded set*）；
- 结果页的每一行随之下发它自己的 handle（`plugin_id` + 插件内 id）；
- 动作派发改用 handle，于是**两层列表对寻址完全透明**——管你在第几层，handle 都能找到行。

**这要求把 `Item Handle` 下发到前端**（现在它是框架内部类型，不下发）。
`PluginItemDisplay` 因此要带上 handle，主列表那一层也一并带上——
**一层带、一层不带会更糟**。

> 注意这与 `Item Index` **并存**，不是替换：主列表仍然靠 `item_index` 翻页与缓存
> （`search()` / `page()` 的 token 语义不变），`Item Handle` 只用于**动作派发**。

### 3.5 隔离程度与代价

一个包一个 Worker：插件跑在**自己的全局**里，摸不到宿主 DOM、改不到宿主的 `window`，
插件之间也不共享全局；`terminate()` 还顺手给了"重载插件代码"的能力。

代价与还没拿到的东西：

- **插件没有 DOM**，所以图标与文案必须是纯数据。这一条本来就成立：插件交上来的图标一直是
  字符串、结果行一直是纯对象，`search` / `run` 两个函数留在 Worker 内不过线。
  （旧版把这一条写成"与 ADR-0010 的 `ReactNode` 注册形状冲突"，是误判：`ReactNode` 只出现在
  宿主自己写的注册里——`launcher.tsx` 与 `js_host.tsx`，它们不进 Worker。）
- **不再零成本**：每个插件多一个 Worker 与一份消息协议，`register` 从同步调用变成消息往返。
- **同进程不同线程**：插件死循环不拖住界面，但仍与 webview 同进程，谈不上安全边界。
  要安全边界得走独立进程（Alfred/Flow/Wox 那条）。
- `csp: null` 意味着当前没有任何页面侧可执行内容的限制；把 CSP 收紧到 `script-src asset:`
  （Worker 另需 `worker-src`）是顺带收益。

---

## 四、前端：`Plugin Search Page`

### 4.1 与 `Preview` 的关系（Q21）

- **非插件条目**：`Preview` 行为**完全不变**（`⇧+Enter` 开关侧栏，显示 Selection 全文）。
- **插件条目**：`Enter` 与 `⇧+Enter` **都**进入 `Plugin Search Page`。

于是 `CONTEXT.md` 里 `Preview` 的定义**从"显示 Selection 全文"放宽为"显示当前项延伸内容的侧栏"**
——插件条目的延伸内容就是它的搜索结果页。

### 4.2 它是一份新的 `item_list`

`Plugin Search Page` 装的是**一份新的 `item_list`**，不是主列表的一份延续。
因此 reducer 里需要一份与 `conclusion` 并列的状态（形如
`plugin_search: { plugin_id, item_list, selection, action_index } | null`）。

主列表的 `conclusion`（含 `token`、`item_index`、分页）**在结果页打开期间保持不变**，
返回后原样恢复。

### 4.3 返回与状态（Q24）

- **ESC 与退格都返回主列表**；
- 返回后 `Selection` 回到那行插件条目上（原样）；
- `Plugin Search Page` 的选中位置**不保留**，下次进入从头开始。

### 4.4 结果行为空

结果行可以是空的（插件没搜到、插件代码加载失败、插件抛错）。
**显示成空页**，并在日志里记一条，**不弹错误**（界面上没有渲染"为什么失败"的地方，
见 §7 开放问题）。

---

## 五、命令、capability 与配置改动

新增命令（都在 `commands.rs`，只做转发，实现留在不依赖 Tauri 的那一层）：

| 命令 | 作用 |
| --- | --- |
| `plugin_list_packages` | 列出已发现的 `Plugin Package`（`id` / `name` / `entry`），供验证与重扫后核对 |
| `plugin_open_plugin_search` | 触发一次 `Plugin Search`，返回 `Plugin Search Page` 的数据 |
| `plugin_report_search_results` | 插件执行完搜索后把结果行回传（§3.3 的回程） |
| `plugin_reload_packages` | 重扫 `Plugin Folder`（§2.5，挂在临时托盘项上） |

**`capabilities/default.json` 要动**：现有权限只覆盖窗口 `main`（`core:default`、
`core:window:allow-start-dragging`、`opener:default`、`log:default`、`notification:default`）。
新增命令要在这里放行；**配置窗口不加**（Q9：这轮只在主窗口加载）。

**`src-tauri/tauri.conf.json` 要动两处**：`bundle.resources`（§1.1）与
`app.security.assetProtocol`（§3.1）。

---

## 六、必须显式重开的已知边界（不在本轮实现）

**`action_icon` 的全表查必须加插件维度。** ADR-0010:23 原文写着：

> `action_icon` 是**全表**查：行里只带动作 id，不带它属于哪个插件，所以两个插件用同一个动作
> id 配不同图标时会撞在一起……**等真有 JS 插件注册进来再谈**要不要加插件维度。

一旦真有两个插件，`copy` / `open_url` 这类通用动作 id 必然重复。
修法是给查找加上插件维度（`registry.tsx:109` 的 `action_icon` 与 `actions_of` 都要带上来源），
这又要连带改 `PluginItemDisplay` 的形状。

**本轮只把它记进 ADR 与 issue，不实现**——因为没有第二个插件时定不下来它的确切形状。

---

## 七、开放问题（本轮不裁）

1. **结果页为空时要不要提示原因**：现在没有渲染错误文案的位置，且 `ActionOutcome` 三态
   明确不带原因（Q6）。等界面有了位置再谈。
2. **`Plugin Search` 要不要接受关键字以外的输入**（比如用户在主搜索框里输入的后半段）。
   现在只能传"命中的关键字"，因为框架看不出哪部分是插件的、哪部分是参数。
3. **重扫时消失的包怎么处理**：保留它的条目（本轮的做法）还是移除？移除要处理
   "用户正停在那一行上"。
4. **结果行的类型名是否要求跨插件唯一**：清单里 `types` 是插件自报的，两个插件报同一个
   类型名会共用同一份动作表。现在没有校验。
5. **结果页的选中位置是否保留**（Q24 选了不保留）——若以后发现常驻有用再议。

---

## 八、验证

**必须实测的第一件事**：`resolve_resource("plugins")` 在 `tauri dev` 与 `tauri build`
两态各解析到什么绝对路径，以及 asset protocol 的 scope 是否对两态都命中。
**这条没确认之前，§1.1 与 §3.1 都还不能算成立。**

其余验证：

1. **端到端**：放一个探针 `Plugin Package` 进仓库的 `plugins/`，跑完 §成功判据的 1–4 条。
2. **坏插件隔离**（成功判据第 5 条），三种坏法各测一次：
   - 清单语法坏 / 缺 `id` / `id` 重复；
   - `index.js` 抛错；
   - `entry` 指向不存在的文件。
   三次都要求：应用启动正常、主检索正常、别的插件正常。
3. **打包后加插件**：`tauri build` 之后，往安装目录的 `plugins/` 里加一个新目录，
   重启应用，验证被扫到（这条是 Q3 的核心诉求，必须真做）。
4. **`cargo build` 干净** + `pnpm exec tsc --noEmit` 干净。
5. **单测**：`manifest.yml` 的解析（合法 / 缺必填 / 注释与空行 / 值里的空格）。
   按 AGENTS.md，测试留给用户；**本轮只写生产代码，测试逻辑若要改先与用户对齐**。

---

## 九、交接说明（交付环境的已知障碍）

`src/plugins/` 下有 **6 个"作废但删不掉"的残文件**，原因是 Windows 沙箱 ACL 拒绝了删除
（`.scratch/acl-report/` 记着：`WRITE_OWNER` 缺失）：

```
src/plugins/launcher.ts
src/plugins/registry.ts
src/plugins/probe.ts
src/main/item/action_icons.tsx
src/main/item/item_icons.tsx
src/main/interaction/action_labels.ts
```

它们**与本轮功能无关**，但会让工作树一直脏着。**本轮不尝试删除**（同样的 ACL 会再次拒绝），
只在这里记一笔：真要清理需要人工处理权限，或换一个能拿到 `WRITE_OWNER` 的方式。

同时提醒：**ADR-0010 与 `src/plugins/` 尚未提交**（工作树未跟踪）。
本轮的改动叠在它们之上，所以提交时要注意顺序。

> 后续：这六条已经清掉了（权限恢复之后 `Remove-Item` 直接成功），
> ADR-0010 与 `src/plugins/` 也已随"继续实现"那次提交入库。本节的记录到此为止。

---

## 十、被显式否掉的选项（备查）

| 选项 | 否掉的理由 |
| --- | --- |
| 宿主 `eval` 插件源码 | 领域里五家主流启动器无一家这么做；且是对 CSP 的隐性依赖（`csp: null` 掩盖着） |
| 直接把 asset URL 交给 `new Worker()` / module Worker | 前者撞同源规则，后者 `import()` 会撞 CORS；经典 Worker + `importScripts` 才走得通（§3.1） |
| 每个插件一个 iframe / 子 webview | 只买到"独立文档"，买不到"坏插件不拖死界面"（同进程同线程），两头不靠 |
| 独立进程 + IPC（Alfred/Flow/Wox 那条） | 同样是下一轮的正经主干；本轮不付这个工程量 |
| 新增 `PluginRegistry::register_items` 异步注册口 | 条目在 `setup` 里就能从清单同步拿到，没有"异步到账"这个需求 |
| 运行时 dylib 扫描 | 第一轮已否（ABI 是不该顺手承诺的坑），本轮不改 |
| 目录名即 `PluginId`、不要清单 | 清单是七家主流启动器的共同做法；没有清单就无法"扫目录建索引"，坏插件也无法在执行前识别 |
| YAML 解析器 | 字段是平的，手写行格式够用；引依赖不划算 |
| 插件传任意绝对路径当图标 | 把"给个图标"变成"读任意文件并显示"，收益与风险不成比例 |
| 结果行只有"执行"一个动作 | 动作表在界面上显示不出来（文案依赖"类型 + 动作"查 `label_key`） |
| 插件自己派发动作 | 与 ADR-0009/0010 收拢"投影与派发只有一处"的方向相反 |
| 启动时就装载所有插件代码 | 让"坏插件拖慢启动"变成现实，白费懒加载换来的收益 |
| 复用现有 `re_plugin` 托盘项做重扫 | 踩 Q2 的线（不顺手统一两套体系），本轮用独立的临时项 |
