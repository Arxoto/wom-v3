# AGENTS.md

给在本仓库工作的 agent 的指引。

## 本文档自己的规则

- 只放两类内容：**开发原则**与**架构索引**，保持简短。
- **代码是唯一事实依据**。字段名、函数签名、模块清单、格式、常量、依赖版本一律看代码，本文档不复述这类事实。
- **长期维护的只有代码里的文档注释**：设计原因与选型取舍就近写在被解释的模块 / 类型上。`docs/adr/` 只记「当时的结构是什么」并指向对应类型，不写理由。
- 需求文档（`.scratch/<feature>/`）只放需求与当事的设计；需求结清即冻结，之后不再维护、也不作为改代码时的依据。
- 与代码冲突时以代码为准，顺手把本文档改回来；不要为迁就本文档而改代码。

## 开发原则

- 只写生产代码；测试留给用户。允许修改既有测试，但**测试逻辑一有改动，先与用户对齐**。
- 动手前先读该模块顶部的文档注释与相关 ADR。
- 注释分三层：函数 / 字段写「它是什么」（入参、返回值、不变量）；文件头顶部的文档注释放本文件的方案选型；ADR 与 `.scratch` 票据放理由与历史。代码在做什么要能从源码直接读懂，改动过程与逐次取舍不进注释。
- 函数级最多留一句理由，且必须是「不写就容易被人改错」的那一种（例：对不上直接报错，别退化成空页）；一句话说不完的理由属于文件头或 ADR，别在函数上展开。
- 依赖单向流动：`commands` → `configs` / `window_utils` / `global_shortcut`，不允许反向依赖。
- 配置只有一份：`Config` 既是 `config.json` 的格式，也是运行时唯一依据；派生值用方法现算，不另存运行时结构。
- 配置读写不对称：读盘宽容（未知键忽略、缺失字段取默认），写盘严格（必须整份回传并通过校验）。
- 前端手写镜像 Rust 的类型（见 `src/core.tsx`），改名不会有编译期报错，靠金样本测试兜底。
- 平台差异交给平台 API 或 tauri 解析，本项目不另立映射表。
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
| `builtin_plugins/` | 内建条目与检索：`base` / `common` / `persistence` / `search` / `stat` |

前端 `src/`：`core.tsx` 是两个入口共用的部分（类型镜像、invoke 封装、css 变量），`index_main.tsx` / `index_config.tsx` 是入口，`AppMain.tsx` / `AppConfig.tsx` 是根组件，`main/` 装主窗口的布局与各区块。

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
