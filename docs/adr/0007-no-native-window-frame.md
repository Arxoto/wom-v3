# 主窗口不做原生框架

主窗口一律透明、无原生框架（`decorations(false)`），`WindowEffect` 里带原生框架的那个形态（`Framed`）删除。它的用途只是把无原生效果的外观当普通窗口用，而面板的尺寸、发丝线、置顶与拖动都按无框架写，多这一种形态就多一条要跟着布局一起维护的路。配置里残留的 `"Framed"` 按未知枚举名处理（见 ADR-0004），退回未选择、跟随平台推荐值。

落地：`WindowEffect`（`src-tauri/src/window_effect.rs`）与主窗口创建（`src-tauri/src/window_utils.rs`）；Solid Panel 也只剩无框架这一种。作废 ADR-0003 里 Solid / Framed 的拆分，其余不变（阴影只用系统原生阴影、窗口尺寸只为 1px 发丝线留 2px）。
