use crate::error::{ApiError, Result};
use async_trait::async_trait;
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

/// Simplified plugin trait based on driver patterns
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;
    
    /// Plugin version
    fn version(&self) -> &str;
    
    /// Plugin description
    fn description(&self) -> &str;
    
    /// Initialize plugin with configuration
    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    /// Check if plugin handles a specific hook
    fn handles_hook(&self, hook: &PluginHook) -> bool {
        match hook {
            PluginHook::PreRequest | PluginHook::PostRequest => true,
            _ => false,
        }
    }
    
    /// Handle pre-request processing (simplified - no request mutation for now)
    async fn handle_pre_request(&self, _url: &str, _context: &PluginContext) -> Result<()> {
        // Default implementation - plugins can override
        Ok(())
    }
    
    /// Handle post-request processing
    async fn handle_post_request(&self, _url: &str, _status: u16, _context: &PluginContext) -> Result<()> {
        // Default implementation - plugins can override
        Ok(())
    }
    
    /// Handle errors
    async fn handle_error(&self, _error: &ApiError, _context: &PluginContext) -> Result<()> {
        Ok(())
    }
    
    /// Handle retry attempts
    async fn handle_retry(&self, _attempt: u32, _context: &PluginContext) -> Result<()> {
        Ok(())
    }
    
    /// Handle pre-response processing (before body is consumed)
    async fn handle_pre_response(&self, _status: u16, _context: &PluginContext) -> Result<()> {
        Ok(())
    }
    
    /// Handle post-response processing (after body is consumed)
    async fn handle_post_response(&self, _body: &str, _context: &PluginContext) -> Result<()> {
        Ok(())
    }
    
    /// Handle streaming data chunks
    async fn handle_stream(&self, _chunk: &[u8], _context: &PluginContext) -> Result<()> {
        Ok(())
    }
}

/// Plugin manager for registering and executing plugins
pub struct PluginManager {
    pub plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<()> {
        self.plugins.push(plugin);
        Ok(())
    }
    
    /// Execute plugins for pre-request hook
    pub async fn execute_pre_request(&self, url: &str) -> Result<()> {
        let context = PluginContext::new(PluginHook::PreRequest);
        
        for plugin in &self.plugins {
            if plugin.handles_hook(&PluginHook::PreRequest) {
                plugin.handle_pre_request(url, &context).await?;
            }
        }
        Ok(())
    }
    
    /// Execute plugins for post-request hook
    pub async fn execute_post_request(&self, url: &str, status: u16) -> Result<()> {
        let context = PluginContext::new(PluginHook::PostRequest);
        
        for plugin in &self.plugins {
            if plugin.handles_hook(&PluginHook::PostRequest) {
                plugin.handle_post_request(url, status, &context).await?;
            }
        }
        Ok(())
    }
    
    /// Execute plugins for error hook
    pub async fn execute_error(&self, error: &ApiError) -> Result<()> {
        let context = PluginContext::new(PluginHook::OnError);
        
        for plugin in &self.plugins {
            plugin.handle_error(error, &context).await?;
        }
        Ok(())
    }
    
    /// Execute plugins for retry hook
    pub async fn execute_retry(&self, attempt: u32) -> Result<()> {
        let context = PluginContext::new(PluginHook::OnRetry);
        
        for plugin in &self.plugins {
            plugin.handle_retry(attempt, &context).await?;
        }
        Ok(())
    }
    
    /// Execute plugins for pre-response hook
    pub async fn execute_pre_response(&self, status: u16) -> Result<()> {
        let context = PluginContext::new(PluginHook::PreResponse);
        
        for plugin in &self.plugins {
            if plugin.handles_hook(&PluginHook::PreResponse) {
                plugin.handle_pre_response(status, &context).await?;
            }
        }
        Ok(())
    }
    
    /// Execute plugins for post-response hook
    pub async fn execute_post_response(&self, body: &str) -> Result<()> {
        let context = PluginContext::new(PluginHook::PostResponse);
        
        for plugin in &self.plugins {
            if plugin.handles_hook(&PluginHook::PostResponse) {
                plugin.handle_post_response(body, &context).await?;
            }
        }
        Ok(())
    }
    
    /// Execute plugins for stream hook
    pub async fn execute_stream(&self, chunk: &[u8]) -> Result<()> {
        let context = PluginContext::new(PluginHook::OnStream);
        
        for plugin in &self.plugins {
            if plugin.handles_hook(&PluginHook::OnStream) {
                plugin.handle_stream(chunk, &context).await?;
            }
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting plugin implementation
pub struct RateLimitPlugin {
    requests_per_second: u32,
    last_request_time: std::sync::Mutex<Option<std::time::Instant>>,
}

impl RateLimitPlugin {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
            last_request_time: std::sync::Mutex::new(None),
        }
    }
    
    async fn enforce_rate_limit(&self) -> Result<()> {
        let now = std::time::Instant::now();
        let sleep_duration = {
            let last_time = self.last_request_time.lock().unwrap();
            
            if let Some(last) = *last_time {
                let min_interval = std::time::Duration::from_secs_f64(1.0 / self.requests_per_second as f64);
                let elapsed = now.duration_since(last);
                
                if elapsed < min_interval {
                    Some(min_interval - elapsed)
                } else {
                    None
                }
            } else {
                None
            }
        };
        
        if let Some(duration) = sleep_duration {
            tokio::time::sleep(duration).await;
        }
        
        // Update the last request time
        {
            let mut last_time = self.last_request_time.lock().unwrap();
            *last_time = Some(std::time::Instant::now());
        }
        
        Ok(())
    }
}

#[async_trait]
impl Plugin for RateLimitPlugin {
    fn name(&self) -> &str {
        "rate_limit"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Rate limits API requests"
    }
    
    fn handles_hook(&self, hook: &PluginHook) -> bool {
        matches!(hook, PluginHook::PreRequest)
    }
    
    async fn handle_pre_request(&self, _url: &str, _context: &PluginContext) -> Result<()> {
        self.enforce_rate_limit().await
    }
}

/// Basic logging plugin implementation
pub struct LoggingPlugin;

impl LoggingPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        "logging"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Logs HTTP requests and responses"
    }
    
    async fn handle_pre_request(&self, url: &str, _context: &PluginContext) -> Result<()> {
        println!("[PLUGIN-LOG] Making request to: {}", url);
        Ok(())
    }
    
    async fn handle_post_request(&self, url: &str, status: u16, _context: &PluginContext) -> Result<()> {
        println!("[PLUGIN-LOG] Response from {}: {}", url, status);
        Ok(())
    }
    
    async fn handle_error(&self, error: &ApiError, _context: &PluginContext) -> Result<()> {
        println!("[PLUGIN-LOG] Error occurred: {}", error);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugins.len(), 0);
    }
    
    #[tokio::test]
    async fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Arc::new(LoggingPlugin::new());
        
        assert!(manager.register_plugin(plugin).is_ok());
        assert_eq!(manager.plugins.len(), 1);
    }
    
    #[tokio::test]
    async fn test_plugin_execution() {
        let mut manager = PluginManager::new();
        manager.register_plugin(Arc::new(LoggingPlugin::new())).unwrap();
        
        // Test plugin execution doesn't panic
        assert!(manager.execute_pre_request("https://test.com").await.is_ok());
        assert!(manager.execute_post_request("https://test.com", 200).await.is_ok());
    }
}