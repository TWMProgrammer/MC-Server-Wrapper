# 🤖 Agent Instructions

This document provides explicit instructions for AI agents working on the **MineCraft Server Wrapper** project.

---

## 🛠️ Verification & Quality Control

### **1. Check for Errors After Every Change**
After making any code modifications, you **MUST** verify the integrity of the codebase.
- Run `cargo check` to ensure there are no compilation errors or linter warnings.
- Run `cargo test` to execute all unit and integration tests.
- Run `npm run build` or the appropriate frontend build command to ensure TypeScript/React integrity.
- **NEVER** leave the project in a broken state. If you introduce errors, fix them immediately before proceeding.

### **2. Match Existing Coding Style**
- Use idiomatic Rust patterns (e.g., `anyhow` for errors, `tokio` for async).
- Maintain consistent naming conventions (snake_case for functions/variables, PascalCase for types).
- Document new public functions and structures using `///` doc comments.

---

## 🏗️ Project Architecture Rules

### **1. Core Logic First**
- Always implement logic in the [core](file:///c:/Users/Administrator/Desktop/Devving/Minecraft/MC%20Server%20Wrapper/src/core) module before exposing it via the CLI or UI.
- Ensure modules are properly declared in [mod.rs](file:///c:/Users/Administrator/Desktop/Devving/Minecraft/MC%20Server%20Wrapper/src/core/mod.rs).

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

### **5. UI & UX Standards**
- **Notifications**: ALWAYS use the `useToast` hook for success, error, or info messages. Toasts slide from the bottom right.
- **Confirmations**: NEVER use the default browser `confirm()` box. Use the `ConfirmDropdown` component for any action that requires user confirmation.
- **Dropdowns & Popups**: Use `createPortal` from `react-dom` for all custom dropdowns, select menus, and popups.
    - **NEVER** use the native HTML `<select>` element.
    - **ALWAYS** use the custom `Select` component from `ui/components/Select.tsx` for all dropdown selections.
    - **App Scaling**: Components using `createPortal` MUST manually implement app scaling. Use the `useAppSettings` hook and apply a scale transform to the portal container:
      ```tsx
      const { settings } = useAppSettings();
      // ...
      return createPortal(
        <div 
          className="fixed inset-0 z-[100] overflow-hidden"
          style={{
            width: `${100 / settings.scaling}%`,
            height: `${100 / settings.scaling}%`,
            transform: `scale(${settings.scaling})`,
            transformOrigin: 'top left',
          }}
        >
          {/* Modal Content */}
        </div>,
        document.body
      );
      ```
    - This ensures consistent styling, proper application scaling, and prevents clipping by parent containers with `overflow: hidden/auto`.
- **Framer Motion**: Use `framer-motion` for smooth transitions and animations where appropriate.
- **Opaque Modals**: All modals, including the Plugin Configuration Modal, MUST be fully opaque. NEVER use glass/blur effects (`backdrop-blur`) for the modal background itself, though the overlay (the background behind the modal) should remain blurred and semi-transparent.

---

## 📝 Workflow Requirements

- **Task Management**: Always use `TodoWrite` to plan and track your progress.
- **Implementation Plan**: Keep [IMPLEMENTATION_PLAN.md](file:///c:/Users/Administrator/Desktop/Devving/Minecraft/MC%20Server%20Wrapper/documents/IMPLEMENTATION_PLAN.md) updated as you complete phases.
- **Persistence**: Do not stop until the user's query is completely resolved and verified.

### **2. Markdown & Documentation**
- **Clickable Links**: 
    - **Internal Links**: Prefer relative paths (e.g., `[basename](../path/to/file)`) for links within the same repository. This is more robust and prevents line-wrapping issues.
    - **Absolute Links**: Use the `file:///` protocol only when absolute paths are strictly necessary.
- **Link Format**: 
    - **Spaces**: Encode spaces as `%20`.
    - **Basenames**: Use the file's basename (e.g., `java_manager.rs`) as the link text, not the full path.
- **Progress Tracking**: Use `- [ ]` and `- [x]` for all planning and audit documents.
