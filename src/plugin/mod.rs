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
    async fn handle_pre_request(&self, url: &str, _context: &PluginContext) -> Result<()> {
        // Default implementation - plugins can override
        Ok(())
    }
    
    /// Handle post-request processing
    async fn handle_post_request(&self, url: &str, status: u16, _context: &PluginContext) -> Result<()> {
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
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
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