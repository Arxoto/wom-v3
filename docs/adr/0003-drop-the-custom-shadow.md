---
Status: superseded by ADR-0007 for the Solid / Framed split
---

# 去掉自绘阴影，阴影一律用系统原生阴影

窗口阴影只用系统原生阴影，自绘阴影那条路径（`custom_shadow` 键与 `index_frame.html`）删除。不启用原生效果的外观拆成 `Solid`（无原生框架）与 `Framed`（带原生框架），窗口尺寸只为 1px 发丝线留 2px。

落地：`WindowEffect`（`src-tauri/src/window_effect.rs`）。`custom_shadow` 已不是配置字段，旧文件里的该键按未知键忽略。作废 ADR-0002。
