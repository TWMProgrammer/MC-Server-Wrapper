# 🧪 Testing Guide

This guide provides instructions on how to run the full test suite for the **MineCraft Server Wrapper** project.

## 📋 Prerequisites

Ensure you have the following installed:
- **Rust** (latest stable version)
- **Node.js** (v18 or later)
- **npm** (comes with Node.js)

---

## 🦀 Backend Testing (Rust)

The backend tests are split into core logic tests and Tauri command tests.

### **1. Core Logic Tests**
These tests cover the server management logic, configuration handling, and integration workflows.
```bash
cargo test
```
*Note: This runs tests defined in `tests/core/`.*

### **2. Tauri Command Tests**
These tests verify the API endpoints exposed to the frontend.
```bash
cargo test /src-tauri
```
*Note: This runs tests defined in `tests/tauri/`.*

### **3. Integrity Check**
Run this to ensure there are no compilation errors or linter warnings:
```bash
cargo check
```

---

## ⚛️ Frontend Testing (React)

The frontend uses **Vitest** for unit and component testing.

### **1. Unit & Component Tests**
Run all frontend tests using Vitest:
```bash
npm run test
```

### **2. Watch Mode**
To run tests in watch mode during development:
```bash
npx vitest
```

---

## 🎭 End-to-End (E2E) Testing

We use **Playwright** for end-to-end testing to ensure the frontend and backend work together correctly.

### **1. Run E2E Tests**
```bash
npm run test:e2e
```
*Note: This will automatically start the development server (`npm run dev`) and run tests against it.*

### **2. Debugging E2E Tests**
To run Playwright in UI mode for debugging:
```bash
npx playwright test --ui
```

---

## 🚀 Running the Full Suite

To perform a complete verification of the application, run the following commands in order:

1. **Backend Check**: `cargo check`
2. **Backend Tests**: `cargo test`
3. **Tauri Tests**: `cd src-tauri && cargo test && cd ..`
4. **Frontend Tests**: `npm run test`
5. **E2E Tests**: `npm run test:e2e`

---

## 🛠️ Troubleshooting

- **Test Failures in CI**: Ensure all dependencies are updated and `Cargo.lock` / `package-lock.json` are in sync.
- **Port Conflicts**: E2E tests use port `3000` by default. Ensure it's free or update `playwright.config.ts`.
- **Environment Variables**: Most tests use mock data or temporary directories, but ensure you have internet access for tests that interact with external APIs (like Modrinth or Spiget) if they aren't fully mocked.
