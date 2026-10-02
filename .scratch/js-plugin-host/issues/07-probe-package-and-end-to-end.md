# 07 — 探针插件包与端到端验证

Status: resolved
Category: enhancement

## 目标

把 spec §八 的验证真跑一遍，产出一个可重复的验证结论。

## 范围

1. **探针 `Plugin Package`**（仓库根的 `plugins/`，随包分发）：
   - `manifest.yml`：一个 `id`、若干 `keywords`、一个结果类型名、两个 `actions`（验证顺序即优先级）；
   - `index.js`：调 `window.__WOM_PLUGIN__` 注册类型/动作的图标与文案，并给出搜索函数
     （返回几条结果，其中一条**故意没有动作**，验证"结果行可以没有动作"）；
   - 一个 `icon.png`（或 SVG）验证图片文件路径那条路。
2. **端到端**：spec §八 的 1–4 条。
3. **坏插件隔离**：三种坏法各测一次（清单坏 / `index.js` 抛错 / `entry` 不存在），
   三次都要求应用启动正常、主检索正常、别的插件正常。
4. **打包后加插件**：`tauri build` 之后往安装目录的 `plugins/` 加一个新目录，
   重启验证被扫到（**这条是需求的核心诉求，必须真做**）。
5. `cargo build` 与 `pnpm exec tsc --noEmit` 干净。
6. 把结论、失败与意外写进本 issue 的 `## Answer`。

## 不在范围

测试（按 AGENTS.md 留给用户，测试逻辑要改先对齐）；
`action_icon` 的插件维度（spec §六，下一轮）。

## Blocked by

01–06 全部。

## Answer

### 已经做完的

1. **探针 `Plugin Package`**（仓库根 `plugins/probe/`）：
   `manifest.yml`（`id: probe`、`keywords: probe,wom-probe`、`icon: icon.png`、
   `entry: index.js`、有序的两个动作 `greet,log`、一个结果类型 `probe_result`）、
   `index.js`（用 data URI 画两张动作图标与一张类型图标、交出中文文案表、给出搜索函数
   与动作处理函数；三行结果里**第二行故意不带动作**）、`icon.png`（条目自带的那张图）。
2. 端到端与坏插件隔离所需的**代码路径**都在：清单坏了 / 入口文件不在 → 扫描跳过并 warn，
   条目不受影响；`index.js` 抛错 → 装载器判定"没注册"，这一次搜索是空页，主窗口照常。
3. `cargo build` / `cargo test`（35 通过）/ `pnpm build`（tsc + vite）干净。

### 还没做的（需要跑起来）

`tauri dev` / `tauri build` 都是**起 GUI** 的动作，本次会话没有跑，所以：

- spec §八 的端到端 1–4 条（主检索命中插件条目 → 结果页 → 结果行图标与文案 → 结果行动作）
  **没有真跑过**；
- 坏插件隔离的三种坏法**没有各跑一次**；
- 打包后往安装目录的 `plugins/` 加一个包、重启验证被扫到**没做**。

跑的时候可以用的入口：托盘菜单 "Reload Plugin Packages"（重扫并记一条包数日志）、
`plugin_list_packages`（`id` / `name` / 入口绝对路径 / 入口在不在）、
以及 `plugin folder: path=… exists=…` 那条启动日志（开发态与打包态各一条）。

### 一个已经实测到的落点（subset of issue 01）

`cargo build` 之后，`bundle.resources` 声明的 `../plugins/` 被 `tauri-build` 拷到了
`src-tauri/target/debug/plugins/`，探针包在里面；而 Windows 上 `resolve_resource` 解的是
可执行文件所在目录，所以开发态的 `Plugin Folder` 就是它。剩下的三件（运行时返回的路径、
asset protocol scope 是否命中、打包态落点）仍然要跑起来才知道——见 issue 01。

补充（同一次 `pnpm tauri dev`）：应用启动日志里有
`plugin folder: path=\\?\…\target\debug\plugins exists=true`、
`js plugin ready: probe`、`plugin registered: probe`，扫描、清单解析、代理注册三步跑通。

### 开发态端到端：真跑过了

一次人工点检（主窗口里敲字、回车、左右切动作），日志逐步对得上：

```
# 2/3：插件条目的动作打开结果页——Rust 的请求与前端回程各一趟，17ms
invoke plugin_open_plugin_search start
invoke plugin_report_search_results start      ← 前端先装载脚本、跑插件搜索，再回传
invoke plugin_open_plugin_search done in 17ms

# 4：结果行的动作派发到插件里（默认动作 greet）
run action plugin=probe action=greet
[plugin] probe run: action=greet row=关键字：probe
[plugin] action dispatched: probe greet
```

即：`<script src>` 经 asset protocol 装载并执行、结果行按插件自己注册的图标与中文画出来、
结果行的动作回到了 `index.js` 的 `run()` 里。**这一条也回答了 issue 01 的 scope 问题**。

启动隔离：三个坏包各一条 warn（`bad_manifest` 行号、`no_id` 缺 id、`dup_demo` 重复 id），
应用与其它插件照常起来；`bad_entry`（入口不在）与 `throws`（脚本抛错）的条目照常能被搜到，
按人工点检的结果，打开它们的搜索页是**空页**（这两次装载的 warn 没被日志截到，见文末）。

重复 id 的实际表现（人工确认时以为会搜不到，其实不对）：**先扫到的那个照常注册**，
后扫到的被跳过，所以 `probe-dup` 命中一行（`Dup A`），日志里
`search settle key="probe-dup" token=41 total=1`。

### 运行期加包：也真跑过了

应用**跑着**的时候往 `plugins/` 里放一个新包（`second`），点托盘
"Reload Plugin Packages"：

```
[tray][INFO] try reload plugin packages
[plugin_host][INFO] js plugin ready: second
[plugin_framework][INFO] plugin registered: second     ← 新增，不是 replaced
[tray][INFO] plugin packages reloaded: 5

search settle key="probe-se" total=1                   ← 新包的条目被主检索命中
invoke plugin_open_plugin_search start
[plugin] second run: action=ping row=Second 命中：probe-se
[plugin] action dispatched: second ping
```

不重编译、不重启，加一个目录就能被扫到、被搜到、能装载、能派发——Q3 的核心诉求成立。

### 仍然待做

- **打包态**：`tauri build` 之后的安装目录落点，以及往安装目录的 `plugins/` 加一个包、
  重启验证被扫到（spec §八 第 3 条，Q3 的原始诉求）。
- 坏包的三种坏法里，"清单坏 / 缺 id / id 重复"已由启动日志证明；"入口不存在 / 脚本抛错"
  只做到"条目照常可搜 + 搜索页空"这一步，**日志里没有留下那两次装载的 warn**（没截到）。

### 打包态（安装目录）实测：本次补完

用 `pnpm tauri build --bundles nsis` 出包并静默安装到 `%LOCALAPPDATA%\wom`，补上此前缺的两条：

1. **打包态落点**：`plugin folder: path=\\?\C:\Users\Jesus\AppData\Local\wom\plugins exists=true`，
   安装目录里有随包分发的 `plugins\probe\`（`manifest.yml` / `index.js` / `icon.svg`）。
2. **打包后加包**（Q3 的核心诉求）：装完之后往安装目录的 `plugins\` 放一个 `second` 包，重启应用 →
   `js plugin ready: second` 加 `plugin registered: second`。**不重编译、不重打包**，目录一放就能被扫到。

坏包隔离也在打包态复验了一遍（清单坏 / 缺 id / id 重复各一条 warn，其余插件照常注册），
只剩"入口不存在 / 脚本抛错"这两种**装载期**坏法仍要点一次界面才能看到 warn。

### 人工点检收尾：已完成

上面那两条界面项已由人工点检走完：主检索命中插件条目 → 结果页 → 结果行图标与文案 → 结果行动作；
`entry` 不存在 / `index.js` 抛错两种装载期坏法各点一次，都是空页。至此 spec §八 的 1–5 条
全部跑过一遍。

点检过程中发现并修掉一个 bug：**从 `Plugin Search Page` 返回主列表时，列表从头开始显示**——
Selection 是对的，但可见窗口回到了第一行，不是进入前那一屏。原因是 `Body` 的滚动偏移按"层"记账
（`src/main/Body.tsx`），换层时被清零，返回主列表时没有把离开前的偏移还回来。已改成两层各存一份
滚动位置，换层不再互相覆盖。

## Comments

后续重构（插件包重扫）：`plugin_list_packages` 命令已删除，验证入口只剩托盘项与日志；
端到端点检结论不受影响。
