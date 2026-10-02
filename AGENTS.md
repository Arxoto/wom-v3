# AGENTS.md

给在本仓库工作的 agent 的指引。

## 开发原则

- 默认不写注释；设计理由的注释要先征得用户同意。
- 允许改动使既有注释与代码相符，新增注释内容需要先征得用户同意。
- 只写生产代码；测试留给用户。允许修改既有测试，但**测试逻辑一有改动，先与用户对齐**。
- 依赖单向流动：`commands` → `configs` / `window_utils` / `global_shortcut`，不允许反向依赖。
- 前端手写镜像 Rust 的类型（见 `src/core.tsx`），改名靠金样本测试兜底。
- 中文只住在前端：Rust 的日志、错误串与下发的值一律 ASCII（诊断用英文），用户可见的文案由前端按文案键组装（见 `label_key`）；Rust 里只有注释与测试数据可以带中文。
- 注释与提交信息沿用仓库现状：中文。

## 架构索引

WOM 是 Tauri v2 桌面启动器：常驻托盘，由全局快捷键唤出主面板。两个 HTML 入口各挂一个 React app——`index.html`（主窗口）与 `index_config.html`（配置窗口）。

Rust 后端 `src-tauri/src/`：

| 模块 | 职责 |
| --- | --- |
| `lib.rs` | 应用组装：插件注册、托盘菜单、窗口生命周期、命令注册表 |
| `commands.rs` | 前端调用的命令；依赖的最外层 |
| `configs.rs` | `Config` 的读写、校验与派生值 |
| `window_effect.rs` | 窗口外观：原生效果、可用性与降级链 |
| `window_utils.rs` | 窗口的创建、显示与重建 |
| `global_shortcut.rs` | 全局快捷键的注册与运行期状态 |
| `shortcuts.rs` | 快捷键字符 |
| `constants.rs` | 文件名、窗口 label 等常量 |
| `builtin_plugins/` | 内建条目与检索：`base` / `common` / `persistence` / `search` / `stat`；与插件体系并行存在 |
| `plugin_framework/` | 插件框架：插件与条目注册、检索、投影、动作派发（见 `docs/adr/0008`） |
| `plugin_impl_launcher/` | launcher 插件：`persistence` / `action` / `init` |
| `plugin_package/` | `Plugin Package` 格式：`manifest`（清单解析）与 `Plugin Folder` 扫描（见 `docs/adr/0011`） |
| `plugin_proxy_js.rs` | JS 插件在框架里的 Rust 侧代理：与插件共用 `PluginId` 的 `Plugin` 替身（见 `docs/adr/0011`） |
| `plugin_host.rs` | 插件框架的宿主侧：注册表组装与重载 / `context`（`PluginContext` 的 Tauri 实现）/ `js`（扫描同步、运行期账本与搜索回程） |

前端 `src/`：`core.tsx` 是两个入口共用的部分（类型镜像、invoke 封装、css 变量），`index_main.tsx` / `index_config.tsx` 是入口，`AppMain.tsx` / `AppConfig.tsx` 是根组件，`main/` 装主窗口的布局与各区块，`plugins/` 装前端插件注册表。

主窗口读的是插件那一套命令（`plugin_search` …）；内建那一套命令仍在 Rust 侧注册，但前端已无封装（见 `docs/adr/0009`）。

`plugins/` 的职责（见 `docs/adr/0010` / `0011`）：`registry.tsx` 是注册表本体——「类型名 → 图标 / 动作 id → 图标 / `label_key` → 中文」三张表与动作解析，纯数据无 React 组件；`launcher.tsx` 是 launcher 插件的注册内容（默认导出的注册函数，内置与 JS 插件走同一个 `register`）；`launcher_icons.tsx` / `launcher_action_icons.tsx` 是 launcher 自己的图标；`js_host.tsx` / `js_host_icons.tsx` 是 JS 插件宿主给插件条目那一条注册；`host.ts` 是 JS 插件宿主的前端一半（`<script src>` 装载器 + `window.__WOM_PLUGIN__` 窄接口）；`plugin_icon.tsx` 画一张图标（JSX 或图片地址）。内置插件的注册在 `index_main.tsx` 里、渲染之前完成。

仓库根的 `plugins/` 是 `Plugin Folder` 的源目录，随 `bundle.resources` 分发；打包后往安装目录的同名目录里加一个 `Plugin Package`（一目录 = `manifest.yml` + `index.js`），重启即可被扫到。

文档与约定：`CONTEXT.md`（术语表）、`docs/adr/`（架构决定）、`docs/agents/`（issue tracker、triage 标签、domain 规则）。

## 构建 / 运行

- Node 环境由用户预先切好：agent 直接执行 `pnpm` / `node`，不要自行调用 `fnm` 或另找 Node。
- 开发：`pnpm tauri dev`
- 打包：`pnpm tauri build`
- 前端单独：`pnpm dev`（1420）、`pnpm build`（tsc + vite build）
- 前端类型检查：`pnpm exec tsc --noEmit`
- Rust 测试：`cargo test --manifest-path src-tauri/Cargo.toml`
- 包管理器用 pnpm

## Agent skills

- Issue tracker：spec 与 issue 是 `.scratch/<feature>/` 下的 markdown，见 `docs/agents/issue-tracker.md`。
- Triage labels：五个标准角色，见 `docs/agents/triage-labels.md`。
- Domain docs：单上下文，`CONTEXT.md` 与 `docs/adr/` 在仓库根，见 `docs/agents/domain.md`。
