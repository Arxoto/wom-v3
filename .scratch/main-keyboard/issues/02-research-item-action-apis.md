# 四类动作的插件覆盖

Type: research
Status: claimed

## Question

四类候选动作——复制进剪贴板、用默认浏览器打开、用默认方式打开（文件/文件夹/应用）、在文件资源管理器中选中该文件——在 Tauri v2 官方插件里分别怎么实现？

要确认：`tauri-plugin-clipboard-manager` 是否存在、写文本的 API 与平台支持；`tauri-plugin-opener` 的准确 API 面（open_url / open_path / reveal_item_in_dir 之类）；各自需要的依赖、capabilities 权限与配置；Windows / macOS / Linux 的可用性与限制；`navigator.clipboard` 这条官方路径的条件。

## Answer

结论落在 [`research/item-action-apis.md`](../research/item-action-apis.md)（本机一手源码：`tauri-plugin-opener` 2.5.5 + `tauri-plugin-clipboard-manager` 2.3.3）。

- 四类动作都有官方路径：复制 → `tauri-plugin-clipboard-manager` 的 `write_text`（**当前不是本仓库依赖**，要新增）；默认浏览器打开 → `opener::open_url`；默认方式打开 → `opener::open_path`；资源管理器中选中 → `opener::reveal_item_in_dir`。
- `opener:default` 已经包含 `allow-open-url` 与 `allow-reveal-item-in-dir`，但**不含 `open_path`**。
- 关键机制：插件的 ACL 只拦前端 IPC 调用；Rust 侧 `app.opener()...` / `app.clipboard()...` 不经过校验。动作在自家 Rust 命令里执行时，`capabilities` 不是网关，只需要 Cargo 依赖 + 注册插件。
