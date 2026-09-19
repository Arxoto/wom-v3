# WOM

WOM is a Tauri desktop launcher: it lives in the system tray, is toggled by a global shortcut, and shows one search panel.

## Language

### Configuration

**Config**:
The configuration persisted on disk and used at runtime; the single source of truth for the app's behaviour, and the whole of what the config window reads and writes.
_Avoid_: settings, ConfigData, ConfigSettings, ConfigFile, Editable Config

### Search and items

**Item**:
A single piece of built-in content that search can find and that carries a name, a description, keywords, and an action; it is the smallest unit of both searching and acting.
_Avoid_: entry, record, result

**Item Display**:
The projection of an Item rendered in the list, holding its type, name, and description, and carrying its Item Index so an Item Action can be addressed without the list row.
_Avoid_: ItemView, display model, item DTO

### Main window interaction

**Selection**:
The Item in the search results that the main window is currently aimed at; ↑/↓ move it and Enter acts on it.
_Avoid_: 高亮, focus, cursor, 当前项

**Preview**:
The side panel of the main window that shows the Selection in full; ⇧+Enter toggles it and ESC closes it.
_Avoid_: detail view, panel, 预览页面

**Item Action**:
One of the things an Item can be made to do; an Item carries the list of Item Actions it offers in priority order, and Enter runs the first — its default, the one the list row shows.
_Avoid_: command, operation, verb, 触发动作

**Item Index**:
The position of an Item in the set loaded from the settings file; it rides along with the Item Display and addresses that Item when running an Item Action.
_Avoid_: id, item_id, key

**List Position**:
The row of the loaded result list that the Selection currently occupies. It is not the Item Index.
_Avoid_: index, position, row number, 列表下标

**Hint Bar**:
The row of key hints at the bottom of the main window, showing what the keys do in the current mode; it is what Tail renders, not Tail itself — Tail is the layout shell that sits beside Head and Body.
_Avoid_: footer, status bar, 底部提示条

### Window appearance

**Window Effect**:
The native OS backdrop material rendered behind the main window's content, chosen from the effects the running platform supports. A Window Effect cannot be combined with the Window Frame.
_Avoid_: 毛玻璃, blur, background color, theme

**Solid Panel**:
The window appearance used when no Window Effect is chosen; it comes in two flavours, one with the native Window Frame and one frameless.
_Avoid_: none, default window, plain window

**Window Frame**:
The native OS window decoration (title bar and system border) of the main window; when absent the window is frameless and keeps only the system shadow.
_Avoid_: 边框, chrome, decorations

**Edge Border**:
The 1px hairline stroke along the outer edge of the window panel.
_Avoid_: outline, stroke, divider, 边框

**Window Effect Downgrade**:
Substituting the next supported Window Effect when the selected one is unavailable on the running platform, leaving the stored preference untouched.
_Avoid_: fallback, override, auto-fix
