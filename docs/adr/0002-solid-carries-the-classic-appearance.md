---
Status: superseded by ADR-0003
---

# 经典外观开关并进 WindowEffect 的 Solid 变体

`custom_shadow`（自绘阴影）和 `window_frame`（原生框架）只在没有 Window Effect 时有意义：一旦选了原生效果，窗口就必须透明、无原生框架、无自绘阴影，这两个开关只能为 false，三者在配置里各自独立就会出现互相矛盾的组合。所以合并成一个值——`WindowEffect::Solid { custom_shadow, window_frame }` 表示"不使用原生效果"的那条路，代码通过 `custom_shadow()` / `window_frame()` / `transparent()` 取值，原生效果一律返回 false / false / true。代价是 `Solid` 在 JSON 里是结构体形式（`{"Solid": {"custom_shadow": true, "window_frame": false}}`）而其他变体是字符串，并且旧配置里平铺的两个字段只能作为一次性的迁移输入读取、不能再写出。
