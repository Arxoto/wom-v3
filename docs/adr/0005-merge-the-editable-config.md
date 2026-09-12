# 合并 Config 与 EditableConfig

可编辑配置与持久化文件的字段完全一致（12 个字段一一对应，`editable_config_mirrors_every_config_field` 一度专门盯着这件事），再维持两个结构体就只剩"改一个字段要改两处"的成本，没有换来任何隔离——真正的约束是"写盘必须整份回传"。所以合并成一个 `Config`：读写命令 `fetch_config` / `set_config` 直接收发它；键集合与值都在写路径上校验（`parse_full_config` 拒绝漏字段或多字段，`Config::validate` 拒绝越界的值），读盘继续容忍未知键与缺失字段。代价是以后 Config 里出现不能让用户改的字段时，要么重新拆出 DTO，要么在写路径上单独排除。
