# GEMINI.md — next_tablet_driver

## Project Overview

**next_tablet_driver** is a cross-platform tablet driver written in Rust, targeting **Windows** and **Linux**.
It is designed for use with **Osu!** and general drawing applications, providing low-latency HID input handling, a graphical configuration UI, and WebSocket-based communication.

- **Author:** iSweat
- **Version:** 1.0.0
- **Rust Edition:** 2024
- **Minimum Rust Version:** 1.93.1

---

## Architecture Summary

| Layer | Role |
|---|---|
| HID Input | Raw tablet data via `hidapi` |
| Input Emulation | Mouse/keyboard injection via `enigo` |
| GUI | Immediate-mode UI via `eframe` / `egui` |
| Config | JSON persistence via `serde` + `serde_json` + `directories` |
| IPC / Remote | WebSocket server via `tungstenite` |
| Updates / API | HTTP client via `ureq` |
| Tray | System tray icon via `tray-icon` |
| Theming | Catppuccin theme via `catppuccin-egui` |
| Logging | Structured logging via `log` + `chrono` |

---

## Dependencies

### Cross-platform (`[dependencies]`)

| Crate | Version | Purpose |
|---|---|---|
| `hidapi` | 2.6.3 | Raw HID device communication (tablet input) |
| `eframe` | 0.33.3 | GUI framework (egui backend) |
| `egui_extras` | 0.33.3 | Additional egui widgets |
| `catppuccin-egui` | 5.7.0 | Catppuccin color theme for egui 0.33 |
| `display-info` | 0.5.0 | Screen resolution / monitor info |
| `enigo` | 0.6.1 | Cross-platform mouse/keyboard emulation |
| `serde` | 1.0 | Serialization framework (with `derive`) |
| `serde_json` | 1.0 | JSON serialization/deserialization |
| `base64` | 0.22.1 | Base64 encoding/decoding |
| `directories` | 6.0.0 | OS-specific config/data directories |
| `rfd` | 0.17.2 | Native file dialogs |
| `log` | 0.4 | Logging facade |
| `chrono` | 0.4 | Date/time utilities |
| `crossbeam-channel` | 0.5.15 | Multi-producer multi-consumer channels |
| `include_dir` | 0.7 | Embed entire directories at compile time |
| `tungstenite` | 0.29.0 | WebSocket client/server |
| `ureq` | 2.12.1 | Lightweight HTTP client (with JSON support) |
| `tray-icon` | 0.22.0 | System tray icon |
| `image` | 0.25.10 | Image loading (PNG only, minimal features) |

### Windows only (`[target.'cfg(windows)'.dependencies]`)

| Crate | Version | Purpose |
|---|---|---|
| `windows-sys` | 0.61.2 | Win32 API bindings (threading, security, input, media) |

**Enabled Win32 features:**
- `Win32_Foundation`
- `Win32_System_Threading`
- `Win32_Security`
- `Win32_UI_WindowsAndMessaging`
- `Win32_UI_Input_KeyboardAndMouse`
- `Win32_Media`

### Windows build dependencies (`[target.'cfg(windows)'.build-dependencies]`)

| Crate | Version | Purpose |
|---|---|---|
| `winres` | 0.1 | Embed Windows resources (icons, version info) into the binary |

### Linux only (`[target.'cfg(target_os = "linux")'.dependencies]`)

| Crate | Version | Purpose |
|---|---|---|
| `evdev` | 0.13 | Linux input event device interface |
| `nix` | 0.29 | Unix syscall bindings (`sched`, `fs` features) |
| `libc` | 0.2 | Raw C bindings |

---

## Build Profiles

### Release (`[profile.release]`)

| Option | Value | Effect |
|---|---|---|
| `opt-level` | `"z"` | Optimize for **binary size** (smallest output) |
| `lto` | `true` | Link-time optimization (cross-crate inlining) |
| `codegen-units` | `1` | Single codegen unit for maximum optimization |
| `panic` | `"abort"` | No unwinding on panic (smaller binary, faster) |
| `strip` | `true` | Strip debug symbols from the final binary |

> **Note:** The release profile prioritizes **small binary size** and **performance** over compile time.

---

## Platform Notes

### Windows
- Requires the `build.rs` script using `winres` to embed application resources.
- Uses `windows-sys` for low-level Win32 calls (input simulation, thread priorities, multimedia timers).
- Tablet input handled via `hidapi`.

### Linux
- Tablet input can be handled via `hidapi` **or** `evdev` kernel input events.
- `nix` is used for CPU scheduler (`sched`) and filesystem (`fs`) operations (e.g., setting real-time thread priority).
- May require `udev` rules or running as root/with elevated permissions to access `/dev/input/` devices.

---

## Common Development Commands

```bash
# Build (debug)
cargo build

# Build (release — optimized & stripped)
cargo build --release

# Run (debug)
cargo run

# Run (release)
cargo run --release

# Check without building
cargo check

# Run tests
cargo test

# Check for issues (lints)
cargo clippy -- -D warnings

# Format code
cargo fmt

# Show dependency tree
cargo tree

# Audit dependencies for vulnerabilities
cargo audit
```

---

## Code Conventions

- **Edition:** Rust 2024 — use modern idioms (`use` in `let`, `async` closures, etc.)
- **Error handling:** Prefer `Result`/`Option` with `?` propagation; avoid `unwrap()` in library/driver code.
- **Logging:** Use the `log` crate macros (`log::info!`, `log::warn!`, `log::error!`) — never `println!` in production paths.
- **Serialization:** All persistent config structs must derive `serde::Serialize` + `serde::Deserialize`.
- **Channels:** Use `crossbeam-channel` for inter-thread communication between the HID input thread and the GUI thread.
- **Platform guards:** Wrap platform-specific code in `#[cfg(windows)]` / `#[cfg(target_os = "linux")]` blocks.

---

## Key Architecture Patterns

### Input → GUI pipeline

```
HID Thread (hidapi / evdev)
    │
    │  crossbeam-channel
    ▼
Processing Thread (enigo input emulation)
    │
    │  crossbeam-channel
    ▼
GUI Thread (eframe / egui)
```

### Config persistence

- Config is serialized to JSON via `serde_json` and stored in the OS config directory resolved by `directories`.
- Images/assets embedded at compile time via `include_dir`.

### WebSocket

- `tungstenite` provides a WebSocket server for external tools (e.g., overlays, companion apps) to connect and receive tablet state.

---

## File Structure (expected)

```
next_tablet_driver/
├── build.rs               # winres resource embedding (Windows)
├── Cargo.toml
├── GEMINI.md
├── src/
│   ├── main.rs            # Entry point
│   ├── driver/            # HID/evdev input handling
│   ├── gui/               # eframe/egui UI code
│   ├── config/            # serde config structs & persistence
│   ├── websocket/         # tungstenite server
│   ├── platform/          # #[cfg] platform-specific code
│   │   ├── windows.rs
│   │   └── linux.rs
│   └── utils/             # Shared utilities
└── assets/                # Embedded via include_dir
```

---

## Notes for AI Assistance

- Always respect `#[cfg(windows)]` / `#[cfg(target_os = "linux")]` guards — do not suggest cross-platform code that breaks platform separation.
- The `eframe`/`egui` version is **0.33.x** — use the API for that version specifically (not 0.29 or 0.31).
- `catppuccin-egui` is configured with `default-features = false` and `features = ["egui33"]` — do not suggest the default feature set.
- `image` is configured with `default-features = false, features = ["png"]` — only PNG is supported.
- `ureq` is `2.x`, not `3.x` — the API differs significantly between major versions.
- The release profile targets **binary size** (`opt-level = "z"`) — do not suggest changes that increase binary size without good reason.
