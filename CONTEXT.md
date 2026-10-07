# WOM

WOM is a Tauri desktop launcher: it lives in the system tray, is toggled by a global shortcut, and shows one search panel.

## Language

### Configuration

**Config**:
The configuration persisted on disk and used at runtime; the single source of truth for the app's behaviour, and the whole of what the config window reads and writes.
_Avoid_: settings, ConfigData, ConfigSettings, ConfigFile, Editable Config

### Search and items

**Item**:
A single piece of content that search can find and that carries a name, a description, keywords, and an action; it is the smallest unit of both searching and acting.
_Avoid_: entry, record, result

**Plugin Item**:
An Item a plugin registers with the framework. It is plain data—a priority, keywords, a name, a description, and a type name the framework never interprets—and it carries the ordered Plugin Actions it offers.
_Avoid_: plugin entry, plugin record

**Item Handle**:
The identity of something a Plugin Action can be addressed to: the plugin it came from plus its place within that plugin—a Plugin Item's registration number or a Plugin Search Result's row index. It is what a Plugin Action is addressed by, so an item can be acted on without knowing its position in the loaded set.
_Avoid_: id, item_id, key

**Plugin Framework**:
The layer that registers plugins, searches the Items they provide, projects them for the frontend, and routes a Plugin Action back to the plugin that registered it. It owns no item of its own and knows no content type.
_Avoid_: plugin host, plugin manager, plugin runtime

**Plugin Context**:
The narrow interface through which a plugin reaches the host—writing logs, resolving a path, reporting the outcome of an action. It is what keeps a plugin independent of Tauri.
_Avoid_: host API, plugin API

**Plugin Action**:
One of the things a Plugin Item can be made to do, registered by its plugin along with the label key the frontend resolves. A Plugin Item carries the ordered list of the Plugin Actions it offers.
_Avoid_: handler, callback, trigger

**Plugin Folder** (插件目录):
The directory shipped outside the binary that holds Plugin Packages; it is what `bundle.resources` delivers and what the host scans at startup, so packages can be added after the app is built.
_Avoid_: plugins dir, plugin path, plugin root

**Plugin Package** (插件包):
One plugin in the Plugin Folder: a directory holding a `manifest.json` that declares its `type` (`js` or `html`), a JS entry or an `html` page. The manifest's `id` is its identity, not the directory name.
_Avoid_: plugin, plugin script, plugin module

**Frontend Plugin** (前端插件):
A Plugin Package whose manifest declares `type: html` and an `html` page instead of a JS entry: it ships no plugin code, and acting on its Plugin Item opens that page directly in a window.
_Avoid_: html plugin, page plugin, static plugin

**Plugin Search** (插件搜索):
The act of asking a Plugin Package for Plugin Search Results; it is the plugin's own search, run when its Plugin Item is acted on, and the framework never performs it.
_Avoid_: plugin query, plugin lookup, dynamic search

**Plugin Search Result** (插件搜索结果):
A row a Plugin Package returns from a Plugin Search. It is not a Plugin Item: the framework does not hold it and it is computed per query, though the framework draws its row.
_Avoid_: plugin item, search hit, sub item

**Plugin Search Page** (插件搜索页):
The second result page that holds the Plugin Search Results of one Plugin Package; the framework draws its rows, and ESC or backspace returns to the main list.
_Avoid_: sub list, plugin list, second list

### Launcher plugin

**Launcher**:
The content types, persistence, and actions of the launcher plugin: system commands, commands, web pages, and scanned paths. It is the first plugin, and the only built-in one.
_Avoid_: default plugin, core plugin

**Launcher Item Source**:
One line of the launcher plugin's manifest, as read and written by its persistence layer; it is what a Launcher Item is built from.
_Avoid_: manifest line, source record

**Item Display**:
The projection of an Item rendered in the list, holding its type, name, and description, and carrying its Item Handle so an Item Action can be addressed without the list row.
_Avoid_: ItemView, display model, item DTO

### Main window interaction

**Selection**:
The Item in the search results that the main window is currently aimed at; ↑/↓ and the wheel move it and Enter acts on it.
_Avoid_: 高亮, focus, cursor, 当前项

**Preview**:
The side panel of the main window that shows the Selection's extended content—for most Items it is the Selection in full, and for a Plugin Item it is that package's Plugin Search Page; ⇧+Enter toggles it and ESC closes it.
_Avoid_: detail view, panel, 预览页面

**Item Action**:
One of the things an Item can be made to do; an Item carries the list of Item Actions it offers in priority order.
_Avoid_: command, operation, verb, 触发动作

**Current Action**:
The Item Action an Item is left at, remembered per Item; it starts as the Item's first, its default, and it is the one the list row shows. ←/→ and ⇧+wheel move it through that Item's list, Enter runs the Selection's, and the Hint Bar names the Selection's.
_Avoid_: selected action, primary action, 当前动作

**Item Index**:
The position of an Item in the set the framework holds; it rides along with the Item Display as paging bookkeeping, and is not how an Item Action is addressed.
_Avoid_: id, item_id, key

**List Position**:
The row of the loaded result list that the Selection currently occupies. It is not the Item Index.
_Avoid_: index, position, row number, 列表下标

**Hint Bar**:
The row of key hints at the bottom of the main window, showing what the keys do in the current mode; it is what Tail renders, not Tail itself — Tail is the layout shell that sits beside Head and Body.
_Avoid_: footer, status bar, 底部提示条

### Window appearance

**Window Effect**:
The native OS backdrop material rendered behind the main window's content, chosen from the effects the running platform supports; the window itself is always frameless and transparent.
_Avoid_: 毛玻璃, blur, background color, theme

**Solid Panel**:
The window appearance used when no Window Effect is chosen: a frameless panel that draws its own Edge Border and background.
_Avoid_: none, default window, plain window

**Edge Border**:
The 1px hairline stroke along the outer edge of the window panel.
_Avoid_: outline, stroke, divider, 边框

**Window Effect Downgrade**:
Substituting the next supported Window Effect when the selected one is unavailable on the running platform, leaving the stored preference untouched.
_Avoid_: fallback, override, auto-fix
