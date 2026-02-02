# Minecraft Server Wrapper

A powerful, modern, and cross-platform desktop application designed to simplify the management of Minecraft server instances. Built with performance and ease of use in mind, this wrapper provides a comprehensive interface for creating, configuring, and monitoring your Minecraft servers.

## 🚀 Features

- **Instance Management**: 
  - Create, delete, and clone multiple Minecraft server instances.
  - Support for various software loaders: Fabric, Forge, NeoForge, Paper, Purpur, and Bedrock.
  - Import existing server archives (ZIP, 7z).
- **Marketplace Integration**:
  - Browse and install **Mods** from Modrinth and CurseForge.
  - Browse and install **Plugins** from Modrinth and Spiget.
  - Built-in review and dependency management for mods and plugins.
- **Real-time Monitoring & Control**:
  - Live interactive console with command history.
  - Real-time resource usage tracking (CPU & Memory).
  - Comprehensive **Player Management**: Whitelist, Ops, Banned Players/IPs, and User Cache management.
- **Advanced Configuration**:
  - Sophisticated editors for `server.properties`, JSON, TOML, YAML, and XML.
  - Tree-based YAML editor for complex configurations.
  - Monaco-based text editor for direct file editing.
- **Automation & Safety**:
  - **Scheduled Tasks**: Automate backups and server restarts using cron expressions.
  - **Backup System**: Create, restore, and manage server backups with ZIP compression.
- **Modern UI/UX**:
  - Clean, responsive dashboard built with React 19 and Tailwind CSS.
  - Dynamic app scaling and smooth animations with Framer Motion.
  - Dark-themed, high-performance interface.

## 🛠️ Tech Stack

### Backend (Rust)
- **Tauri v2**: Cross-platform desktop framework.
- **Tokio**: Asynchronous runtime for concurrent operations.
- **SQLx & SQLite**: Reliable database for instance metadata and task scheduling.
- **tokio-cron-scheduler**: For managing automated server tasks.
- **sysinfo**: Real-time system and process monitoring.

### Frontend (React)
- **React 19**: Latest modern UI development.
- **TypeScript**: Full type safety across the application.
- **Tailwind CSS**: Utility-first styling for a responsive design.
- **Monaco Editor**: Powering advanced configuration editing.
- **Framer Motion**: Smooth transitions and UI animations.
- **Recharts**: Performance metric visualization.

## 📥 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.77.2 or higher)
- [Node.js](https://nodejs.org/) (v18 or higher)
- [npm](https://www.npmjs.com/)

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/mc-server-wrapper.git
   cd mc-server-wrapper
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

4. **Build for production**:
   ```bash
   npm run tauri build
   ```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
