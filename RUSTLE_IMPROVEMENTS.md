# Rustle Editor: Improvement and Refactoring Suggestions

This document summarizes the architectural improvements and refactoring opportunities discussed for the Rustle editor project.

## 1. Architecture & State Management

The current Elm-like architecture is a good start, but a more robust Redux-style pattern would be beneficial for long-term scalability and maintainability.

- **Centralize State:** Refactor the state management to use a single, global `AppState` struct. This creates a single source of truth for the entire application.
- **Use a Pure Reducer:** Implement a single, pure `reducer` function that is responsible for all state changes. This makes state transitions predictable and easy to test.
  ```rust
  fn reducer(state: &AppState, action: &Action) -> AppState { ... }
  ```
- **Introduce Middleware:** Use a middleware pattern to handle side effects like file I/O, logging, or asynchronous API calls. This keeps the reducers pure and the codebase organized.
- **Improve Component Model:** Enhance the `Component` trait by introducing `Props` for passing data down the component tree and lifecycle methods (`on_mount`, `on_unmount`) for managing component setup and teardown.

## 2. Testing Strategy

Implement a multi-layered testing strategy to ensure the correctness and stability of the codebase.

- **Snapshot Testing:** Use the `insta` crate for snapshot testing the UI. This is an excellent way to do end-to-end testing of the rendering pipeline. We should backport the `TestCanvas` from `rustle_core2` to facilitate this.
- **Unit Testing:** Write unit tests for all pure functions, especially:
  - The `reducer` function.
  - `nom` parsers for command-line input.
  - Utility functions and data structures.
- **Integration Testing:** Write tests to verify that different parts of the system work together correctly.

## 3. Storage Abstraction

To support different file storage backends on different platforms, we should introduce a `Storage` trait.

- **Define a `Storage` Trait:** Create a trait that defines a generic interface for file operations.

  ```rust
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum DirEntry {
      File(String),
      Directory(String),
  }

  #[async_trait::async_trait]
  pub trait Storage {
      async fn read(&self, path: &str) -> Result<String>;
      async fn write(&self, path: &str, content: &str) -> Result<()>;
      async fn list(&self, path: &str) -> Result<Vec<DirEntry>>;
  }
  ```
- **Implement Concrete Backends:**
  - **TUI:** Create a `FileSystemStorage` that uses `tokio::fs`.
  - **WebUI:** Implement backends for:
    - The **File System Access API** for a native-like experience.
    - **`localStorage`** for a simple, offline-first option.
    - **Cloud Storage (Google Drive, Dropbox, etc.)** by using their respective REST APIs and OAuth for authentication.
- **File Browser:** Use the `storage.list()` method to build a `FileBrowser` component.

## 4. API & Modularity

- **Granular Error Handling:** Define custom, specific error types for different modules (`BufferError`, `RenderError`, etc.) instead of relying solely on `anyhow` for library code.
- **Shared Frontend Crate:** Extract logic that is duplicated between `rustle_tui` and `rustle_webui` (like key event mapping) into a shared utility crate.
- **Configuration System:** Implement a robust configuration system that allows users to customize keybindings, themes, and other settings across all frontends.

## 5. Layout Engine

- **Stick with `taffy`:** The current proof-of-concept using `taffy` for layout is the correct path forward. It's a powerful, platform-agnostic Flexbox implementation that is a perfect fit for the project's multi-frontend architecture.
