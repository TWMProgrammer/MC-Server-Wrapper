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
- [x] **Basic process spawning (Start/Stop)**
    - [x] `Child` process management with `tokio::process`.
    - [x] Environment variable passing and working directory isolation.
- [x] **Console output capture**
    - [x] Async stdout/stderr streaming.
    - [x] ANSI color code support and filtering.
- [x] **Basic TOML configuration**
    - [x] Global settings for application-wide behavior.
    - [x] Persistence using `serde` and `tokio::fs`.
- [x] **Directory Structure Initialization**
    - [x] Create `backups/` directory for server snapshots.
    - [x] Create `resources/` directory for application configurations.
    - [x] Create `server/` directory as the root for all instances.
    - [x] Ensure folders are relative to the executable.
- [x] **Logging & Error Handling**
    - [x] Setup `tracing` for structured logging.
    - [x] Implement centralized error handling with `anyhow`.

### Phase 2: Management
- [x] **Instance Management System**
    - [x] Create `InstanceManager` to track multiple server directories.
    - [x] Implement unique ID generation for instances.
    - [x] Metadata storage for each instance (version, last run, etc.).
- [x] **Version Downloader**
    - [x] Integrate with Mojang's Version Manifest API.
    - [x] Support for Fabric/Paper/Forge/NeoForge (Metadata scraping/APIs).
    - [x] Hash verification for downloaded artifacts.
- [x] **Process Interaction**
    - [x] Implement `send_command` via `stdin`.
    - [x] Graceful shutdown (send "stop", wait for exit).
    - [x] **Robust State Machine**: Implemented `lifecycle_loop` for managing `Stopped` -> `Starting` -> `Running` -> `Stopping` -> `Stopped` transitions.
    - [x] **Auto-Restart**: Integrated crash detection and automatic recovery logic.
- [x] **Data Safety**
    - [x] Automated backup triggers (on stop/scheduled).
    - [x] Compression support (zip/tar.gz).
- [x] **Scheduled Tasks**
    - [x] Cron-style scheduling for restarts and backups.
    - [x] Persistence of schedules in instance metadata.
    - [x] UI for managing schedules.
- [x] **Java Management**
    - [x] Automatic detection of system Java versions.
    - [x] Integration with Adoptium API for downloading specific JDKs.

### Phase 3: Enhancement
- [x] **Tauri-based Desktop UI**
    - [x] React + Tailwind foundation.
    - [x] Real-time state synchronization via Tauri Events.
- [x] **Instance Management System Enhancement**
    - [x] **Plugins Management (Phase 1)**
        - [x] Basic listing of `.jar` files in `plugins/` directory.
        - [x] Enable/Disable plugins via file renaming (`.jar.disabled`).
        - [x] Safe uninstallation with confirmation and optional config cleanup.
    - [x] **Mods Management (Phase 1)**
        - [x] Basic listing of `.jar` files in `mods/` directory.
        - [x] Enable/Disable mods via file renaming (`.jar.disabled`).
        - [x] Safe uninstallation with optional config cleanup.
        - [x] Marketplace integration (Modrinth & CurseForge).
        - [x] Dependency resolution (Modrinth & CurseForge).
        - [x] Scoped configuration editor with Monaco integration.
        - [x] Bulk actions for mod management.
- [x] **Instance Creation UI Redesign**
    - [x] **Frontend: `CreateInstanceModal` Component**
        - [x] Header section for instance Name and Group.
        - [x] Navigation sidebar for installation methods.
        - [x] Version selection grid with sorting and filtering.
        - [x] Mod loader selection panel (Fabric, Forge, Quilt, NeoForge).
    - [x] **Local Import Support**
        - [x] Recursive directory copy and ZIP extraction.
        - [x] Tauri commands for folder/ZIP inspection.
    - [x] **Backend: Metadata & API Integration**
        - [x] Fetch detailed version metadata and release dates.
        - [x] Mod Loader metadata fetching (Fabric/Forge/Quilt APIs).
- [x] **Plugin/Mod Downloader**
    - [x] Modrinth API integration for mods and plugins.
    - [x] Spigot (Spiget) API integration.
    - [x] CurseForge API support.
    - [x] Dependency resolution for complex mods.
- [x] **Resource Monitoring**
    - [x] Live CPU/RAM usage tracking.
    - [x] Disk I/O monitoring.
    - [x] Historical usage graphs with `recharts`.
- [x] **Visual Config Editors**
    - [x] `server.properties` visual editor with type safety.
    - [x] YAML/JSON editor for plugin configurations.
    - [x] Search and filter within configuration files.
- [x] **Player Management**
    - [x] Whitelist and Ops management via console commands (when running) and JSON/legacy files (when offline).
    - [x] Banned players and IPs management.
    - [x] User cache integration for offline/online UUIDs.

### Phase 4: Quality & Reliability
- [x] **Automated Testing Suite**
    - [x] Unit tests for `mc_server_wrapper_core` (Config, Managers, Downloader).
    - [x] Integration tests for Tauri commands in `src-tauri`.
    - [x] Frontend unit & component tests using `Vitest` and `React Testing Library`.
    - [x] E2E tests for critical user flows (Create Instance, Start/Stop Server) using `Playwright`.
- [/] **Robust Error Handling**
    - [ ] Replace `unwrap()`/`expect()` with proper error handling in `lib.rs`.
    - [x] Implement React Error Boundaries in the frontend.
- [x] **Production Observability**
    - [x] Persistent file logging for production builds using `tauri-plugin-log`.

### Phase 5: Future Expansion
- [ ] **Auto-Updater**
    - [ ] Core application update check on startup.
    - [ ] Automated backup before applying updates.
- [ ] **Remote Web Dashboard**
    - [ ] Next.js + Tailwind frontend for remote access.
    - [ ] JWT-based authentication and Role-Based Access Control (RBAC).
    - [ ] Dockerized deployment options.
- [ ] **Advanced Networking**
    - [ ] Built-in tunnel support (e.g., ngrok or cloudflare tunnels).
    - [ ] Proxy support for Velocity/BungeeCord clusters.
- [ ] **Developer Tools**
    - [ ] Public REST API for external automation.
    - [ ] WebSocket stream for real-time console access from 3rd party apps.
    - [ ] OpenAPI/Swagger documentation.
- [ ] **Optimization & Maintenance**
    - [ ] Log rotation and cleanup policies.
    - [ ] Advanced performance profiling tools for the server process.
- [ ] **Crash Reporting**
    - [ ] Crash reporting integration (e.g., Sentry or custom).

---

## 🛠️ Industry Standards & Best Practices
- **Safety**: No `unsafe` code unless strictly necessary for FFI.
- **Testing**: Comprehensive unit tests for core logic and integration tests for process management.
- **Documentation**: Full `rustdoc` coverage for the core library.
- **CI/CD**: GitHub Actions for automated builds, linting (`clippy`), and formatting (`rustfmt`).
