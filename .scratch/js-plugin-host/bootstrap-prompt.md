# 提示词：在 Tauri v2 项目里搭一个业务无关的 JS 插件框架

> 下面整段就是提示词，可原样贴给新项目里的 agent。它只描述框架本身：插件从哪来、
> 怎么被宿主发现、代码在哪执行、宿主与插件怎么通信、出错与重载怎么处理。
> 不含任何具体业务。

---

## 目标

给一个 Tauri v2 桌面应用加一层**通用插件框架**，要求：

1. 插件是磁盘上的一个目录（一个声明文件 + 一个 JS 入口），**打包之后**往插件目录里放一个新
   目录、重启即可被宿主发现，不需要重新编译；
2. 宿主**不接触插件源码字符串**（不用 `eval` / `new Function`），插件代码与宿主界面隔离；
3. 插件坏掉 / 声明坏掉 / 目录不存在时，宿主应用照常启动与运行；
4. 框架层与 Tauri 解耦，以后换宿主或换执行方式时，改动集中在一个横切面上。

## 总体分层

```
宿主外壳（Tauri + webview）      文件扫描、宿主能力实现、窗口与事件
框架层（纯 Rust，不依赖 Tauri） 插件抽象、注册表、包模型、生命周期
前端（React/TS）                插件交上来的展示数据 → 图标/文案/渲染
插件（JS，跑在 Worker 里）      通过宿主提供的窄全局接口与宿主通信
```

依赖方向单向：前端 → 命令层 → 框架层。框架层不依赖 Tauri，插件更不依赖；插件要用的宿主能力
一律经过一个窄接口（可注入的实现），而不是拿到 `AppHandle`、`window` 或 `self`。

## 一、插件包格式

```
plugins/                     # 插件根目录，随应用一起分发，运行期可增删
└── <目录名>/                # 目录名只是定位用，身份由声明文件里的 id 决定
    ├── manifest.<ext>       # 必填：声明
    ├── index.js             # 入口（可在声明里改名）
    └── <资源文件>           # 选填：图标等
```

声明文件建议用**极简的 `key: value` 行格式**（不引 YAML 解析器）：一行一条、按第一个 `:`
切开；`#` 起头是注释、空行忽略；键值两侧 trim、值内部原样保留；同键后写的赢；认不出的键忽略。
字段控制在最小集合：`id`（必填、稳定 ASCII）、`name`、`version`、`entry`、以及插件要用的
资源路径与权限声明。**声明只做索引与校验，不承载任何可执行内容。**

关键决定：声明在应用 `setup` 里**同步**解析完，于是**宿主能不能发现这个插件，不依赖它的 JS
能不能执行**——这是"坏插件不影响主程序"的最强形式。入口 JS 一律**懒加载**，启动时不执行。

## 二、Rust 框架层

抽象最小化：

```rust
pub struct PluginId(pub String);          // 稳定 ASCII 标识

pub trait PluginContext: Send + Sync {    // 插件能用的全部宿主能力，都是窄窄一条
    fn app_data_dir(&self) -> Result<PathBuf, String>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String>;
    fn write_clipboard(&self, text: &str) -> Result<(), String>;
    fn open_url(&self, url: &str) -> Result<(), String>;
    fn log(&self, level: LogLevel, msg: &str);
    // 只放确需的能力，错误一律 Result<_, String>（ASCII 诊断文本）
}

pub trait Plugin: Send + Sync {
    fn id(&self) -> PluginId;
    fn init(&self, cx: &dyn PluginContext) -> Result<(), PluginError>;   // 只在需要时调
    // 宿主把一次调用交给插件：命令名 + 纯数据参数 → 结果或错误
    fn invoke(&self, cx: &dyn PluginContext, command: &str, payload: serde_json::Value)
        -> Result<serde_json::Value, PluginError>;
    fn shutdown(&self);                                                   // 卸载/重载
}
```

注册表要点：

- 内部用 `Vec<插件块>`（注册顺序即顺序）配 `HashMap<PluginId, usize>` 只做反查；
  任何迭代顺序都以 `Vec` 为准，不要依赖 HashMap 的顺序；
- 装配期注册取 `&mut`，运行期重扫/热重载取 `&self`（托管进 Tauri 后只有 `State`）；
- **失败隔离**：单个插件 `init` 失败只记日志并跳过它，其余插件与宿主启动不受影响；
- 卸载与重载以"插件 id 整块替换"为语义，句柄由框架分配、插件不自己造；
- 派发一律按"插件 id + 插件内 id"寻址，不要用插件在全局列表里的下标——插件的运行期产物
  可能不在框架持有的列表里。

框架层不做任何业务判断：命令名与参数形状都是不透明数据，框架只负责路由与隔离。

## 三、JS 插件在 Rust 侧有一个代理

每个 JS 包在注册表里占一个插件槽，槽里放一个**代理 `Plugin` 实现**，它与入口 JS 共用同一个
`PluginId`（来自声明文件）。没有这个代理，插件注册的东西一旦被触发，框架会走到一个不存在的
插件上。

代理的实现全部只读声明：`id`、元数据、能力清单、入口路径。真正需要执行的调用再转给前端
（见下一节）。于是"插件是否存在、声明了什么"与"插件代码是否可用"彻底解耦。

## 四、执行路线（这一步是全框架的核心）

**不注入 `<script>`、不求值源码**，用 Worker 执行：

1. 宿主由声明里的相对路径拼出入口的绝对路径，转成 webview 可取的 asset URL 交给前端；
2. 前端起一个**与页面同源的经典 Worker**（前端自己的一个脚本文件），
   Worker 内用 `importScripts(asset URL)` 取插件代码；
3. 一个插件一个 Worker：插件跑在自己的全局里，摸不到宿主文档，插件之间不共享全局；
   `terminate()` 就是"卸载"，重新起一个就是"重载"。

为什么必须是经典 Worker：`new Worker(assetUrl)` 撞同源规则，module Worker 里 `import()`
撞 CORS；`importScripts` 走经典脚本那条 no-cors 路，跨源的 asset URL 能直接取。
代价是**Worker 脚本里一个 `import` / `export` 都不能有**——TypeScript 会给"只引用类型"的
模块补一句 `export {}`，经典 Worker 会直接语法报错，所以宿主与 Worker 共用的消息类型要声明成
**全局类型**，两边都不 import。

需要的宿主配置：asset protocol 打开且 scope 覆盖插件目录（scope 是构建期 glob、不是快照，
打包后新增的文件照样命中）；Rust 依赖里显式开启对应 feature（`cargo build` / `cargo test`
不走应用的配置文件）。

## 五、宿主与插件之间的接口

Worker 全局上挂一个**窄接口对象**（挂接口，不挂宿主内部对象）：

```js
self.__PLUGIN_HOST__ = {
  register(spec),      // 交出插件的元数据与入口点，宿主转交给框架/前端
  request(capability), // 需要宿主能力时走这里，由宿主白名单决定放不放行
  log(text),
  fail(text),
}
```

消息协议（结构化克隆）保持最小：

```
宿主 → Worker：{ kind: "load" } | { kind: "invoke", id, command, payload } | { kind: "dispose" }
Worker → 宿主：{ kind: "registered", spec } | { kind: "result", id, value }
             | { kind: "error", id, message } | { kind: "log" | "fail", text }
```

两条硬规则：

- **函数不能过线**：插件注册的展示数据（图标、文案、类型标识）必须是纯数据，通常是字符串；
  真正的处理函数留在 Worker 内，宿主只按 id 触发；
- **有请求就要有回程**：凡宿主在等的消息，任何分支（包括异常）都必须回一条，否则那次调用
  永远不落地。宿主侧对回程做超时。

## 六、前端：元数据注册与渲染

插件交上来的展示元数据（类型标识 → 图标、动作 id → 图标、文案键 → 文案）注册进一个
**前端注册表单例**，每个 webview 一份。内置能力与 JS 插件走**同一条注册路径**，否则 JS 插件
能用的能力会永远只是内置能力的子集。图标允许两种形态：内置的 JSX 元素与插件交上来的字符串
（图片地址 / data URI），认不出就不画。文案统一由前端按文案键组装，宿主侧只传键、不传中文。

宿主侧只下发**结构**（有哪些插件、插件声明的标识、入口点在哪里），不解释标识的含义。

## 七、生命周期与容错

```
setup：扫插件目录 → 解析声明 → 为每个合法包注册代理（不执行任何插件代码）
首次被需要：起 Worker + 装载入口 → 插件报告"已注册" → 之后才允许转发调用
失败：记录一条即可；失败不缓存，下次再需要时重试
重扫：重读声明、对已有 id 整块替换、新包注册；消失的包先保留槽位、按产品决定何时清理
卸载：terminate() + 调 shutdown，清掉在飞的请求与账本
```

隔离边界要说清楚：同进程不同线程**不是安全边界**，只能保证插件不拖住界面、不碰宿主 DOM。
要真正的安全边界得走独立进程 + IPC（把 Worker 换成子进程），这一轮不做，但接口要留出这个口子。

## 八、必须避开的坑

1. 经典 Worker 里不能有任何 `import` / `export`，含 `import type`；
2. 不要把 asset URL 直接交给 `new Worker()` 或 module Worker；
3. 宿主不持有插件源码字符串，不用 `eval` / `new Function`；
4. 插件返回的数据过线前先归一化：字段缺了给默认值，但**一条都不能丢**（宿主按顺序/下标记账）；
5. 请求与回程要一一对应，异常分支也要回；有超时；
6. 标识（插件 id、命令名、类型名、文案键）在宿主侧一律当不透明 ASCII 字符串，不解析、不校验语义；
7. 中文只住在前端，宿主日志与错误串一律 ASCII。

## 九、验证清单

1. 打包后往插件根目录手工加一个插件，重启能被发现；
2. 三种坏法各测一次（声明语法坏 / 入口抛错 / 入口文件不存在），都要求宿主应用正常启动、其余插件正常；
3. 插件处理函数阻塞或抛错时，宿主界面不卡死，错误只落日志；
4. 重载/重扫后插件用的是新代码；
5. `cargo build` 与前端类型检查都干净。

