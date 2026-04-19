# NextTabletDriver — AI Context File

## 1. Project identity

NextTabletDriver (NextTD) is a cross-platform (Windows + Linux), low-latency USB tablet driver with an `egui` GUI. It replaces vendor drivers for drawing tablets (Wacom, XP-Pen, Huion, VEIKK, etc.) with a unified pipeline optimized for osu! and digital art. Single binary, no async runtime, no database — purely synchronous with dedicated OS threads.

## 2. Architecture overview

Single crate (`next_tablet_driver`), edition 2024, Rust 1.93.1. No workspace.

| Module | Responsibility |
|---|---|
| `drivers` | HID detection, USB init sequences, vendor-specific packet parsers |
| `drivers::parsers` | `ReportParser` trait impls per vendor (wacom, xp_pen, huion, veikk, fallback) |
| `core::config::models` | All serde config structs (`MappingConfig` is the root) |
| `core::math::transform` | Coordinate math: physical→normalized→screen, rotation, relative deltas |
| `engine::tablet_manager` | Background thread: detect→poll→parse→pipeline→inject loop |
| `engine::pipeline` | Per-packet processing: normalize→filter→project→inject |
| `engine::injector` | OS event injection (`SendInput`/`enigo` on Windows, `uinput`/`evdev` on Linux) |
| `engine::state` | `SharedState` — `RwLock`/`AtomicU32` bridge between engine thread and GUI |
| `filters` | `Filter` trait + `FilterPipeline`; impls: `DevocubAntichatter`, `SpeedStatsFilter` |
| `app` | `TabletMapperApp` (eframe::App), lifecycle, auto-updater, WebSocket server |
| `ui` | egui panels (Output, Filters, PenSettings, Console, Settings, Release), theme, components |
| `settings` | JSON persistence to `ProjectDirs` (`last_session.json` + named presets) |
| `startup` | Platform autostart registration (Windows `.lnk` / Linux `.desktop`) |
| `logger` | Custom `log::Log` impl → in-memory ring buffer (500 entries) + debug.log file |

**Dependency flow**: `main` → `app` → `engine` + `ui` → `drivers` + `filters` + `core`. No circular deps.

## 3. Core domain model

- **`TabletData`** (`drivers/mod.rs`): Canonical per-packet struct (x/y as raw `u16`, pressure, tilt, buttons, timestamps). Central to all data flow.
- **`MappingConfig`** (`core/config/models.rs`): Root config struct serialized to `settings.json`. Every field has `#[serde(default)]` for forward-compatible deserialization.
- **`ActiveArea`** / **`TargetArea`**: Tablet physical area (mm) and screen pixel area. `ActiveArea.x/y` are **center** offsets, not top-left.
- **`TabletConfiguration`** (`drivers/config.rs`): JSON schema (PascalCase serde) from OpenTabletDriver format. Loaded at startup from embedded + on-disk `tablets/` dir.
- **`NextTabletDriver`** trait (`drivers/mod.rs`): Hardware abstraction — `get_specs()`, `get_physical_specs()`, `parse()`.
- **`ReportParser`** trait (`drivers/parsers/mod.rs`): Vendor-specific byte→`TabletData` conversion. Dispatched by `create_parser()` matching on OTD class name strings.
- **`Filter`** trait (`filters/mod.rs`): `process(x, y, config) → (x, y)` in normalized UV space. Must be `Send + Sync`.
- **`SharedState`** (`engine/state.rs`): Thread bridge between engine (1000Hz) and GUI (60Hz). See §4 Config propagation for the version counter pattern.
- **`Pipeline`** (`engine/pipeline.rs`): Stateful per-packet processor. Tracks last position for relative mode.
- **`Injector`** (`engine/injector.rs`): Platform-specific, re-exported via `pub use platform::Injector`. Tracks button state to avoid event spam.

## 4. Critical conventions

- **Error handling**: No `anyhow`/`thiserror`. Errors are `Result<(), String>` or `Result<T, Box<dyn Error>>`. Logging via `log` crate with **named targets** (e.g., `log::info!(target: "TabletManager", ...)`). Always use a target string.
- **No async**: Zero async code. Threading via `std::thread::spawn`. Channels via `crossbeam-channel`. No runtime.
- **Platform abstraction**: `#[cfg(windows)]` / `#[cfg(target_os = "linux")]` with inner `mod platform { ... }` + `pub use platform::*` re-export pattern. Used in `injector.rs`, `startup.rs`, `main.rs`.
- **Config propagation**: UI writes to `SharedState.config` via `RwLock`, bumps `config_version` (AtomicU32). Engine thread polls version every 50ms, clones config on change. Never hold the write lock from the engine thread.
- **Coordinate spaces**: Raw hardware units → millimeters → normalized UV `[0.0, 1.0]` → screen pixels. `physical_to_normalized` does NOT clamp; `normalized_to_screen` DOES clamp.
- **Tablet configs**: JSON files in `tablets/` dir use **PascalCase** keys (OTD format). Disk configs override embedded (by name dedup). Init reports are **base64-encoded** byte arrays.
- **Naming**: Modules use snake_case. Log targets use PascalCase (e.g., `"TabletManager"`, `"Detect"`). The logger whitelist in `logger.rs` must be updated when adding new targets.
- **Forbidden**: Never `unwrap()` on `RwLock` in the engine hot path without understanding contention. Never block the engine thread with I/O. Never use `async`.

## 5. External dependencies map

| Dependency | Role | Owner module |
|---|---|---|
| `hidapi` | USB HID device detection and raw packet I/O | `drivers`, `engine::tablet_manager` |
| `eframe`/`egui` | Immediate-mode GUI framework | `app`, `ui` |
| `enigo` | Mouse event injection (Windows only) | `engine::injector` |
| `evdev` | uinput virtual device creation (Linux only) | `engine::injector` |
| `windows-sys` | Win32 FFI (mutex, timer, thread priority, cursor) | `main.rs`, `engine`, `app::lifecycle` |
| `serde`/`serde_json` | Config serialization (JSON) | `core::config`, `settings`, `drivers::config` |
| `crossbeam-channel` | Lock-free MPSC channels (tablet data, updates) | `engine::tablet_manager`, `app` |
| `tungstenite` | WebSocket server for streaming overlays | `app::websocket`, `filters::stats` |
| `include_dir` | Embeds `tablets/` JSON configs into binary at compile time | `drivers` |
| `catppuccin-egui` | Theme presets | `ui::theme` |
| `tray-icon` | System tray integration | `app::lifecycle` |
| `rfd` | Native file dialogs for profile import/export | `ui::panels` |
| `ureq` | HTTP client for auto-update checks (GitHub API) | `app::autoupdate` |

## 6. Data flow (absolute mode, one packet)

1. `tablet_manager::run_manager` → `device.read_timeout(&mut buf, 1000)` (blocking HID read)
2. `driver.parse(&buf[..len])` → vendor `ReportParser` → `TabletData { x, y, pressure, ... }`
3. `pipeline.process()`:
   - `(x as f32 / max_w) * phys_w` → `x_mm`, `y_mm`
   - `transform::physical_to_normalized(x_mm, y_mm, area...)` → `(u, v)` in `[0,1]`
   - `filters.process(u, v, config)` → filtered `(u, v)` (antichatter, speed stats)
   - `transform::normalized_to_screen(u, v, target...)` → `(screen_x, screen_y)`
   - `injector.move_absolute(screen_x, screen_y, u, v)` → OS cursor move
   - `injector.set_left_button(pressure > threshold)` → OS click
4. `tablet_sender.send(data)` → GUI thread receives via `crossbeam` channel

## 7. Test infrastructure

- **Run**: `cargo test` (no special harness). Tests compile on both Windows and Linux.
- **Unit tests**: Inline `#[cfg(test)] mod tests` in `transform.rs`, `antichatter.rs`, `config.rs`, `models.rs`.
- **Test config**: Always use `MappingConfig::default_test()` — it provides deterministic, self-consistent defaults. Never construct `MappingConfig` manually in tests.
- **Writing a valid test**: Create concrete structs (no mocking framework). For filter tests, instantiate the filter, build a config via `default_test()`, mutate only the fields under test, and call `process()` directly. For transform tests, call the pure functions with known inputs and assert with `< 1e-6` tolerance for floats.
- **No integration tests directory**. No CI currently visible (`.github/` exists but not inspected).

## 8. Known constraints and gotchas

1. **`ActiveArea.x/y` are center offsets**, not top-left corner. Misinterpreting this breaks all coordinate math.
2. **`SharedState` uses `RwLock`, not `Mutex`**. The engine thread reads config; the GUI writes it. Deadlocks possible if both sides write without care.
3. **Parser dispatch** (`create_parser`) matches on C# class name substrings from OTD JSON. Adding a new parser requires updating both the match chain and the `parsers/` module tree.
4. **Logger target whitelist**: New log targets won't appear in the in-app console unless added to the `allowed_targets` array in `logger.rs`.
5. **Windows absolute injection** actually uses relative deltas from `GetCursorPos` to avoid DPI scaling issues — not true `SendInput` absolute mode.
6. **Linux requires `/dev/uinput`** permissions. The injector panics (`expect`) if uinput is unavailable.
7. **Thread priority**: Engine thread sets `TIME_CRITICAL` on Windows, `nice(-11)` on Linux. This is intentional for latency but can starve other processes.
8. **No graceful shutdown**: Background threads (`run_manager`, WebSocket, tray listener) run in infinite loops with no cancellation tokens.
9. **`tablets/` JSON configs are OpenTabletDriver-compatible** but only a subset of parsers are implemented. Unknown parsers fall through to `FallbackParser`.
10. **Edition 2024 with let-chains**: The codebase uses `if let ... && let Ok(...)` syntax (stabilized in edition 2024). Older Rust toolchains will fail.

## 9. Active work context

> **⚠️ Fill this section before every AI session.** Even two lines prevent the model from wasting tokens on broad exploration.

- **Current feature / task**: General maintenance — no specific feature in progress.
- **Files in scope**: Any.
- **Known issues**: None active.
- **Do not touch**: `tablets/` JSON configs (OTD upstream format), `build.rs`.
