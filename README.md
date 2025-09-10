# Modular API Client

A lightweight, extensible HTTP client built with Rust and Hyper, designed for modularity and plugin-based extensibility. Perfect foundation for building agent systems and complex API integrations.

## Features

- 🚀 **High Performance**: Built on Hyper for async HTTP/1.1 and HTTP/2 support
- 🔌 **Plugin System**: Extensible architecture with plugin hooks for requests, responses, errors, and streaming
- 💾 **Smart Storage**: File download/upload with XDG Base Directory support
- 🌊 **Stream Processing**: Built-in stream handling with buffering, chunking, and rate limiting
- ⚙️ **Configurable**: TOML-based configuration with sensible defaults
- 🔄 **Retry Logic**: Automatic retry with exponential backoff
- 📊 **Observability**: Built-in logging and metrics collection
- 🛡️ **Error Handling**: Comprehensive error types and handling

## Architecture

The library is organized into modular components:

```
src/
├── lib.rs          # Main library exports
├── client.rs       # HTTP client implementation
├── config.rs       # Configuration management
├── plugin.rs       # Plugin system and built-in plugins
├── storage.rs      # File storage and download management
├── streaming.rs    # Stream processing utilities
├── error.rs        # Error types and handling
└── main.rs         # Usage examples
```

### Core Components

#### 1. ApiClient
The main HTTP client with plugin support, retry logic, and convenience methods.

#### 2. Plugin System
Extensible plugin architecture with hooks for:
- Pre/post request processing
- Response handling
- Error handling
- Stream processing
- Retry logic

#### 3. Storage Manager
Handles file operations with:
- XDG Base Directory compliance
- Progress tracking
- Size limits
- Temporary file management

#### 4. Stream Handler
Advanced streaming capabilities:
- Response streaming
- Buffered/chunked streams
- Rate limiting
- Progress tracking

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
modular-api-client = "0.1.0"
```

### Basic Usage

```rust
use modular_api_client::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with default configuration
    let client = ApiClientBuilder::new()
        .with_timeout(std::time::Duration::from_secs(30))
        .with_user_agent("MyApp/1.0.0".to_string())
        .build()
        .await?;
    
    // Simple GET request
    let response = client.get("https://api.example.com/data").await?;
    
    // Download and save file
    let path = client.download_file(
        "https://example.com/large-file.zip", 
        "downloads/file.zip"
    ).await?;
    
    // Download JSON data
    let data: serde_json::Value = client
        .download_json("https://api.example.com/json")
        .await?;
    
    Ok(())
}
```

### Configuration

The client uses XDG Base Directory specification for configuration:

```rust
// Load existing config or create default
let mut config = Config::load().unwrap_or_default();

// Customize settings
config.client.timeout = 60;
config.client.max_retries = 5;
config.storage.max_file_size = 500 * 1024 * 1024; // 500MB

// Save for future use
config.save()?;
```

Configuration file location (Linux): `~/.config/modular-api-client/config.toml`

### Plugin System

Create custom plugins by implementing the `Plugin` trait:

```rust
use async_trait::async_trait;

pub struct AuthPlugin {
    token: String,
}

#[async_trait]
impl Plugin for AuthPlugin {
    fn name(&self) -> &str { "auth" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Adds authentication headers" }
    
    async fn initialize(&mut self, config: &serde_json::Value) -> Result<()> {
        // Initialize from config
        Ok(())
    }
    
    fn handles_hook(&self, hook: &PluginHook) -> bool {
        matches!(hook, PluginHook::PreRequest)
    }
    
    async fn handle_pre_request(
        &self,
        request: &mut Request<Body>,
        _context: &PluginContext,
    ) -> Result<()> {
        request.headers_mut().insert(
            "authorization",
            format!("Bearer {}", self.token).parse().unwrap()
        );
        Ok(())
    }
}

// Register and use
let mut plugin_manager = PluginManager::new();
plugin_manager.register_plugin(Arc::new(AuthPlugin { 
    token: "your-token".to_string() 
}))?;

let client = ApiClientBuilder::new()
    .with_plugin_manager(plugin_manager)
    .build()
    .await?;
```

### Built-in Plugins

#### LoggingPlugin
Logs HTTP requests and responses:

```rust
let logging_plugin = Arc::new(LoggingPlugin::new());
plugin_manager.register_plugin(logging_plugin)?;
```

#### RateLimitPlugin
Implements rate limiting:

```rust
let rate_limit = Arc::new(RateLimitPlugin::new(60)); // 60 requests/minute
plugin_manager.register_plugin(rate_limit)?;
```

### Stream Processing

```rust
// Stream with custom processing
client.stream_response("https://api.example.com/stream", |chunk| async move {
    println!("Received {} bytes", chunk.len());
    // Process chunk
    Ok(())
}).await?;

// Rate-limited streaming
let response = client.get("https://example.com/large-file").await?;
let stream = client.streams().response_to_stream(response);
let rate_limited = client.streams().create_rate_limited_stream(stream, 1024); // 1KB/s

// Collect with progress tracking
let progress_stream = client.streams().track_progress(rate_limited, |bytes, total| {
    println!("Downloaded: {} bytes", bytes);
});

let data = client.streams().collect_stream(progress_stream, None).await?;
```

### Storage Operations

```rust
// Save data
let path = client.storage().save_string("Hello, World!", "greeting.txt").await?;

// Save JSON
let data = serde_json::json!({"key": "value"});
client.storage().save_json(&data, "data.json").await?;

// Load data
let content = client.storage().load_string("greeting.txt").await?;

// Stream to file with progress
client.storage().save_stream(
    stream,
    "large-file.bin",
    Some(Box::new(|bytes, total| {
        println!("Progress: {} bytes", bytes);
    }))
).await?;

// Storage statistics
let stats = client.storage().storage_stats().await?;
println!("Total storage: {} bytes", stats.total_size);
```

## Advanced Usage

### Custom Error Handling

```rust
match client.get("https://api.example.com").await {
    Ok(response) => { /* handle success */ },
    Err(ApiError::HttpStatus { status }) => {
        println!("HTTP error: {}", status);
    },
    Err(ApiError::Timeout) => {
        println!("Request timed out");
    },
    Err(ApiError::RateLimit) => {
        println!("Rate limit exceeded");
    },
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```

### Agent System Foundation

This client is designed to be a foundation for agent systems. Here's how you might extend it:

```rust
pub struct Agent {
    client: ApiClient,
    state: AgentState,
}

impl Agent {
    pub async fn process_task(&mut self, task: Task) -> Result<TaskResult> {
        // Use the client for API calls
        // Leverage plugins for cross-cutting concerns
        // Use storage for persisting state
        // Use streaming for large data processing
        todo!()
    }
}
```

## Configuration Reference

```toml
[client]
user_agent = "ModularApiClient/0.1.0"
timeout = 30
max_retries = 3
retry_delay = 1000
base_url = "https://api.example.com"

[client.default_headers]
"X-API-Version" = "v1"

[storage]
base_path = "/home/user/.local/share/modular-api-client"
temp_path = "/home/user/.cache/modular-api-client"
max_file_size = 104857600  # 100MB
cleanup_on_exit = true

[plugins]
enabled_plugins = ["logging", "rate_limiter"]

[streaming]
buffer_size = 8192
chunk_size = 4096
max_concurrent_streams = 10
stream_timeout = 300
```

## Security Architecture

KICK follows a **flexible-by-default** security model designed for development productivity and operational safety:

### Current Security Model
- **Input Validation**: Header and path sanitization to prevent injection attacks
- **Safe File Operations**: XDG compliance with path traversal protection  
- **Flexible Networking**: Supports localhost, private networks, and public APIs
- **TLS Support**: HTTPS connections with proper certificate validation

### Security Features
- **Header Validation**: Prevents CRLF injection and malformed headers
- **Path Sanitization**: Blocks directory traversal attempts in file operations
- **Safe Downloads**: Automatic filename sanitization and secure storage
- **Error Context**: Security-conscious error messages that don't leak sensitive information

### Development-Friendly Approach
KICK intentionally allows connections to:
- `localhost` and `127.0.0.1` for local development
- Private IP ranges (`192.168.x.x`, `10.x.x.x`) for internal APIs
- Local domains (`.local`) for testing environments

This design prioritizes developer productivity while maintaining essential security boundaries.

### Security Roadmap
Future versions will include:
- Optional SSRF protection for production environments
- Enhanced certificate validation controls
- Request/response content filtering
- Advanced rate limiting and abuse prevention

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions welcome! Please see CONTRIBUTING.md for guidelines.
