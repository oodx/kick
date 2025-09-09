use std::sync::Arc;
use std::time::Duration;

use hyper::{Request, Method};
use hyper_util::client::legacy::Client;
use hyper_tls::HttpsConnector;
use hyper_util::rt::TokioExecutor;
use http_body_util::{Empty, Full, BodyExt};
use bytes::Bytes;
use tokio::time::timeout;
use serde_json::json;

use crate::error::{ApiError, Result};

/// Clean, minimal HTTP client for testing basic patterns
pub struct DriverClient {
    client: Client<HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>>,
    timeout_duration: Duration,
}

impl DriverClient {
    pub fn new() -> Self {
        let connector = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);
        
        Self {
            client,
            timeout_duration: Duration::from_secs(30),
        }
    }
    
    pub async fn get(&self, url: &str) -> Result<String> {
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("user-agent", "kick-driver/0.1.0")
            .body(Empty::<Bytes>::new().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
            
        let response = timeout(self.timeout_duration, self.client.request(request))
            .await
            .map_err(|_| ApiError::Timeout)?
            .map_err(|e| ApiError::other(format!("Client error: {}", e)))?;
            
        let status = response.status();
        if !status.is_success() {
            return Err(ApiError::HttpStatus { status });
        }
        
        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .map_err(ApiError::Http)?
            .to_bytes();
            
        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))
    }
    
    pub async fn post_json(&self, url: &str, data: &serde_json::Value) -> Result<String> {
        let json_body = serde_json::to_string(data)?;
        
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("content-type", "application/json")
            .header("user-agent", "kick-driver/0.1.0")
            .body(Full::new(Bytes::from(json_body)).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
            
        let response = timeout(self.timeout_duration, self.client.request(request))
            .await
            .map_err(|_| ApiError::Timeout)?
            .map_err(|e| ApiError::other(format!("Client error: {}", e)))?;
            
        let status = response.status();
        if !status.is_success() {
            return Err(ApiError::HttpStatus { status });
        }
        
        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .map_err(ApiError::Http)?
            .to_bytes();
            
        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))
    }
}

/// Simple plugin trait for testing
pub trait DriverPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn before_request(&self, url: &str) -> Result<()>;
    fn after_response(&self, url: &str, status: u16) -> Result<()>;
}

/// Basic logging plugin
pub struct LoggingPlugin;

impl DriverPlugin for LoggingPlugin {
    fn name(&self) -> &str {
        "logging"
    }
    
    fn before_request(&self, url: &str) -> Result<()> {
        println!("[DRIVER] Making request to: {}", url);
        Ok(())
    }
    
    fn after_response(&self, url: &str, status: u16) -> Result<()> {
        println!("[DRIVER] Response from {}: {}", url, status);
        Ok(())
    }
}

/// Plugin manager for driver
pub struct DriverPluginManager {
    plugins: Vec<Arc<dyn DriverPlugin>>,
}

impl DriverPluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    pub fn add_plugin(&mut self, plugin: Arc<dyn DriverPlugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn before_request(&self, url: &str) -> Result<()> {
        for plugin in &self.plugins {
            plugin.before_request(url)?;
        }
        Ok(())
    }
    
    pub fn after_response(&self, url: &str, status: u16) -> Result<()> {
        for plugin in &self.plugins {
            plugin.after_response(url, status)?;
        }
        Ok(())
    }
}

/// Enhanced client with plugin support
pub struct DriverApiClient {
    client: DriverClient,
    plugins: DriverPluginManager,
}

impl DriverApiClient {
    pub fn new() -> Self {
        Self {
            client: DriverClient::new(),
            plugins: DriverPluginManager::new(),
        }
    }
    
    pub fn with_plugin(mut self, plugin: Arc<dyn DriverPlugin>) -> Self {
        self.plugins.add_plugin(plugin);
        self
    }
    
    pub async fn get(&self, url: &str) -> Result<String> {
        self.plugins.before_request(url)?;
        
        let result = self.client.get(url).await;
        
        match &result {
            Ok(_) => self.plugins.after_response(url, 200)?,
            Err(ApiError::HttpStatus { status }) => {
                self.plugins.after_response(url, status.as_u16())?;
            }
            _ => {}
        }
        
        result
    }
    
    pub async fn post_json(&self, url: &str, data: &serde_json::Value) -> Result<String> {
        self.plugins.before_request(url)?;
        
        let result = self.client.post_json(url, data).await;
        
        match &result {
            Ok(_) => self.plugins.after_response(url, 200)?,
            Err(ApiError::HttpStatus { status }) => {
                self.plugins.after_response(url, status.as_u16())?;
            }
            _ => {}
        }
        
        result
    }
}

/// Test driver functionality
pub async fn run_driver_tests() -> Result<()> {
    println!("=== KICK DRIVER TESTS ===");
    
    // Test basic HTTP client
    println!("\n1. Testing basic HTTP GET...");
    let client = DriverClient::new();
    
    match client.get("https://httpbin.org/get").await {
        Ok(response) => {
            println!("✓ GET request successful");
            println!("Response preview: {}", &response[..200.min(response.len())]);
        }
        Err(e) => {
            println!("✗ GET request failed: {}", e);
            return Err(e);
        }
    }
    
    // Test POST JSON
    println!("\n2. Testing JSON POST...");
    let test_data = json!({
        "test": "data",
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    match client.post_json("https://httpbin.org/post", &test_data).await {
        Ok(response) => {
            println!("✓ POST request successful");
            println!("Response preview: {}", &response[..200.min(response.len())]);
        }
        Err(e) => {
            println!("✗ POST request failed: {}", e);
            return Err(e);
        }
    }
    
    // Test plugin system
    println!("\n3. Testing plugin system...");
    let client_with_plugins = DriverApiClient::new()
        .with_plugin(Arc::new(LoggingPlugin));
    
    match client_with_plugins.get("https://httpbin.org/status/200").await {
        Ok(_) => println!("✓ Plugin-enabled GET successful"),
        Err(e) => {
            println!("✗ Plugin-enabled GET failed: {}", e);
            return Err(e);
        }
    }
    
    // Test error handling
    println!("\n4. Testing error handling...");
    match client_with_plugins.get("https://httpbin.org/status/404").await {
        Ok(_) => println!("✗ Expected 404 error but got success"),
        Err(ApiError::HttpStatus { status }) if status.as_u16() == 404 => {
            println!("✓ 404 error handled correctly");
        }
        Err(e) => {
            println!("✗ Unexpected error: {}", e);
            return Err(e);
        }
    }
    
    println!("\n=== ALL DRIVER TESTS PASSED ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_driver_creation() {
        let client = DriverClient::new();
        // Just verify we can create without panicking
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_plugin_manager() {
        let mut manager = DriverPluginManager::new();
        manager.add_plugin(Arc::new(LoggingPlugin));
        
        // Test plugin execution doesn't panic
        assert!(manager.before_request("https://test.com").is_ok());
        assert!(manager.after_response("https://test.com", 200).is_ok());
    }
}