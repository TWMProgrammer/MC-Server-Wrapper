# 🤖 Agent Instructions

This document provides explicit instructions for AI agents working on the **MineCraft Server Wrapper** project.

---

## 🛠️ Verification & Quality Control

### **1. Check for Errors After Every Change**
After making any code modifications, you **MUST** verify the integrity of the codebase.
- Run `cargo check` to ensure there are no compilation errors or linter warnings.
- Run `cargo test` to execute all unit and integration tests.
- **NEVER** leave the project in a broken state. If you introduce errors, fix them immediately before proceeding.

### **2. Match Existing Coding Style**
- Use idiomatic Rust patterns (e.g., `anyhow` for errors, `tokio` for async).
- Maintain consistent naming conventions (snake_case for functions/variables, PascalCase for types).
- Document new public functions and structures using `///` doc comments.

---

## 🏗️ Project Architecture Rules

### **1. Core Logic First**
- Always implement logic in the [core](file:///c%3A/Users/Administrator/Desktop/Devving/Minecraft/MC%20Server%20Wrapper/src/core) module before exposing it via the CLI or UI.
- Ensure modules are properly declared in [mod.rs](file:///c%3A/Users/Administrator/Desktop/Devving/Minecraft/MC%20Server%20Wrapper/src/core/mod.rs).

### **2. Safety & Error Handling**
- Use `anyhow::Result` for fallible operations.
- Avoid `unwrap()` or `expect()` unless it's logically impossible for the operation to fail.
- Prefer `context()` to provide meaningful error messages during failures.

### **3. Dependency Management**
- Before adding a new dependency to [Cargo.toml](file:///c%3A/Users/Administrator/Desktop/Devving/Minecraft/MC%20Server%20Wrapper/Cargo.toml), check if an existing one can do the job.
- Group dependencies logically and keep versions up to date.

### **4. Modular Design & File Limits**
- Follow a modular design to keep the code manageable and easy to navigate.
- Keep individual files under **200 lines** of code. If a file exceeds this limit, refactor and split it into smaller modules.

---

## 📝 Workflow Requirements

- **Task Management**: Always use `TodoWrite` to plan and track your progress.
- **Implementation Plan**: Keep [IMPLEMENTATION_PLAN.md](file:///c%3A/Users/Administrator/Desktop/Devving/Minecraft/MC%20Server%20Wrapper/documents/IMPLEMENTATION_PLAN.md) updated as you complete phases.
- **Persistence**: Do not stop until the user's query is completely resolved and verified.
