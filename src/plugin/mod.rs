use crate::error::{ApiError, Result};
use async_trait::async_trait;
use hyper::{Request, Response};
use hyper::body::Body;
use http_body_util::combinators::BoxBody;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Plugin hook points in the request/response lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PluginHook {
    PreRequest,
    PostRequest,
    PreResponse,
    PostResponse,
    OnError,
    OnRetry,
    OnStream,
}

/// Context passed to plugins
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub hook: PluginHook,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PluginContext {
    pub fn new(hook: PluginHook) -> Self {
        Self {
            hook,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Main plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;
    
    /// Plugin version
    fn version(&self) -> &str;
    
    /// Plugin description
    fn description(&self) -> &str;
    
    /// Initialize the plugin with configuration
    async fn initialize(&mut self, config: &serde_json::Value) -> Result<()>;
    
    /// Check if plugin should handle this hook
    fn handles_hook(&self, hook: &PluginHook) -> bool;
    
    /// Handle pre-request processing
    async fn handle_pre_request(
        &self,
        _request: &mut Request<BoxBody<Bytes, hyper::Error>>,
        _context: &PluginContext,
    ) -> Result<()> {
        Ok(())
    }
    
    /// Handle post-request processing
    async fn handle_post_request(
        &self,
        _request: &Request<BoxBody<Bytes, hyper::Error>>,
        _context: &PluginContext,
    ) -> Result<()> {
        Ok(())
    }
    
    /// Handle pre-response processing
    async fn handle_pre_response(
        &self,
        _response: &mut Response<BoxBody<Bytes, hyper::Error>>,
        _context: &PluginContext,
    ) -> Result<()> {
        Ok(())
    }
    
    /// Handle post-response processing
    async fn handle_post_response(
        &self,
        _response: &Response<BoxBody<Bytes, hyper::Error>>,
        _context: &PluginContext,
    ) -> Result<()> {
        Ok(())
    }
    
    /// Handle error cases
    async fn handle_error(
        &self,
        _error: &ApiError,
        _context: &PluginContext,
    ) -> Result<()> {
        Ok(())
    }
    
    /// Handle retry attempts
    async fn handle_retry(
        &self,
        _attempt: usize,
        _context: &PluginContext,
    ) -> Result<()> {
        Ok(())
    }
    
    /// Handle streaming data
    async fn handle_stream(
        &self,
        _data: &[u8],
        _context: &PluginContext,
    ) -> Result<Vec<u8>> {
        Ok(_data.to_vec())
    }
}

/// Plugin manager to handle registration and execution
pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    hook_plugins: HashMap<PluginHook, Vec<String>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            hook_plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        
        // Build hook mapping
        for hook in [
            PluginHook::PreRequest,
            PluginHook::PostRequest,
            PluginHook::PreResponse,
            PluginHook::PostResponse,
            PluginHook::OnError,
            PluginHook::OnRetry,
            PluginHook::OnStream,
        ] {
            if plugin.handles_hook(&hook) {
                self.hook_plugins
                    .entry(hook)
                    .or_insert_with(Vec::new)
                    .push(name.clone());
            }
        }
        
        self.plugins.insert(name, plugin);
        Ok(())
    }
    
    /// Execute plugins for a specific hook
    pub async fn execute_hook(
        &self,
        hook: PluginHook,
        context: &PluginContext,
        callback: impl Fn(&Arc<dyn Plugin>, &PluginContext) -> futures::future::BoxFuture<'_, Result<()>>,
    ) -> Result<()> {
        if let Some(plugin_names) = self.hook_plugins.get(&hook) {
            for plugin_name in plugin_names {
                if let Some(plugin) = self.plugins.get(plugin_name) {
                    callback(plugin, context).await?;
                }
            }
        }
        Ok(())
    }
    
    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.get(name)
    }
    
    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Example logging plugin implementation
pub struct LoggingPlugin {
    name: String,
    enabled: bool,
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self {
            name: "logging".to_string(),
            enabled: true,
        }
    }
}

#[async_trait]
impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Logs HTTP requests and responses"
    }
    
    async fn initialize(&mut self, config: &serde_json::Value) -> Result<()> {
        if let Some(enabled) = config.get("enabled").and_then(|v| v.as_bool()) {
            self.enabled = enabled;
        }
        Ok(())
    }
    
    fn handles_hook(&self, hook: &PluginHook) -> bool {
        matches!(hook, 
            PluginHook::PreRequest | 
            PluginHook::PostResponse | 
            PluginHook::OnError
        )
    }
    
    async fn handle_pre_request(
        &self,
        request: &mut Request<Body>,
        _context: &PluginContext,
    ) -> Result<()> {
        if self.enabled {
            tracing::info!("Outgoing request: {} {}", request.method(), request.uri());
        }
        Ok(())
    }
    
    async fn handle_post_response(
        &self,
        response: &Response<Body>,
        _context: &PluginContext,
    ) -> Result<()> {
        if self.enabled {
            tracing::info!("Received response: {}", response.status());
        }
        Ok(())
    }
    
    async fn handle_error(
        &self,
        error: &ApiError,
        _context: &PluginContext,
    ) -> Result<()> {
        if self.enabled {
            tracing::error!("Request error: {}", error);
        }
        Ok(())
    }
}

/// Example rate limiting plugin
pub struct RateLimitPlugin {
    name: String,
    requests_per_minute: usize,
    last_reset: std::time::Instant,
    request_count: usize,
}

impl RateLimitPlugin {
    pub fn new(requests_per_minute: usize) -> Self {
        Self {
            name: "rate_limiter".to_string(),
            requests_per_minute,
            last_reset: std::time::Instant::now(),
            request_count: 0,
        }
    }
}

#[async_trait]
impl Plugin for RateLimitPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Implements rate limiting for API requests"
    }
    
    async fn initialize(&mut self, config: &serde_json::Value) -> Result<()> {
        if let Some(rpm) = config.get("requests_per_minute").and_then(|v| v.as_u64()) {
            self.requests_per_minute = rpm as usize;
        }
        Ok(())
    }
    
    fn handles_hook(&self, hook: &PluginHook) -> bool {
        matches!(hook, PluginHook::PreRequest)
    }
    
    async fn handle_pre_request(
        &self,
        _request: &mut Request<BoxBody<Bytes, hyper::Error>>,
        _context: &PluginContext,
    ) -> Result<()> {
        // Note: In a real implementation, you'd want thread-safe counters
        // This is simplified for demonstration
        if self.last_reset.elapsed().as_secs() >= 60 {
            // Reset would happen here
        }
        
        if self.request_count >= self.requests_per_minute {
            return Err(ApiError::RateLimit);
        }
        
        Ok(())
    }
}