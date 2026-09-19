# 合并 Config 与 EditableConfig

配置只有一个类型：`Config` 既是 `config.json` 的格式，也是运行时配置。读写命令 `fetch_config` / `set_config` 直接收发它，不再另建只含可编辑字段的 DTO。

落地：`Config`（`src-tauri/src/configs.rs`）；写路径经 `parse_full_config` 与 `Config::validate` 校验，读盘继续容忍未知键与缺失字段。作废 ADR-0004。
