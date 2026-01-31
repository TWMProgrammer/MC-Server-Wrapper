# 🛠️ MineCraft Server Wrapper: Implementation Plan

A high-performance, modern, and aesthetically pleasing Minecraft server management solution built with **Rust**.

---

## 🚀 Vision
To create the "gold standard" of Minecraft server wrappers. Leveraging Rust's safety and speed to provide a seamless experience for both casual players and professional server admins.

## 🏗️ Tech Stack
| Component | Technology | Rationale |
| :--- | :--- | :--- |
| **Language** | Rust | Performance, memory safety, and concurrency. |
| **Async Runtime** | `tokio` | Industry standard for high-performance I/O. |
| **Frontend Framework** | `Tauri` (Desktop) / `Axum` (Web) | Native performance with modern Web UI (React/Next.js). |
| **Serialization** | `serde` | Robust configuration handling (TOML/JSON). |
| **Database** | `SQLite` (via `sqlx`) | Lightweight, zero-config relational storage. |
| **Logging** | `tracing` | Structured logging for debugging and monitoring. |

---

## 💎 Core Features

### 1. Instance Management
- **Multi-Instance Support**: Run multiple servers simultaneously in isolated environments.
- **Version Switcher**: One-click install for Vanilla, Paper, Spigot, Forge, Fabric, and Quilt.
- **Java Version Management**: Automatically detect and download required JDK versions (8, 11, 17, 21).

### 2. Process Control & Monitoring
- **Real-time Console**: Low-latency ANSI-colored console output with command history.
- **Auto-Restart**: Intelligent crash detection and automatic recovery.
- **Resource Tracking**: Live dashboards for CPU, RAM, and Disk I/O usage.
- **Scheduled Tasks**: Cron-style scheduling for restarts, backups, and commands.

### 3. Advanced Configuration
- **Visual Editor**: Edit `server.properties` and YAML configs with a user-friendly UI instead of raw text.
- **Template System**: Save server configurations as templates for rapid deployment.

### 4. Plugin & Mod Integration
- **Marketplace**: Browse and install plugins from SpigotMC, Modrinth, and CurseForge directly.
- **Auto-Updater**: Keep mods and plugins up to date automatically.

---

## 📐 Architecture Design

### **Layer 1: The Core (Rust Library)**
- `ServerHandle`: Manages the child process (stdin/stdout).
- `ConfigManager`: Handles persistence of settings.
- `EventManager`: Broadcasts server status changes via a pub/sub system.

### **Layer 2: The API (Axum/JSON)**
- Secure REST/WebSocket API for remote management.
- JWT-based authentication.

### **Layer 3: The Interface (Tauri/React)**
- A sleek, dark-themed UI inspired by modern IDEs.
- Interactive terminal component.
- Drag-and-drop file management.

---

## 🗺️ Roadmap

### Phase 1: Foundation (MVP)
- [x] Basic process spawning (Start/Stop).
- [x] Console output capture.
- [x] Basic TOML configuration.
- [x] **Directory Structure Initialization**
    - [x] Create `backups/` directory for server snapshots.
    - [x] Create `resources/` directory for application configurations.
    - [x] Create `server/` directory as the root for all instances.
    - [x] Ensure folders are relative to the executable.

### Phase 2: Management
- **Instance Management System**
    - [x] Create `InstanceManager` to track multiple server directories.
    - [x] Implement unique ID generation for instances.
    - [x] Metadata storage for each instance (version, last run, etc.).
- **Version Downloader**
    - [x] Integrate with Mojang's Version Manifest API.
    - [x] Support for Fabric/Paper/Forge/NeoForge (Metadata scraping/APIs).
    - [x] Hash verification for downloaded artifacts.
- **Process Interaction**
    - [x] Implement `send_command` via `stdin`.
    - [x] Graceful shutdown (send "stop", wait for exit).
- **Data Safety**
    - [x] Automated backup triggers (on stop/scheduled).
    - [x] Compression support (zip/tar.gz).

### Phase 3: Enhancement
- [x] Tauri-based Desktop UI (React + Tailwind foundation).
- [x] Plugin/Mod downloader (Modrinth, Spigot, and CurseForge).
- [x] Resource monitoring (Live CPU/RAM graphs in UI).
- [ ] **Instance Creation UI Redesign (Prism Launcher Inspired)**
    - [ ] **Frontend: `CreateInstanceModal` Component**
        - [ ] Header section for instance Name and Group.
        - [ ] Navigation sidebar for installation methods (Custom, Import, Modrinth, etc.).
        - [ ] Version selection grid/table with sorting (Version, Release Date, Type).
        - [ ] Advanced filtering for versions (Snapshots, Betas, Experiments).
        - [ ] Mod loader selection panel (Fabric, Forge, Quilt, NeoForge).
        - [ ] Integrated search for both versions and mod loaders.
    - [ ] **Backend: Metadata & API Integration**
        - [ ] Extend `VersionDownloader` to fetch detailed version metadata (release dates).
        - [ ] Implement Mod Loader metadata fetching (Fabric/Forge/Quilt APIs).
        - [ ] Add Tauri commands for asynchronous version listing and filtering.
- [ ] Visual `server.properties` editor.
- [ ] Auto-updater for core components.

### Phase 4: Scale
- [ ] Remote Web Dashboard.
- [ ] API for external integrations.
- [ ] Advanced clustering/proxy support.

---

## 🛠️ Industry Standards & Best Practices
- **Safety**: No `unsafe` code unless strictly necessary for FFI.
- **Testing**: Comprehensive unit tests for core logic and integration tests for process management.
- **Documentation**: Full `rustdoc` coverage for the core library.
- **CI/CD**: GitHub Actions for automated builds, linting (`clippy`), and formatting (`rustfmt`).
