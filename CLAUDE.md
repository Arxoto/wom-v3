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

### Two entry points (multi-window)

The Rust backend (`lib.rs`) creates two window types:
- **Main window** — uses `index.html` (transparent, no decorations). Window material and native frame come from the `window_effect` config.
- **Config window** — uses `index_config.html`.

Each HTML entry loads a separate React app via its own `src/index_*.tsx`.

### Rust backend (`src-tauri/src/`)

- **`lib.rs`** — App setup: registers plugins (log, opener, global-shortcut, custom inner-plugin), creates tray menu, manages windows, prevents exit on window close (app stays in tray).
- **`configs.rs`** — Configuration stored as JSON in app data dir (`config.json`). `Config` is both the file format and the runtime source of truth; derived values (window size, effective window effect, `show_main_auto`) are methods, not fields. `EditableConfig` is the user-editable subset (`From<&Config>` / `Config::apply`) and is what the frontend reads and writes. Uses `arc_swap::ArcSwap` for lock-free hot-reload, and a `layout_coupling` module for constants shared with the frontend. Commands: `fetch_editable_config`, `fetch_effect_info`, `set_editable_config` (validate → save → reload → recreate the window if the effect changed → re-register the shortcut → emit `config_changed`).
- **`window_effect.rs`** — `WindowEffect` enum (`Solid` / `Framed` / `Mica` / `Acrylic` / `Vibrancy`) plus per-platform availability, the downgrade chain, the tint alpha, and applying the effect to a window through `window-vibrancy`.
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

- **`core.tsx`** — Shared setup: disables context menu (near-native feel), mirrors the Rust config types (`EditableConfig`, `EffectInfo`, `WindowEffect`, `MainWindowMode`), fetches them via `invoke('fetch_editable_config')` / `invoke('fetch_effect_info')`, sets CSS custom properties (`--head-h`, `--tail-h`, `--item-h`, `--color-bg-alpha`), and re-reads them on the `config_changed` event.
- **`AppMain.tsx`** — Main window layout: `Box > Static(Head) > DividerTop > Elastic(Body) > DividerBottom > Static(Tail)`. Layout components in `main/Layout.tsx` use flexbox with CSS variables for dimensions.
- **`head.tsx`** — Search input with ghost/suggestion text layer overlaid on a real input.
- **`body.tsx`** — Scrollable item list with optional preview panel.
- **`core.css`** — CSS custom properties for color scheme, scrollbar styling, text selection disabled globally.

### Data flow

1. On startup, Rust loads `inner_plugins.txt` → parses into `Vec<Item>` → stored in `ItemsStat`
2. User types in Head input → frontend calls `invoke('search', { k: query })`
3. Rust runs 4-tier matching → deduplicates via BitVec → caches result → returns first page
4. Scrolling calls `invoke('search_page', { index: N })` for pagination
5. Config changes are written by `set_editable_config` (config window); the tray menu "Reload Global Config" re-reads `config.json` for hand edits. Both go through `reload_data`, which broadcasts `config_changed` so open windows re-read

## Agent skills

### Issue tracker

Issues and specs live as markdown files under `.scratch/<feature>/` in this repo. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical triage roles, each using its own name as the `Status:` value. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
