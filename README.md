# Minecraft Server Wrapper

A powerful, modern, and cross-platform desktop application designed to simplify the management of Minecraft server instances. Built with performance and ease of use in mind, this wrapper provides a comprehensive interface for creating, configuring, and monitoring your Minecraft servers.

## 🚀 Features

- **Instance Management**: Create, delete, and manage multiple Minecraft server instances with ease.
- **Mod Loader Support**: Seamless integration with popular mod loaders including:
  - Fabric
  - Forge
  - NeoForge
  - Paper
  - Purpur
- **Plugin Integration**: Browse and install plugins directly from:
  - Modrinth
  - CurseForge
  - Spiget
- **Real-time Monitoring**:
  - Live console output and command input.
  - Resource usage tracking (CPU, Memory).
  - Player management (list, op, kick, ban).
- **Advanced Configuration**:
  - Built-in editor for `server.properties`, JSON, TOML, YAML, and XML files.
  - Automated backups and restore functionality.
- **Modern UI**: A clean, responsive dashboard built with React and Tailwind CSS.

## 🛠️ Tech Stack

### Backend
- **Rust**: The core logic and server management are implemented in high-performance, memory-safe Rust.
- **Tauri**: Provides the cross-platform desktop framework and bridge between the backend and frontend.
- **Tokio**: Asynchronous runtime for handling concurrent server processes and I/O.
- **SQLx & SQLite**: Lightweight and reliable database for storing instance metadata and configurations.
- **Reqwest**: Handles all network requests for downloading server jars, mods, and plugins.

### Frontend
- **React 19**: Modern UI development with a component-based architecture.
- **TypeScript**: Ensuring type safety and better developer experience.
- **Tailwind CSS**: Utility-first CSS framework for a highly customizable and responsive design.
- **Framer Motion**: Smooth animations and transitions.
- **Lucide React**: Beautiful and consistent iconography.
- **Recharts**: Data visualization for server performance metrics.

## 📥 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Node.js](https://nodejs.org/) (v18 or higher)
- [npm](https://www.npmjs.com/) or [yarn](https://yarnpkg.com/)

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

## 📄 License

This project is licensed under the ISC License - see the [LICENSE](LICENSE) file for details.
