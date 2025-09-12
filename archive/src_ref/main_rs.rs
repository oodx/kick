use modular_api_client::prelude::*;
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();
    
    // Load or create configuration
    let mut config = Config::load().unwrap_or_default();
    
    // Customize configuration
    config.client.user_agent = "MyApp/1.0.0".to_string();
    config.client.timeout = 60; // 60 seconds
    
    // Save configuration for future use
    config.save()?;
    
    // Set up plugin manager with example plugins
    let mut plugin_manager = PluginManager::new();
    
    // Register logging plugin
    let logging_plugin = Arc::new(modular_api_client::plugin::LoggingPlugin::new());
    plugin_manager.register_plugin(logging_plugin)?;
    
    // Register rate limiting plugin
    let rate_limit_plugin = Arc::new(modular_api_client::plugin::RateLimitPlugin::new(60));
    plugin_manager.register_plugin(rate_limit_plugin)?;
    
    // Build the client
    let client = ApiClientBuilder::new()
        .with_config(config)
        .with_user_agent("ModularClient/Example".to_string())
        .with_header("X-Custom-Header".to_string(), "example-value".to_string())
        .with_plugin_manager(plugin_manager)
        .build()
        .await?;
    
    // Example 1: Simple GET request
    println!("=== Example 1: Simple GET request ===");
    match client.get("https://httpbin.org/get").await {
        Ok(response) => {
            println!("Status: {}", response.status());
            let body = client.stream_handler.collect_stream(
                client.stream_handler.response_to_stream(response),
                None
            ).await?;
            println!("Response length: {} bytes", body.len());
        }
        Err(e) => println!("Request failed: {}", e),
    }
    
    // Example 2: Download and save file
    println!("\n=== Example 2: Download and save file ===");
    match client.download_file("https://httpbin.org/json", "example.json").await {
        Ok(path) => println!("File saved to: {:?}", path),
        Err(e) => println!("Download failed: {}", e),
    }
    
    // Example 3: Download JSON data
    println!("\n=== Example 3: Download JSON data ===");
    match client.download_json::<serde_json::Value>("https://httpbin.org/json").await {
        Ok(data) => println!("JSON data: {}", serde_json::to_string_pretty(&data)?),
        Err(e) => println!("JSON download failed: {}", e),
    }
    
    // Example 4: Stream processing
    println!("\n=== Example 4: Stream processing ===");
    client.stream_response("https://httpbin.org/stream/5", |chunk| async move {
        println!("Received chunk of {} bytes", chunk.len());
        // Process chunk here
        Ok(())
    }).await?;
    
    // Example 5: POST request with JSON
    println!("\n=== Example 5: POST request with JSON ===");
    let post_data = serde_json::json!({
        "key": "value",
        "number": 42,
        "array": [1, 2, 3]
    });
    
    match client.post_json("https://httpbin.org/post", &post_data).await {
        Ok(response) => println!("POST status: {}", response.status()),
        Err(e) => println!("POST failed: {}", e),
    }
    
    // Example 6: Storage operations
    println!("\n=== Example 6: Storage operations ===");
    
    // Save some data
    let test_data = "Hello, World!";
    let path = client.storage().save_string(test_data, "test.txt").await?;
    println!("Saved test data to: {:?}", path);
    
    // Load it back
    let loaded_data = client.storage().load_string("test.txt").await?;
    println!("Loaded data: {}", loaded_data);
    
    // List files
    let files = client.storage().list_files(None).await?;
    println!("Files in storage: {:?}", files);
    
    // Get storage stats
    let stats = client.storage().storage_stats().await?;
    println!("Storage stats: {:?}", stats);
    
    // Example 7: Using streams with rate limiting
    println!("\n=== Example 7: Rate-limited streaming ===");
    let response = client.get("https://httpbin.org/bytes/1024").await?;
    let stream = client.stream_handler.response_to_stream(response);
    let rate_limited = client.stream_handler.create_rate_limited_stream(stream, 1024); // 1KB/s
    let progress_stream = client.stream_handler.track_progress(rate_limited, |bytes, _| {
        println!("Progress: {} bytes", bytes);
    });
    
    let collected = client.stream_handler.collect_stream(progress_stream, None).await?;
    println!("Rate-limited download completed: {} bytes", collected.len());
    
    // Cleanup temporary files if configured
    if client.config().storage.cleanup_on_exit {
        client.storage().cleanup_temp_files().await?;
        println!("Cleaned up temporary files");
    }
    
    println!("\n=== All examples completed successfully! ===");
    Ok(())
}

// Additional example: Custom plugin
pub struct CustomMetricsPlugin {
    request_count: std::sync::atomic::AtomicU64,
    error_count: std::sync::atomic::AtomicU64,
}

impl CustomMetricsPlugin {
    pub fn new() -> Self {
        Self {
            request_count: std::sync::atomic::AtomicU64::new(0),
            error_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    pub fn get_metrics(&self) -> (u64, u64) {
        (
            self.request_count.load(std::sync::atomic::Ordering::Relaxed),
            self.error_count.load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}

#[async_trait]
impl Plugin for CustomMetricsPlugin {
    fn name(&self) -> &str {
        "metrics"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Collects request and error metrics"
    }
    
    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    fn handles_hook(&self, hook: &modular_api_client::plugin::PluginHook) -> bool {
        matches!(hook, 
            modular_api_client::plugin::PluginHook::PreRequest | 
            modular_api_client::plugin::PluginHook::OnError
        )
    }
    
    async fn handle_pre_request(
        &self,
        _request: &mut hyper::Request<hyper::body::Body>,
        _context: &modular_api_client::plugin::PluginContext,
    ) -> Result<()> {
        self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    async fn handle_error(
        &self,
        _error: &ApiError,
        _context: &modular_api_client::plugin::PluginContext,
    ) -> Result<()> {
        self.error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}