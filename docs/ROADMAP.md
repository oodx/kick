# Project Roadmap: 'Kick' Modular API Client

This roadmap outlines the high-level work required to refactor, test, and prove the correctness of the 'Kick' modular API client, moving from foundational improvements to more complex, integrated features. Each phase includes milestones and an initial high-level story point estimate.

## High-Level Story Point Estimate: 75

*   **Story Point Definition:** A relative measure of effort, complexity, and uncertainty. These are initial estimates and may be refined as tasks are broken down.

## Phase 1: Foundational Improvements & Testing (Estimated Story Points: 20)

**Objective:** Establish a robust testing framework and address critical architectural issues related to the core HTTP client.

### Milestones:

*   **M1.1: Setup Testing Infrastructure (5 SP)**
    *   Add necessary `dev-dependencies` (e.g., `tokio::test`, `assert_matches`, `mockito` or `wiremock`).
    *   Create a `tests/` directory with a basic `mod.rs` and initial test setup.
    *   Implement a simple unit test for `Config` loading and saving.

*   **M1.2: Refactor HTTP Client Usage (8 SP)**
    *   Replace the manual `TcpStream` connection and `http1::handshake` with a proper `hyper::Client` instance within `ApiClient`.
    *   Configure `hyper::Client` to use `hyper_tls::HttpsConnector` for correct TLS handling and connection pooling.
    *   Update `ApiClient::execute_single_request` to leverage the `hyper::Client` for all HTTP requests.

*   **M1.3: Basic `ApiClient` Tests (5 SP)**
    *   Write integration tests for `ApiClient::get` and `ApiClient::post_json` methods against a mock HTTP server (e.g., `httpbin.org` or a local mock).
    *   Verify the retry logic and timeout handling of `ApiClient::execute_request`.

*   **M1.4: Enhance Error Handling (2 SP)**
    *   Refine `ApiError` variants to provide more specific context (e.g., include the URL and HTTP method in `ApiError::HttpStatus` or `ApiError::Timeout`).
    *   Ensure all potential error paths within `ApiClient` return the most appropriate and informative `ApiError` type.

## Phase 2: Plugin System Refinement & Core Features (Estimated Story Points: 30)

**Objective:** Make the plugin system more powerful and complete core functionalities like base URL handling and robust storage.

### Milestones:

*   **M2.1: Implement Response Plugin Hooks (10 SP)**
    *   Modify `ApiClient::execute_request` to correctly invoke `PluginHook::PreResponse` and `PluginHook::PostResponse` at the appropriate stages of response processing.
    *   Update the `Plugin` trait and `PluginManager` to fully support these new hooks, allowing plugins to inspect and modify responses.
    *   Write unit/integration tests to verify the functionality of response plugin hooks.

*   **M2.2: Plugin Configuration & Initialization (8 SP)**
    *   Enhance `PluginManager` to be responsible for initializing registered plugins, passing their specific configurations from `Config::plugins::plugin_settings` to the `plugin.initialize()` method.
    *   Add tests to ensure plugins are correctly initialized with their respective settings.

*   **M2.3: Base URL Implementation (7 SP)**
    *   Implement logic within `ApiClient` to automatically resolve relative URLs provided in requests against the `Config::client::base_url`.
    *   Add comprehensive tests for various base URL scenarios (e.g., absolute URLs, relative paths, trailing slashes).

*   **M2.4: Storage Manager Robustness (5 SP)**
    *   Improve `StorageManager::ensure_parent_dir` and related file path handling to gracefully manage and create complex, nested directory structures.
    *   Add more robust tests for `StorageManager` operations, including edge cases for file paths and size limits.

## Phase 3: Advanced Features & Polish (Estimated Story Points: 25)

**Objective:** Introduce advanced capabilities, improve overall usability, and ensure comprehensive documentation.

### Milestones:

*   **M3.1: Dynamic Configuration Reloading (Stretch Goal / 10 SP)**
    *   Investigate and implement a mechanism for `ApiClient` to dynamically reload its `Config` without requiring a full re-instantiation of the client (e.g., by wrapping `Config` in `Arc<RwLock<T>>`).
    *   Consider the implications for existing connections and plugin states during a reload.

*   **M3.2: Advanced Streaming & Backpressure (8 SP)**
    *   Review and potentially enhance the `streaming_rs` module to include more sophisticated backpressure and flow control mechanisms, especially for high-volume data transfers.
    *   Add more comprehensive tests for streaming components, focusing on performance, stability, and resource management under load.

*   **M3.3: Comprehensive Documentation (5 SP)**
    *   Add extensive Rustdoc (`///`) comments to all public structs, enums, traits, and functions across the entire library.
    *   Ensure examples are included where appropriate within the documentation.
    *   Generate and review the HTML documentation to ensure clarity and completeness.

*   **M3.4: Expanded Examples (2 SP)**
    *   Create dedicated, focused examples for each major feature (e.g., a standalone plugin example, a custom stream processor example, advanced storage usage scenarios).
    *   Update the `main.rs` example to showcase the newly implemented features and best practices.
