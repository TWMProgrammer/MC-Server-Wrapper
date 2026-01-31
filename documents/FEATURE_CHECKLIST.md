# ✅ Feature Checklist & Testing Guide

This document lists all implemented features and the expected behavior for verification and testing.

---

## 🏗️ Phase 1 & 2: Foundation & Management

### 1. Instance Management
- [ ] **Creation**: Can create a new instance with a specific name and version.
- [ ] **Storage**: Instance metadata is correctly saved in `instances.json`.
- [ ] **Isolation**: Each instance has its own directory and doesn't interfere with others.
- [ ] **Last Run**: The `last_run` timestamp updates correctly when a server starts.

### 2. Version Downloader
- [ ] **Mojang API**: Successfully fetches the version manifest from Mojang.
- [ ] **Paper/Fabric**: Correct metadata scraping for non-vanilla versions.
- [ ] **Download Integrity**: Artifacts are downloaded and hash-verified.
- [ ] **Java Detection**: Correctly identifies the required Java version for the selected Minecraft version.

### 3. Process Control
- [ ] **Spawning**: Server process starts with correct memory flags (`-Xmx`, `-Xms`).
- [ ] **Stdin/Stdout**: Can send commands to the server and receive colored console output.
- [ ] **Graceful Shutdown**: Sending "stop" command results in a clean exit.
- [ ] **Auto-Restart**: Server automatically restarts if it crashes (non-zero exit code).

### 4. Backups
- [ ] **Manual Trigger**: Can create a zip/tar.gz backup of an instance directory.
- [ ] **Auto-Backup**: Backup is triggered correctly on server stop.

---

## 💎 Phase 3: UI & Enhancements

### 1. Tauri Desktop UI
- [ ] **Navigation**: Sidebar correctly lists all created instances.
- [ ] **State Sync**: UI updates in real-time when server status changes (Stopped -> Starting -> Running).
- [ ] **Responsiveness**: UI remains fluid during heavy I/O (like downloads).

### 2. Resource Monitoring
- [ ] **CPU Tracking**: Accurate CPU usage percentage for the specific server process.
- [ ] **Memory Tracking**: Accurate RSS memory usage (MB/GB).
- [ ] **History Graphs**: Graphs correctly plot usage over time with smooth animations.

### 3. Plugin/Mod Downloader
- [ ] **Modrinth**: Search results return correct metadata and primary download links.
- [ ] **Spigot (Spiget)**: Successfully downloads `.jar` files for plugins.
- [ ] **CurseForge**: Requires API key; verify search and download flow with a valid key.
- [ ] **Installation**: Downloaded files are placed in the correct `plugins/` or `mods/` directory of the instance.

---

## 🧪 Testing Procedures

### Unit Tests
Run `cargo test` to verify:
- Config serialization/deserialization.
- Path handling and URL generation.
- Metadata parsing.

### Integration Tests
- Start a mock server and verify console output capture.
- Simulate a crash and verify the auto-restart logic.
- Verify backup file integrity.

### UI/Manual Tests
- Launch the app with `npm run tauri dev`.
- Create a "Test Instance", download a version, and start it.
- Observe the resource graphs while the server is loading chunks.
- Download a small plugin (e.g., EssentialsX) and verify it appears in the `plugins` folder.
