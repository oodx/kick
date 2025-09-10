# UAT-003: PLUGIN CONFIGURATION LOADING
**Status**: Open  
**Priority**: P2 (Medium - Enhances Business Extensibility)  
**Source**: Horus Executive UAT Assessment - FEATHER_USABILITY_01.md  
**Component**: Plugin System  

## Executive Summary
**EXTENSIBILITY GAP**: Plugin system architecture is solid but requires programmatic setup instead of configuration-based loading, limiting executive accessibility and business workflow integration.

## Problem Statement
Current plugin limitations:
1. Plugin trait and manager architecture excellent ✅
2. Logging and rate limiting plugins functional ✅
3. Pre/post request hooks working ✅
4. **No TOML configuration loading for plugins** ❌
5. **Requires programming knowledge for plugin setup** ❌

## Executive Impact
**Expected Executive Workflow**:
```toml
[plugins]
enabled = ["rate_limiter", "auth", "logging"]

[plugins.rate_limiter]
requests_per_minute = 100

[plugins.auth]  
token = "exec-api-key"
header = "X-Corporate-Auth"

[plugins.logging]
level = "info"
```

**Current Reality**: Executives must write Rust code to configure plugins

## Acceptance Criteria
- [ ] TOML configuration section for plugins parsed correctly
- [ ] Plugins auto-loaded based on configuration
- [ ] Plugin settings passed from config to plugin initialization
- [ ] Built-in plugins (logging, rate_limiter) support config-based setup
- [ ] Documentation shows config-based plugin setup examples
- [ ] Integration test demonstrates TOML → plugin loading

## Technical Requirements
1. Extend `Config` struct to include `plugins` section
2. Add `PluginConfig` struct for individual plugin settings
3. Implement `PluginManager::from_config()` method
4. Create plugin registry for built-in plugins
5. Update built-in plugins to accept config-based initialization
6. Add plugin configuration validation

## Suggested Implementation Approach
```rust
// Config structure
[plugins]
enabled_plugins = ["logging", "rate_limiter"]

[plugins.logging]
level = "info"
format = "json"

[plugins.rate_limiter]  
requests_per_minute = 60
```

```rust
// Code changes
impl PluginManager {
    pub fn from_config(plugin_config: &PluginConfig) -> Result<Self> {
        // Auto-register configured plugins
    }
}
```

## Business Value
**MEDIUM**: Enables configuration-driven plugin setup, making the system accessible to non-programmers and supporting business workflow automation.

## Definition of Done
- Plugin configuration loading works end-to-end
- Built-in plugins support config-based initialization
- Documentation includes config examples
- Integration test validates TOML → plugin loading
- Executive-friendly plugin setup (no code required)

## Dependencies
- None (independent enhancement)

## Future Extensions
- Plugin marketplace/discovery
- Hot-reload plugin configuration
- Custom plugin loading from external modules

---
*Created from Horus Executive UAT Assessment - Plugin System Analysis*