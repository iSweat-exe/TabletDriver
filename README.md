# NextTabletDriver ⚡

> A high-performance user-mode drawing tablet driver written in Rust, designed to minimize input lag. A modern alternative to OpenTabletDriver.

![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/github/license/Next-Tablet-Driver/NextTabletDriver?style=for-the-badge)
![Release](https://img.shields.io/github/v/release/Next-Tablet-Driver/NextTabletDriver?style=for-the-badge)
![Stars](https://img.shields.io/github/stars/Next-Tablet-Driver/NextTabletDriver?style=for-the-badge)

---

## ✨ Features

- ⚡ **Ultra-low input lag** — optimized at the driver level for maximum responsiveness
- 🎨 **Drawing & osu! ready** — designed for digital artists and osu! players
- 🖥️ **User-mode driver** — no need to install kernel-level drivers
- 🔌 **HID support** — communicates directly with tablets via HID protocol
- 🌐 **WebSocket support** — real-time communication via `tungstenite`
- 🖱️ **Area mapping** — configure tablet area, screen mapping and more
- 💾 **Persistent settings** — configuration saved locally via `serde_json`
- 🪟 **Windows support** — native Windows integration via `windows-sys`

---

## 📦 Installation

1. Go to the [**Releases**](https://github.com/Next-Tablet-Driver/NextTabletDriver/releases) page
2. Download the latest installer (`.exe`)
3. Run the installer and follow the instructions
4. Launch **NextTabletDriver** and configure your tablet

> ⚠️ Windows only for now.

---

## 🛠️ Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2021)
- Windows 10 or later

### Steps

```bash
git clone https://github.com/Next-Tablet-Driver/NextTabletDriver.git
cd NextTabletDriver
cargo build --release
```

The compiled binary will be available in `target/release/`.

---

## 🗂️ Project Structure

```
NextTabletDriver/
├── .github/        # GitHub Actions & workflows
├── resources/      # Application resources (icons, assets)
├── src/            # Source code
├── tablets/        # Tablet configuration files
├── build.rs        # Build script (Windows resources)
├── installer.iss   # Inno Setup installer script
├── Cargo.toml      # Project dependencies
└── Cargo.lock
```

---

## 🎮 Supported Use Cases

- **osu!** — minimal latency for competitive play
- **Digital drawing** — precise and smooth pen input
- **General tablet use** — area mapping, pressure curve, etc.

---

## 🤝 Contributing

Contributions are welcome! This project is tagged `help-wanted` — feel free to open an issue or submit a pull request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -m 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

---

## 📄 License

This project is open source. See the [LICENSE](LICENSE) file for details.

---

## 👤 Author

[@iSweat](https://github.com/iSweat-exe)

---

<p align="center">
  <a href="https://github.com/Next-Tablet-Driver/NextTabletDriver/releases">Download latest release</a> •
  <a href="https://github.com/Next-Tablet-Driver/NextTabletDriver/issues">Report a bug</a> •
  <a href="https://github.com/Next-Tablet-Driver/NextTabletDriver/discussions">Discussions</a>
</p>