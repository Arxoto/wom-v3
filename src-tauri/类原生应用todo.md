
现状是：set_near_native 只有两条（禁用右键、禁用 Alt 菜单栏），Rust 侧 src-tauri/src/window_utils.rs:62 已经吃掉了透明/框架/阴影/任务栏/置顶/尺寸锁定/devtools/缩放快捷键。缺口按「能不能放 Rust」分三档。

## 一、Rust 侧（前端做不到，或做得不彻底）

1. 单实例（最像原生、最便宜的一条）
现在第二次启动会起第二个进程、多一个托盘图标。挂 tauri-plugin-single-instance，回调里直接 show_main_window，行为就和 Spotlight/Alfred 一致了。

2. 关掉 WebView 的浏览器行为总开关
window_utils.rs:95 那句「禁用快捷键（没有优雅实现，放开限制）」其实是有优雅实现的——Windows 上把 CoreWebView2Settings调一遍就够，比前端逐键 preventDefault 干净得多：

- AreBrowserAcceleratorKeysEnabled(false)：一次干掉 Ctrl+R / F5 / Ctrl+P / Ctrl+F / Ctrl+Shift+I / Ctrl+U
- IsZoomControlEnabled(false) + IsPinchZoomEnabled(false)：Ctrl+滚轮和触控板捏合缩放（zoom_hotkeys_enabled 只管 Ctrl±）
- IsSwipeNavigationEnabled(false)：触摸板左右滑＝历史后退，滑一下就白屏
- IsStatusBarEnabled(false)：悬停链接左下角冒出的网址条
- IsGeneralAutofillEnabled(false) / IsPasswordAutosaveEnabled(false)：输入框的自动填充下拉
- AreDefaultContextMenusEnabled(false)：比前端 preventDefault 更彻底（输入框上的原生菜单也一起没）
- IsSmoothScrollingEnabled(false)：可选，和现有逐行下翻的手感更配

macOS 上对应的是 allowsMagnification = false、allowsBackForwardNavigationGestures = false、allowsLinkPreview = false。
两边都要走 with_webview，需要引入 webview2-com / objc2-web-kit 依赖（window_effect.rs 顶上那个 Liquid Glass todo 已经因为同样理由动过 objc2-app-kit，这笔依赖账要一起算）。

3. on_navigation 白名单
只放行自己的 url，其余交给 opener 丢给系统浏览器。否则点一个链接面板就地变成浏览器：回不去、后退手势又刚被你关掉，只能重启。

4. 别闪白
现在 create_main_window 直接 visible(true)，webview 首帧必然是白底，视觉上完全是网页。做法：建窗时 visible(false) + background_color（app.windows[].backgroundColor / builder 方法），
on_page_load 或前端准备好以后再 show()。原生应用从不闪白。

5. CloseRequested → 隐藏
Framed 有原生标题栏，点 X 现在是真销毁窗口（靠 RunEvent::ExitRequested 兜住不让进程退）。原生常驻应用点 X 只是隐藏，退出只走托盘 Quit。顺带把第 6 条救了。

6. 记住窗口位置
create_main_window 每次都 .center()；rebuild_main_window 里那个 keep_position 只在进程内有效。原生应用会记住上次位置，落盘到 config 即可。

7. 平台身份
macOS：set_activation_policy(ActivationPolicy::Accessory)，不进 Dock（launcher 的正确身份），再配 visible_on_all_workspaces(true) 跨 Space。
Windows 侧 skip_taskbar 现在只在非 Framed 时生效，如果目标是"永远不进任务栏"，可以统一成 true。

8. drag_drop_enabled(false)
不打算接文件拖放的话，直接把 webview 的拖放关掉，省掉"拖个文件进来页面被导航到 file://"和默认的复制光标。

9. 视设计而定
theme(Theme::Light)：配色固定浅色的话建议钉住，否则系统暗色时原生滚动条、表单控件、color-scheme 会自己变暗，跟手写 CSS 打架——不过 Head.css 里有 prefers-color-scheme 分支，说明你可能本来就想跟随系统，那就保持现状。
Windows 11 的 DWM 圆角（DWMWA_WINDOW_CORNER_PREFERENCE）可选，但和 ADR-0003 放弃自绘阴影的立场要一起对齐。

## 二、前端 set_near_native（DOM 事件与 CSS，只能在前端）

Rust 那套开关只在 Windows 上的 WebView2 生效，所以前端仍需要一份跨平台兜底：

- 拦 F5 / Ctrl+R / Ctrl+Shift+R / Ctrl+P / Ctrl+S：刷新会把面板打回初始态，比任何一处闪白都伤
- 拦 dragstart，再配 img, a { -webkit-user-drag: none } 和 draggable={false}：现在从面板拖图片或链接会留 ghost image，拖到桌面还能生成快捷方式
- 拦鼠标中键：Windows 上会出现自动滚动的圆点光标
- CSS 收口：body { cursor: default }、可点元素 pointer，满屏 I 形光标是"网页感"的主要来源；overscroll-behavior: none 挡 macOS 橡皮筋和下拉刷新光晕；::selection 用系统强调色
- core.css:47 的 * { user-select: none } 建议收窄到非交互容器，给 input/textarea 显式放开 user-select: text：Blink/WebKit 里这条会波及输入框内部编辑器，双击选词、拖选都会被吃掉
- 可编辑元素补 autocomplete="off" / autocorrect="off" / autocapitalize="off"：Head.tsx 已经关了 spellCheck，但 macOS 的自动纠正红线还留着，配置页的输入框也该一起补

IME 那条已经处理干净了（useMainInteraction.ts:172 的 isComposing + keyCode 229），不用动。

## 三、感知层（不属于 set_near_native，但决定"像不像原生"）

- 显示/隐藏动效：Spotlight 有 scale+fade。EVENT_MAIN_SHOWN 事件已经有了，窗口本身是透明的，前端完全能在窗口内做 6–8 帧的 opacity/transform 动画；隐藏侧同理，现在 Rust 直接 hide()，要淡出得让前端播完再调 dismiss_main_window。
- 拖动区域：data-tauri-drag-region 现在只挂在 Head.tsx:66 那个图标上（宽度 0.8 * head-h），Solid 无边框时整条 head 都该能拖。挂到 .head-box 上即可，注意别盖住输入框。
