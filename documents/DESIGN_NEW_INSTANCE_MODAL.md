# 🎨 Design: New Instance Modal

This document outlines the architectural and UI design for the "Create New Server" modal, inspired by the Prism Launcher's "New Instance" dialog.

## 🧱 UI Architecture

### 1. Component Structure
The modal will be implemented as a React component (`CreateInstanceModal.tsx`) using a multi-pane layout:

- **Modal Container**: A large, centered overlay with a dark theme (matches the existing UI).
- **Header**:
    - **Instance Icon**: Default grass block, customizable later.
    - **Name Input**: Text field for instance name.
    - **Group Select**: Dropdown for categorizing instances.
- **Main Layout (Flexbox)**:
    - **Sidebar (Left)**: Vertical navigation for installation sources:
        - `Custom` (Mojang Direct)
        - `Import` (Zip/Local Folder)
        - `Modrinth` (Direct search)
        - `CurseForge` (Direct search)
        - `FTB` / `Technic` (Future support)
    - **Content Area (Right)**: Dynamic pane based on sidebar selection.
        - **Version Table**: List of Minecraft versions with columns for ID, Release Date, and Type.
        - **Side Filters**: Checkboxes to toggle Snapshots, Betas, etc.
        - **Search Bar**: Real-time filtering of the version list.
    - **Bottom Pane**: Mod loader selection (None, Fabric, Forge, etc.) with its own version selector and search.
- **Footer**:
    - `OK` (Disabled if selection is invalid)
    - `Cancel`
    - `Help`

### 2. Styling
- **Tailwind CSS**: For responsive layout and theme-consistent colors.
- **Lucide-React**: For sidebar icons and status indicators.
- **Transitions**: Smooth transitions between sidebar categories.

---

## ⚙️ Backend Integration (Rust)

### 1. Data Structures
```rust
#[derive(Serialize)]
pub struct MCVersion {
    pub id: String,
    pub release_date: DateTime<Utc>,
    pub version_type: String, // "release", "snapshot", "old_beta", etc.
}

#[derive(Serialize)]
pub struct ModLoader {
    pub name: String,
    pub versions: Vec<String>,
}
```

### 2. Tauri Commands
- `get_minecraft_versions()`: Fetches and caches the version manifest from Mojang.
- `get_mod_loaders(mc_version: String)`: Fetches compatible mod loaders for a specific Minecraft version.
- `create_instance_full(options: CreateOptions)`: Handles the multi-step process:
    1. Create directory.
    2. Download server JAR.
    3. Install mod loader (if selected).
    4. Save metadata.

---

## 🛠️ Implementation Steps

1.  **Phase 1: Mockup & Layout**
    - Build the static UI in React with Tailwind.
    - Implement sidebar navigation state.
2.  **Phase 2: Version Manifest Enhancement**
    - Update `VersionDownloader` to parse and return release dates.
    - Implement caching to prevent repeated API calls to Mojang.
3.  **Phase 3: Filtering & Search Logic**
    - Implement client-side filtering for the version table.
    - Add sorting by date and version number.
4.  **Phase 4: Mod Loader Integration**
    - Implement API clients for Fabric (meta.fabricmc.net) and Forge (files.minecraftforge.net).
    - Dynamically update mod loader options based on selected MC version.
5.  **Phase 5: Final Hookup**
    - Connect the `OK` button to the `create_instance` logic.
    - Add progress indicators for the download/install process.
