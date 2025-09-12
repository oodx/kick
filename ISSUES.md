# ISSUES.md

## 1. Builder lacks timeout configuration (`with_timeout`) mentioned in README  
`ApiClientBuilder` only exposes methods for config, plugin manager, headers, and user agent, yet the README’s usage example calls `.with_timeout(...)`, which does not exist in the implementation



:::task-stub{title="Add timeout configuration to ApiClientBuilder"}
- File: `src/client/mod.rs`
- Add `with_timeout(Duration)` method to `ApiClientBuilder`, storing the duration.
- Include the timeout when building `ApiClient` by overriding `config.client.timeout`.
- Update README example if API changes.
:::

## 2. `download_file_with_options` assumes UTF‑8, corrupting binary downloads  
File downloads rely on `self.get` and convert the response `String` into bytes, causing failure for non‑UTF‑8 content



:::task-stub{title="Support binary downloads"}
- File: `src/client/mod.rs`
- Implement a byte-oriented request method (e.g., `get_bytes`) using `hyper` body without `String::from_utf8`.
- Use this new method inside `download_file_with_options` to write raw bytes.
- Adjust tests to cover binary data.
:::

## 3. Error plugin hook not triggered on failing PUT/DELETE/PATCH requests  
Non-success responses in `put_json`, `delete`, and `patch_json` return immediately without invoking `execute_error`, leading to inconsistent plugin behavior



:::task-stub{title="Invoke error plugins on request failures"}
- File: `src/client/mod.rs`
- In `put_json`, `delete`, and `patch_json`, call `plugin_manager.execute_error(&error)` before returning the error.
- Ensure tests cover plugin invocation on HTTP errors.
:::

## 4. URL and header inputs bypass security validators  
Requests use raw strings without `UrlValidator` or `HeaderValidator`, missing SSRF and header-injection defenses



:::task-stub{title="Integrate URL and header validation"}
- File: `src/client/mod.rs`
- Validate URLs via `UrlValidator::validate` before building requests.
- Use `HeaderValidator::validate_header` in `with_header`.
- Propagate validation errors as `ApiError`.
:::

## 5. Filename sanitization duplicated instead of reusing `PathValidator`  
`ApiClient` defines its own `sanitize_filename` that mirrors logic already provided by `sec::PathValidator`



:::task-stub{title="Reuse PathValidator for filename sanitization"}
- Files: `src/client/mod.rs`, `src/sec/mod.rs`
- Replace `ApiClient::sanitize_filename` with call to `PathValidator::sanitize_filename`.
- Remove duplicate implementation and adjust imports/tests.
:::

## 6. Retry helper supports only GET and POST  
`execute_request_with_retry` rejects other HTTP methods, limiting reuse for PUT, PATCH, DELETE, etc.



:::task-stub{title="Extend retry helper to more HTTP methods"}
- File: `src/client/mod.rs`
- Accept additional methods (PUT, PATCH, DELETE) and route to corresponding client functions.
- Handle methods without body appropriately.
- Update documentation and tests.
:::

## 7. Retry logic uses constant delay instead of exponential backoff  
Each retry waits the same `retry_delay`, contradicting the framework’s goal of exponential backoff



:::task-stub{title="Implement exponential backoff"}
- File: `src/client/mod.rs`
- Replace constant sleep with exponential backoff (e.g., doubling delay per attempt with an upper bound).
- Expose backoff configuration via `Config` if needed.
- Add tests verifying increasing delays.
:::

## 8. `base_url` configuration field is unused  
`ClientConfig` defines `base_url`, but requests require full URLs and never combine with this setting



:::task-stub{title="Honor base_url in requests"}
- Files: `src/config/mod.rs`, `src/client/mod.rs`
- Modify request builders to prepend `config.client.base_url` when provided and a relative path is passed.
- Add validation to ensure URLs are correctly joined.
- Document base URL usage.
:::

---

These findings highlight areas where the Kick client can be made more robust and feature-complete.

