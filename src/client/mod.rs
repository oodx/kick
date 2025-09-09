use crate::config::Config;
use crate::error::{ApiError, Result};
use crate::plugin::{PluginContext, PluginHook, PluginManager};
use crate::storage::StorageManager;
use crate::streaming::StreamHandler;
use bytes::Bytes;
use futures::Stream;
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper::{Request, Response, Method, Uri};
use hyper::body::Body;
use http_body_util::{Empty, Full, combinators::BoxBody, BodyExt};
use hyper_tls::HttpsConnector;
use hyper_util::rt::TokioExecutor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, timeout};

pub struct ApiClient {
    config: Config,
    plugin_manager: Arc<PluginManager>,
    storage_manager: Arc<StorageManager>,
    stream_handler: Arc<StreamHandler>,
    client: Client<HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
}

impl ApiClient {
    pub fn new(config: Config) -> Self {
        let connector = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build(connector);
        let storage_manager = Arc::new(StorageManager::new(config.clone()));
        let stream_handler = Arc::new(StreamHandler::new(config.clone()));
        let plugin_manager = Arc::new(PluginManager::new());

        Self {
            config,
            plugin_manager,
            storage_manager,
            stream_handler,
            client,
        }
    }
    
    pub fn with_plugins(mut self, plugin_manager: PluginManager) -> Self {
        self.plugin_manager = Arc::new(plugin_manager);
        self
    }
    
    /// Execute HTTP request with full plugin support
    pub async fn execute_request<B>(&self, mut request: Request<B>) -> Result<Response<Incoming>> 
    where 
        B: hyper::body::Body + Send + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        // Apply default headers
        for (name, value) in &self.config.client.default_headers {
            if !request.headers().contains_key(name) {
                request.headers_mut().insert(
                    hyper::header::HeaderName::from_str(name)
                        .map_err(|e| ApiError::other(format!("Invalid header name: {}", e)))?,
                    hyper::header::HeaderValue::from_str(value)
                        .map_err(|e| ApiError::other(format!("Invalid header value: {}", e)))?,
                );
            }
        }
        
        // Set User-Agent if not present
        if !request.headers().contains_key("user-agent") {
            request.headers_mut().insert(
                "user-agent",
                hyper::header::HeaderValue::from_str(&self.config.client.user_agent)
                    .map_err(|e| ApiError::other(format!("Invalid user agent: {}", e)))?,
            );
        }
        
        // Pre-request plugin hook
        let context = PluginContext::new(PluginHook::PreRequest);
        self.plugin_manager
            .execute_hook(PluginHook::PreRequest, &context, |plugin, ctx| {
                Box::pin(async move {
                    plugin.handle_pre_request(&mut request, ctx).await
                })
            })
            .await?;
        
        // Execute request with retries
        let mut last_error = None;
        for attempt in 0..=self.config.client.max_retries {
            if attempt > 0 {
                let retry_context = PluginContext::new(PluginHook::OnRetry)
                    .with_metadata("attempt", serde_json::Value::from(attempt));
                
                self.plugin_manager
                    .execute_hook(PluginHook::OnRetry, &retry_context, |plugin, ctx| {
                        Box::pin(async move {
                            plugin.handle_retry(attempt, ctx).await
                        })
                    })
                    .await?;
                
                sleep(self.config.retry_delay()).await;
            }
            
            match self.execute_single_request(&request).await {
                Ok(response) => {
                    // Post-request plugin hook
                    let post_context = PluginContext::new(PluginHook::PostRequest);
                    self.plugin_manager
                        .execute_hook(PluginHook::PostRequest, &post_context, |plugin, ctx| {
                            Box::pin(async move {
                                plugin.handle_post_request(&request, ctx).await
                            })
                        })
                        .await?;
                    
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(ApiError::other(e.to_string()));
                    
                    // Error plugin hook
                    let error_context = PluginContext::new(PluginHook::OnError)
                        .with_metadata("attempt", serde_json::Value::from(attempt));
                    
                    self.plugin_manager
                        .execute_hook(PluginHook::OnError, &error_context, |plugin, ctx| {
                            Box::pin(async move {
                                plugin.handle_error(&e, ctx).await
                            })
                        })
                        .await?;
                    
                    // Don't retry on certain errors
                    if matches!(e, ApiError::HttpStatus { status } if status.is_client_error()) {
                        return Err(e);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| ApiError::other("All retry attempts failed")))
    }
    
    /// Execute single HTTP request without retries
    async fn execute_single_request(&self, request: &Request<BoxBody<Bytes, hyper::Error>>) -> Result<Response<Incoming>> {
        let uri = request.uri().clone();
        let scheme = uri.scheme_str().unwrap_or("https");
        let host = uri.host().ok_or_else(|| ApiError::other("No host in URI"))?;
        let port = uri.port_u16().unwrap_or(if scheme == "https" { 443 } else { 80 });
        
        // Connect to the server
        let stream = timeout(
            self.config.timeout(),
            TcpStream::connect(format!("{}:{}", host, port))
        ).await
        .map_err(|_| ApiError::Timeout)?
        .map_err(ApiError::Io)?;
        
        let io = TokioIo::new(stream);
        
        // For HTTPS, wrap with TLS
        let (mut sender, conn) = if scheme == "https" {
            let tls_stream = self.connector.call(uri.clone()).await
                .map_err(|e| ApiError::other(format!("TLS connection failed: {}", e)))?;
            http1::handshake(tls_stream).await?
        } else {
            http1::handshake(io).await?
        };
        
        // Spawn the connection task
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                tracing::error!("Connection failed: {:?}", err);
            }
        });
        
        // Clone the request for sending
        let (parts, body) = request.clone().into_parts();
        let new_request = Request::from_parts(parts, body);
        
        // Send the request with timeout
        let response = timeout(
            self.config.timeout(),
            sender.send_request(new_request)
        ).await
        .map_err(|_| ApiError::Timeout)?
        .map_err(ApiError::Http)?;
        
        // Check status code
        if !response.status().is_success() {
            return Err(ApiError::HttpStatus { 
                status: response.status() 
            });
        }
        
        Ok(response)
    }
    
    /// Convenience method for GET requests
    pub async fn get(&self, url: &str) -> Result<Response<Incoming>> {
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .body(Empty::<Bytes>::new().boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
        
        self.execute_request(request).await
    }
    
    /// Convenience method for POST requests with JSON body
    pub async fn post_json<T: Serialize>(&self, url: &str, body: &T) -> Result<Response<Incoming>> {
        let json_body = serde_json::to_string(body)?;
        
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_body)).boxed())
            .map_err(|e| ApiError::other(format!("Failed to build request: {}", e)))?;
        
        self.execute_request(request).await
    }
    
    /// Download file and save to storage
    pub async fn download_file(&self, url: &str, filename: &str) -> Result<std::path::PathBuf> {
        let response = self.get(url).await?;
        
        // Convert response to stream
        let stream = self.stream_handler.response_to_stream(response);
        
        // Save stream to file with progress tracking
        self.storage_manager
            .save_stream(
                stream,
                filename,
                Some(Box::new(|bytes, _total| {
                    tracing::info!("Downloaded {} bytes", bytes);
                })),
            )
            .await
    }
    
    /// Download and return as bytes
    pub async fn download_bytes(&self, url: &str) -> Result<Bytes> {
        let response = self.get(url).await?;
        let stream = self.stream_handler.response_to_stream(response);
        self.stream_handler.collect_stream(stream, None).await
    }
    
    /// Download and parse as JSON
    pub async fn download_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let bytes = self.download_bytes(url).await?;
        let text = String::from_utf8(bytes.to_vec())
            .map_err(|e| ApiError::other(format!("Invalid UTF-8: {}", e)))?;
        let data = serde_json::from_str(&text)?;
        Ok(data)
    }
    
    /// Stream response with custom processing
    pub async fn stream_response<F, Fut>(
        &self,
        url: &str,
        processor: F,
    ) -> Result<()>
    where
        F: Fn(Bytes) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let response = self.get(url).await?;
        let mut stream = Box::pin(self.stream_handler.response_to_stream(response));
        
        while let Some(chunk_result) = futures::StreamExt::next(&mut stream).await {
            let chunk = chunk_result?;
            processor(chunk).await?;
        }
        
        Ok(())
    }
    
    /// Get reference to storage manager
    pub fn storage(&self) -> &StorageManager {
        &self.storage_manager
    }
    
    /// Get reference to stream handler
    pub fn streams(&self) -> &StreamHandler {
        &self.stream_handler
    }
    
    /// Get reference to plugin manager
    pub fn plugins(&self) -> &PluginManager {
        &self.plugin_manager
    }
    
    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// Builder for creating ApiClient with custom configuration
pub struct ApiClientBuilder {
    config: Config,
    plugin_manager: Option<PluginManager>,
}

impl ApiClientBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            plugin_manager: None,
        }
    }
    
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
    
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.client.timeout = timeout.as_secs();
        self
    }
    
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.config.client.user_agent = user_agent;
        self
    }
    
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.config.client.base_url = Some(base_url);
        self
    }
    
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.config.client.default_headers.insert(name, value);
        self
    }
    
    pub fn with_plugin_manager(mut self, plugin_manager: PluginManager) -> Self {
        self.plugin_manager = Some(plugin_manager);
        self
    }
    
    pub async fn build(self) -> Result<ApiClient> {
        // Ensure directories exist
        self.config.ensure_directories()?;
        
        let mut client = ApiClient::new(self.config);
        
        if let Some(plugin_manager) = self.plugin_manager {
            client = client.with_plugins(plugin_manager);
        }
        
        Ok(client)
    }
}

impl Default for ApiClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}