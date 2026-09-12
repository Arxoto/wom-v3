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
The projection of an Item rendered in the list, holding only its type, name, and description.
_Avoid_: ItemView, display model, item DTO

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
