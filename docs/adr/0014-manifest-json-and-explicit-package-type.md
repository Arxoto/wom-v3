# 清单改用 `manifest.json`（serde 反序列化），包的形态由显式 `type` 声明

ADR-0011 定的清单是手写的 `key: value` 行格式（`manifest.yml`）：字段是平的、数量固定，
仓库里也没有 YAML 依赖，所以"自己解析"当时是划算的。这一轮换成 `manifest.json`，
直接 `serde_json::from_str` 到 `PackageManifest`，手写解析器整个删掉。

两条理由：

1. 手写解析器要把语法自己扛一遍（行号、注释、空行、值与空白），而这些 serde 都做完了，
   报错还自带行列号；字段一多，自写的错误分支只会越堆越多。
2. `serde_json` 本来就是直接依赖（配置文件那一侧在用），这里没有新增任何依赖。

## 包的形态由 `type` 显式声明

清单新增必填字段 `"type": "js" | "html"`。ADR-0013 当时靠"`html` 非空就是前端插件"判形态，
两个字段同时存在时谁赢只能靠约定，说不清。现在由 `type` 决定：

- `"type": "js"`：JS 插件，读 `entry`（缺省 `index.js`）；
- `"type": "html"`：前端插件，读 `html`，条目直接打开这一页。

`type` 缺失、`type: html` 却没有 `html`、`keywords` 为空、`id` 不是稳定 ASCII，都是这个包的
错误：调用方记一条 warn 后跳过这个包，应用照常启动（ADR-0011 的隔离语义不变）。

## 代价

- 老包要改一次：`manifest.yml` 手工换成 `manifest.json` 并补上 `type`。仓库里只有探针包，
  一起换了；`.scratch/` 里的 spec 与 ADR-0011 是历史记录，不改，以本 ADR 为准。
- 未知键照旧忽略（serde 的默认行为）：清单里多写一个键不是这个包坏掉的理由。
- 清单从此对语法敏感（引号、逗号、尾随逗号），写错由 serde 报错并指出行列号。
