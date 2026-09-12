# Window Effect 配置存原生效果名，应用时按平台解析降级

`config.json` 里的 Window Effect 存的是各平台的原生效果名（`Mica` / `Acrylic` / `Vibrancy`），而不是 `Translucent` 这类抽象意图：配置界面要说清用户选的是哪种系统材质，Windows 用户看到的选项就该是他认识的 "Mica"。代价是同一份配置换平台后无法直译，因此解析放到应用侧：字段缺失（未选择）取当前平台的推荐值，选中的效果在当前系统版本不可用时沿能力链降级到下一个可用项，用户的选择本身不被改写，换回支持的平台仍然生效。无原生效果的 Solid Panel 见 ADR-0002；macOS 26 的 Liquid Glass 暂不提供（原因见 `window_effect.rs` 的 todo）。
