# Window Effect 配置存原生效果名，应用时按平台解析降级

`config.json` 的 Window Effect 字段存各平台的原生效果名（`Mica` / `Acrylic` / `Vibrancy`），不存 `Translucent` 这类抽象意图。字段缺失（未选择）取当前平台的推荐值；选中的效果在当前系统不可用时，沿能力链降级到下一个可用项；用户的选择本身不被改写。

落地：`WindowEffect` 与 `recommended()` / 可用性解析链在 `src-tauri/src/window_effect.rs`，`Config` 只存这个值。无原生效果的形态见 ADR-0003。
