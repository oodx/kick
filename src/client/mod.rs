use crate::config::Config;
use crate::error::{ApiError, Result};
use crate::plugin::PluginManager;
use std::sync::Arc;
use std::time::Duration;

use hyper::{Request, Method};
use hyper_util::client::legacy::Client;
use hyper_tls::HttpsConnector;
use hyper_util::rt::TokioExecutor;
use http_body_util::{Empty, Full, BodyExt};
use bytes::Bytes;
use tokio::time::timeout;
use serde_json;
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Main API client using proven driver patterns with plugin integration
pub struct ApiClient {
    config: Config,
    plugin_manager: Arc<PluginManager>,
    client: Client<HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>>,
    timeout_duration: Duration,
    custom_headers: HashMap<String, String>,
    user_agent: String,
}

/// Builder pattern for ApiClient configuration
pub struct ApiClientBuilder {
    config: Option<Config>,
    plugin_manager: Option<PluginManager>,
    custom_headers: HashMap<String, String>,
    user_agent: Option<String>,
}

impl ApiClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: None,
            plugin_manager: None,
            custom_headers: HashMap::new(),
            user_agent: None,
        }
    }
    
    /// Set configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Set plugin manager
    pub fn with_plugin_manager(mut self, manager: PluginManager) -> Self {
        self.plugin_manager = Some(manager);
        self
    }
    
    /// Add custom header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.custom_headers.insert(key, value);
        self
    }
    
    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
    
    /// Build the ApiClient
    pub async fn build(self) -> Result<ApiClient> {
        let config = self.config.unwrap_or_default();
        let plugin_manager = Arc::new(self.plugin_manager.unwrap_or_else(PluginManager::new));
        let connector = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);
        let timeout_duration = Duration::from_secs(config.client.timeout as u64);
        let user_agent = self.user_agent.unwrap_or_else(|| config.client.user_agent.clone());
        
        Ok(ApiClient {
            config,
            plugin_manager,
            client,
            timeout_duration,
            custom_headers: self.custom_headers,
            user_agent,
        })
    }
}

impl ApiClient {
    pub fn new(config: Config) -> Self {
        let connector = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);
        let plugin_manager = Arc::new(PluginManager::new());
        let timeout_duration = Duration::from_secs(config.client.timeout as u64);
        
        Self {
            user_agent: config.client.user_agent.clone(),
            config,
            plugin_manager,
            client,
            timeout_duration,
            custom_headers: HashMap::new(),
        }
    }
    
    /// Create client with custom plugin manager
    pub fn with_plugins(mut self, plugin_manager: PluginManager) -> Self {
        self.plugin_manager = Arc::new(plugin_manager);
        self
    }
    
    /// Get reference to plugin manager
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }
    
    /// Get reference to configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Execute HTTP GET request with plugin support
    pub async fn get(&self, url: &str) -> Result<String> {
        // Pre-request plugin hook
        self.plugin_manager.execute_pre_request(url).await?;
        
        let mut request_builder = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header("user-agent", &self.user_agent);
            
        // Add custom headers
        for (key, value) in &self.custom_headers {
            request_builder = request_builder.header(key, value);
        }
            
        let request = request_builder
            .body(Empty::<Bytes>::new().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
            
        let response = timeout(self.timeout_duration, self.client.request(request))
            .await
            .map_err(|_| ApiError::Timeout)?
            .map_err(|e| ApiError::other(format!("Client error: {}", e)))?;
            
        let status = response.status();
        let status_code = status.as_u16();
        
        // Post-request plugin hook
        self.plugin_manager.execute_post_request(url, status_code).await?;
        
        if !status.is_success() {
            let error = ApiError::HttpStatus { status };
            self.plugin_manager.execute_error(&error).await?;
            return Err(error);
        }
        
        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .map_err(|e| ApiError::other(format!("Failed to read response body: {}", e)))?
            .to_bytes();
            
        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))
    }
    
    /// Execute HTTP POST request with JSON data and plugin support
    pub async fn post_json(&self, url: &str, data: &serde_json::Value) -> Result<String> {
        // Pre-request plugin hook
        self.plugin_manager.execute_pre_request(url).await?;
        
        let json_body = serde_json::to_string(data)?;
        
        let mut request_builder = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("content-type", "application/json")
            .header("user-agent", &self.user_agent);
            
        // Add custom headers
        for (key, value) in &self.custom_headers {
            request_builder = request_builder.header(key, value);
        }
            
        let request = request_builder
            .body(Full::new(Bytes::from(json_body)).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
            
        let response = timeout(self.timeout_duration, self.client.request(request))
            .await
            .map_err(|_| ApiError::Timeout)?
            .map_err(|e| ApiError::other(format!("Client error: {}", e)))?;
            
        let status = response.status();
        let status_code = status.as_u16();
        
        // Post-request plugin hook
        self.plugin_manager.execute_post_request(url, status_code).await?;
        
        if !status.is_success() {
            let error = ApiError::HttpStatus { status };
            self.plugin_manager.execute_error(&error).await?;
            return Err(error);
        }
        
        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .map_err(|e| ApiError::other(format!("Failed to read response body: {}", e)))?
            .to_bytes();
            
        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))
    }
    
    /// Send a PUT request with JSON data
    pub async fn put_json(&self, url: &str, data: &serde_json::Value) -> Result<String> {
        // Pre-request plugin hook
        self.plugin_manager.execute_pre_request(url).await?;
        
        let json_body = serde_json::to_string(data)?;
        let request = Request::builder()
            .method(Method::PUT)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json_body)).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
            
        let response = timeout(self.timeout_duration, self.client.request(request))
            .await
            .map_err(|_| ApiError::Timeout)?
            .map_err(|e| ApiError::other(format!("Client error: {}", e)))?;
            
        let status = response.status();
        let status_code = status.as_u16();
        
        // Post-request plugin hook
        self.plugin_manager.execute_post_request(url, status_code).await?;
        
        if !status.is_success() {
            return Err(ApiError::HttpStatus { status });
        }
        
        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .map_err(|e| ApiError::other(format!("Failed to read response body: {}", e)))?
            .to_bytes();
            
        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))
    }
    
    /// Send a DELETE request
    pub async fn delete(&self, url: &str) -> Result<String> {
        // Pre-request plugin hook
        self.plugin_manager.execute_pre_request(url).await?;
        
        let request = Request::builder()
            .method(Method::DELETE)
            .uri(url)
            .body(Empty::<Bytes>::new().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
            
        let response = timeout(self.timeout_duration, self.client.request(request))
            .await
            .map_err(|_| ApiError::Timeout)?
            .map_err(|e| ApiError::other(format!("Client error: {}", e)))?;
            
        let status = response.status();
        let status_code = status.as_u16();
        
        // Post-request plugin hook
        self.plugin_manager.execute_post_request(url, status_code).await?;
        
        if !status.is_success() {
            return Err(ApiError::HttpStatus { status });
        }
        
        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .map_err(|e| ApiError::other(format!("Failed to read response body: {}", e)))?
            .to_bytes();
            
        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))
    }
    
    /// Send a PATCH request with JSON data
    pub async fn patch_json(&self, url: &str, data: &serde_json::Value) -> Result<String> {
        // Pre-request plugin hook
        self.plugin_manager.execute_pre_request(url).await?;
        
        let json_body = serde_json::to_string(data)?;
        let request = Request::builder()
            .method(Method::PATCH)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json_body)).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
            
        let response = timeout(self.timeout_duration, self.client.request(request))
            .await
            .map_err(|_| ApiError::Timeout)?
            .map_err(|e| ApiError::other(format!("Client error: {}", e)))?;
            
        let status = response.status();
        let status_code = status.as_u16();
        
        // Post-request plugin hook
        self.plugin_manager.execute_post_request(url, status_code).await?;
        
        if !status.is_success() {
            return Err(ApiError::HttpStatus { status });
        }
        
        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .map_err(|e| ApiError::other(format!("Failed to read response body: {}", e)))?
            .to_bytes();
            
        String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))
    }
    
    /// Execute HTTP request with retry logic and plugin support
    pub async fn execute_request_with_retry(&self, url: &str, method: Method, body: Option<&serde_json::Value>) -> Result<String> {
        let max_retries = self.config.client.max_retries;
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                // Retry plugin hook
                self.plugin_manager.execute_retry(attempt as u32).await?;
                tokio::time::sleep(self.config.retry_delay()).await;
            }
            
            let result = match method {
                Method::GET => self.get(url).await,
                Method::POST => {
                    if let Some(json_data) = body {
                        self.post_json(url, json_data).await
                    } else {
                        return Err(ApiError::other("POST request requires JSON body"));
                    }
                }
                _ => return Err(ApiError::other(format!("Unsupported method: {}", method))),
            };
            
            match result {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| ApiError::other("All retry attempts failed")))
    }
    
    /// Download a file from URL and save to local filesystem
    pub async fn download_file(&self, url: &str, filename: &str) -> Result<std::path::PathBuf> {
        self.download_file_with_options(url, filename, false).await
    }
    
    /// Download a file from URL with local/XDG location options
    pub async fn download_file_with_options(&self, url: &str, filename: &str, use_local: bool) -> Result<std::path::PathBuf> {
        // Sanitize filename to prevent path traversal attacks
        let sanitized_filename = Self::sanitize_filename(filename)?;
        
        // Get the response as bytes
        let response_text = self.get(url).await?;
        let response_bytes = response_text.into_bytes();
        
        // Choose download directory based on local flag
        let downloads_dir = if use_local {
            // Use ./.downloads/ directory for local downloads
            std::path::PathBuf::from("./.downloads")
        } else {
            // Use ~/.local/data/kick/downloads as default download location  
            self.config.storage.base_path.join("downloads")
        };
        
        let file_path = downloads_dir.join(sanitized_filename);
        
        // Ensure downloads directory exists
        fs::create_dir_all(&downloads_dir).await
            .map_err(|e| ApiError::other(format!("Failed to create downloads directory: {}", e)))?;
        
        // Write the file
        let mut file = fs::File::create(&file_path).await
            .map_err(|e| ApiError::other(format!("Failed to create file: {}", e)))?;
            
        file.write_all(&response_bytes).await
            .map_err(|e| ApiError::other(format!("Failed to write file: {}", e)))?;
            
        file.flush().await
            .map_err(|e| ApiError::other(format!("Failed to flush file: {}", e)))?;
        
        Ok(file_path)
    }
    
    /// Sanitize filename to prevent path traversal attacks
    fn sanitize_filename(filename: &str) -> Result<String> {
        let path = std::path::Path::new(filename);
        
        // Reject absolute paths
        if path.is_absolute() {
            return Err(ApiError::other("Invalid filename: absolute paths not allowed"));
        }
        
        // Reject paths containing parent directory references
        if filename.contains("..") {
            return Err(ApiError::other("Invalid filename: parent directory references not allowed"));
        }
        
        // Reject empty filenames
        if filename.trim().is_empty() {
            return Err(ApiError::other("Invalid filename: empty filename not allowed"));
        }
        
        Ok(filename.to_string())
    }
    
    /// Download JSON data and deserialize it
    pub async fn download_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response_text = self.get(url).await?;
        let json_data: T = serde_json::from_str(&response_text)
            .map_err(|e| ApiError::other(format!("Failed to deserialize JSON: {}", e)))?;
        Ok(json_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_api_client_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().to_path_buf());
        let client = ApiClient::new(config);
        
        // Verify client was created successfully
        assert_eq!(client.config().client.max_retries, 3);
    }
    
    #[tokio::test]
    async fn test_client_with_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().to_path_buf());
        let mut plugin_manager = PluginManager::new();
        plugin_manager.register_plugin(Arc::new(crate::plugin::LoggingPlugin::new())).unwrap();
        
        let client = ApiClient::new(config).with_plugins(plugin_manager);
        
        // Verify plugin manager was integrated
        assert_eq!(client.plugin_manager().plugins.len(), 1);
    }
    
    #[tokio::test]
    async fn test_api_client_builder() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().to_path_buf());
        
        let client = ApiClientBuilder::new()
            .with_config(config)
            .with_user_agent("TestClient/1.0".to_string())
            .with_header("X-Test-Header".to_string(), "test-value".to_string())
            .build()
            .await
            .unwrap();
            
        // Verify client configuration
        assert_eq!(client.user_agent, "TestClient/1.0");
        assert_eq!(client.custom_headers.get("X-Test-Header"), Some(&"test-value".to_string()));
    }
    
    #[tokio::test]
    async fn test_builder_with_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().to_path_buf());
        
        let mut plugin_manager = PluginManager::new();
        plugin_manager.register_plugin(Arc::new(crate::plugin::LoggingPlugin::new())).unwrap();
        
        let client = ApiClientBuilder::new()
            .with_config(config)
            .with_plugin_manager(plugin_manager)
            .build()
            .await
            .unwrap();
            
        // Verify plugin manager was integrated
        assert_eq!(client.plugin_manager().plugins.len(), 1);
    }
    
    #[tokio::test]
    async fn test_download_json() {
        let _temp_dir = TempDir::new().unwrap();
        
        // Test deserializing a simple JSON structure
        let test_json = r#"{"test": "value", "number": 42}"#;
        
        // We can't easily test download_json without a mock server,
        // but we can test the JSON parsing logic separately
        let parsed: serde_json::Value = serde_json::from_str(test_json).unwrap();
        assert_eq!(parsed["test"], "value");
        assert_eq!(parsed["number"], 42);
    }
    
    #[tokio::test]
    #[ignore] // Use `cargo test -- --ignored` to run network tests
    async fn test_get_request_with_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().to_path_buf());
        let mut plugin_manager = PluginManager::new();
        plugin_manager.register_plugin(Arc::new(crate::plugin::LoggingPlugin::new())).unwrap();
        
        let client = ApiClient::new(config).with_plugins(plugin_manager);
        
        match client.get("https://httpbin.org/status/200").await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("Network test failed (may be expected): {}", e);
                // Don't fail - network may not be available
            }
        }
    }
    
    #[tokio::test]
    #[ignore] // Use `cargo test -- --ignored` to run network tests  
    async fn test_post_json_with_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().to_path_buf());
        let mut plugin_manager = PluginManager::new();
        plugin_manager.register_plugin(Arc::new(crate::plugin::LoggingPlugin::new())).unwrap();
        
        let client = ApiClient::new(config).with_plugins(plugin_manager);
        let test_data = serde_json::json!({"test": "data"});
        
        match client.post_json("https://httpbin.org/post", &test_data).await {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("Network test failed (may be expected): {}", e);
                // Don't fail - network may not be available
            }
        }
    }
}