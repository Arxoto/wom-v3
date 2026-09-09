# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Run Commands

- **Dev**: `pnpm tauri dev` (or run `build_scripts/run_dev.ps1` which sets up fnm env first)
- **Build**: `pnpm tauri build` (or `build_scripts/run_build.ps1`)
- **Frontend only**: `pnpm dev` (Vite dev server on port 1420), `pnpm build` (typecheck + Vite build)
- **Rust tests**: `cargo test --manifest-path src-tauri/Cargo.toml`
- The project uses **pnpm** (see `package.json` `packageManager` field) and requires fnm for Node version management (see `.node-version`).

## Architecture

WOM is a **Tauri v2** desktop launcher/search tool (similar to Spotlight/Raycast). It runs in the system tray and is toggled via a global shortcut.

### Three entry points (multi-window)

The Rust backend (`lib.rs`) creates two window types:
- **Main window** — uses either `index.html` (transparent, no decorations) or `index_frame.html` (custom shadow variant). The choice depends on the `custom_shadow` config.
- **Config window** — uses `index_config.html`.

Each HTML entry loads a separate React app via its own `src/index_*.tsx`.

### Rust backend (`src-tauri/src/`)

- **`lib.rs`** — App setup: registers plugins (log, opener, global-shortcut, custom inner-plugin), creates tray menu, manages windows, prevents exit on window close (app stays in tray).
- **`configs.rs`** — Configuration stored as JSON in app data dir (`config.json`). Uses `arc_swap::ArcSwap` for lock-free hot-reload. `ConfigData` is the internal representation (derived from `ConfigSettings`). A `layout_coupling` module defines constants shared with the frontend (frame border width, divider heights). Exposes `fetch_layout_config` Tauri command.
- **`global_shortcut.rs`** — Maps config-defined hotkey (modifiers + single char) to show/hide the main window.
- **`inner_plugins/`** — Custom Tauri plugin providing the core search functionality:
  - **`base.rs`** — `ItemType` enum: `System`, `Cmd`, `Snippets`, `Note`, `Web`, `File`, `Scan`. Each item has a `KeyWord` and `ItemDesc` (string or path).
  - **`common.rs`** — `Item` struct: has a monotonic `ItemId`, type, keyword, name, and description. `new_list()` generates multiple items sharing the same ID for different keywords.
  - **`persistence/load.rs`** — Loads items from `inner_plugins.txt` (app data dir). Reads line-by-line, parses each line, handles Scan items by walking the filesystem via `walkdir`.
  - **`persistence/parse_core.rs`** — Line parser using `<->` as field delimiter and space as keyword delimiter. Dispatches to type-specific parsers.
  - **`persistence/parse_impl_common.rs`** — 4-field format: `Type <-> key1 key2 <-> name <-> desc`
  - **`persistence/parse_impl_system.rs`** — 3-field format: `System <-> key1 key2 <-> name`
  - **`persistence/parse_impl_scan.rs`** — 7-field format: `Scan <-> File Dir <-> .txt .md <-> blacklist <-> r <-> home <-> path`
  - **`persistence/scans_helper.rs`** — Filesystem scanner using `walkdir`, resolves base directories (home/desktop/download/etc.), filters by file type, suffix, and blacklist.
  - **`search.rs`** — Four-tier matching algorithm: exact match → prefix match → contains match → subsequence match. Results merged and deduplicated using `bitvec::BitVec` (ID-based dedup, contiguous IDs assumed for efficiency). Results cached in `ItemSearchStat` (a `Mutex`). Exposes `search` and `search_page` Tauri commands. `ItemSearchPage` is the frontend-facing paginated result (100 items per page). Search result lists include split indices so the frontend can render match-type separators.
  - **`plugins.rs`** — Plugin init: loads items on setup, manages `ItemsStat` and `ItemSearchStat` as Tauri managed state. `reload_setting()` clears search cache and reloads items.

### Frontend (`src/`)

- **`core.tsx`** — Shared setup: disables context menu (near-native feel), fetches layout config from Rust via `invoke('fetch_layout_config')`, sets CSS custom properties (`--head-h`, `--tail-h`, `--item-h`).
- **`AppMain.tsx`** — Main window layout: `Box > Static(Head) > DividerTop > Elastic(Body) > DividerBottom > Static(Tail)`. Layout components in `main/Layout.tsx` use flexbox with CSS variables for dimensions.
- **`head.tsx`** — Search input with ghost/suggestion text layer overlaid on a real input.
- **`body.tsx`** — Scrollable item list with optional preview panel.
- **`core.css`** — CSS custom properties for color scheme, scrollbar styling, text selection disabled globally.

### Data flow

1. On startup, Rust loads `inner_plugins.txt` → parses into `Vec<Item>` → stored in `ItemsStat`
2. User types in Head input → frontend calls `invoke('search', { k: query })`
3. Rust runs 4-tier matching → deduplicates via BitVec → caches result → returns first page
4. Scrolling calls `invoke('search_page', { index: N })` for pagination
5. Config changes are persisted to `config.json`, hot-reloaded via tray menu "Reload Global Config"