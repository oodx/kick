use kick::prelude::*;
use kick::sec::{HeaderValidator, UrlValidator, PathValidator};
use std::sync::Arc;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kick")]
#[command(about = "Kick API Client - A lightweight HTTP client with plugin support")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Make a GET request
    Get {
        /// URL to request
        url: String,
        /// Custom headers (format: "Key:Value")
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
        /// User agent string
        #[arg(short = 'A', long = "user-agent")]
        user_agent: Option<String>,
        /// Save response to file
        #[arg(short = 's', long = "save")]
        save: Option<String>,
        /// Pretty print JSON responses
        #[arg(short = 'p', long = "pretty")]
        pretty: bool,
        /// Verbose output with plugin logging
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Make a POST request with JSON data
    Post {
        /// URL to request
        url: String,
        /// JSON data to post
        #[arg(short = 'd', long = "data")]
        data: String,
        /// Custom headers (format: "Key:Value")
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
        /// User agent string
        #[arg(short = 'A', long = "user-agent")]
        user_agent: Option<String>,
        /// Save response to file
        #[arg(short = 's', long = "save")]
        save: Option<String>,
        /// Pretty print JSON responses
        #[arg(short = 'p', long = "pretty")]
        pretty: bool,
        /// Verbose output with plugin logging
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Download file from URL
    Download {
        /// URL to download from
        url: String,
        /// Output filename
        #[arg(short = 'o', long = "output")]
        output: String,
        /// Download to local ./.downloads/ directory instead of XDG location
        #[arg(short = 'l', long = "local")]
        local: bool,
        /// Custom headers (format: "Key:Value")
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
        /// User agent string
        #[arg(short = 'A', long = "user-agent")]
        user_agent: Option<String>,
        /// Verbose output with plugin logging
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Make a PUT request with JSON data
    Put {
        /// URL to request
        url: String,
        /// JSON data to put
        #[arg(short = 'd', long = "data")]
        data: String,
        /// Custom headers (format: "Key:Value")
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
        /// User agent string
        #[arg(short = 'A', long = "user-agent")]
        user_agent: Option<String>,
        /// Save response to file
        #[arg(short = 's', long = "save")]
        save: Option<String>,
        /// Pretty print JSON responses
        #[arg(short = 'p', long = "pretty")]
        pretty: bool,
        /// Verbose output with plugin logging
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Make a DELETE request
    Delete {
        /// URL to request
        url: String,
        /// Custom headers (format: "Key:Value")
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
        /// User agent string
        #[arg(short = 'A', long = "user-agent")]
        user_agent: Option<String>,
        /// Save response to file
        #[arg(short = 's', long = "save")]
        save: Option<String>,
        /// Pretty print JSON responses
        #[arg(short = 'p', long = "pretty")]
        pretty: bool,
        /// Verbose output with plugin logging
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
    /// Make a PATCH request with JSON data
    Patch {
        /// URL to request
        url: String,
        /// JSON data to patch
        #[arg(short = 'd', long = "data")]
        data: String,
        /// Custom headers (format: "Key:Value")
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
        /// User agent string
        #[arg(short = 'A', long = "user-agent")]
        user_agent: Option<String>,
        /// Save response to file
        #[arg(short = 's', long = "save")]
        save: Option<String>,
        /// Pretty print JSON responses
        #[arg(short = 'p', long = "pretty")]
        pretty: bool,
        /// Verbose output with plugin logging
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },
}

/// Sanitize save filename to prevent path traversal attacks
fn sanitize_save_filename(filename: &str) -> Result<std::path::PathBuf> {
    PathValidator::safe_current_dir_path(filename)
        .map_err(|e| ApiError::other(format!("Save path validation failed: {}", e)).into())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let config = Config::default();
    
    match cli.command {
        Commands::Get { url, headers, user_agent, save, pretty, verbose } => {
            // Validate URL for SSRF protection
            let _validated_url = UrlValidator::validate(&url)
                .map_err(|e| ApiError::other(format!("URL validation failed: {}", e)))?;
            
            let client = build_client(config, headers, user_agent, verbose).await?;
            
            println!("🌐 GET {}", url);
            
            match client.get(&url).await {
                Ok(response) => {
                    println!("✅ Success ({} chars)", response.len());
                    
                    let output = if pretty {
                        format_json(&response).unwrap_or(response)
                    } else {
                        response
                    };
                    
                    if let Some(filename) = save {
                        let safe_filename = sanitize_save_filename(&filename)?;
                        std::fs::write(&safe_filename, &output)?;
                        println!("💾 Saved to: {}", safe_filename.display());
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Request failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Post { url, data, headers, user_agent, save, pretty, verbose } => {
            // Validate URL for SSRF protection
            let _validated_url = UrlValidator::validate(&url)
                .map_err(|e| ApiError::other(format!("URL validation failed: {}", e)))?;
            
            let client = build_client(config, headers, user_agent, verbose).await?;
            
            println!("📤 POST {}", url);
            
            let json_data: serde_json::Value = serde_json::from_str(&data)
                .map_err(|e| ApiError::other(format!("Invalid JSON data: {}", e)))?;
            
            match client.post_json(&url, &json_data).await {
                Ok(response) => {
                    println!("✅ Success ({} chars)", response.len());
                    
                    let output = if pretty {
                        format_json(&response).unwrap_or(response)
                    } else {
                        response
                    };
                    
                    if let Some(filename) = save {
                        let safe_filename = sanitize_save_filename(&filename)?;
                        std::fs::write(&safe_filename, &output)?;
                        println!("💾 Saved to: {}", safe_filename.display());
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("❌ POST failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Download { url, output, local, headers, user_agent, verbose } => {
            // Validate URL for SSRF protection
            let _validated_url = UrlValidator::validate(&url)
                .map_err(|e| ApiError::other(format!("URL validation failed: {}", e)))?;
            
            let client = build_client(config, headers, user_agent, verbose).await?;
            
            println!("📥 Downloading {}", url);
            
            match client.download_file_with_options(&url, &output, local).await {
                Ok(path) => {
                    println!("✅ Downloaded to: {:?}", path);
                }
                Err(e) => {
                    eprintln!("❌ Download failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Put { url, data, headers, user_agent, save, pretty, verbose } => {
            // Validate URL for SSRF protection
            let _validated_url = UrlValidator::validate(&url)
                .map_err(|e| ApiError::other(format!("URL validation failed: {}", e)))?;
            
            let client = build_client(config, headers, user_agent, verbose).await?;
            
            println!("🔄 PUT {}", url);
            
            let json_data: serde_json::Value = serde_json::from_str(&data)
                .map_err(|e| ApiError::other(format!("Invalid JSON: {}", e)))?;
                
            match client.put_json(&url, &json_data).await {
                Ok(response) => {
                    println!("✅ Success ({} chars)", response.len());
                    
                    let output = if pretty {
                        match serde_json::from_str::<serde_json::Value>(&response) {
                            Ok(parsed) => serde_json::to_string_pretty(&parsed).unwrap_or(response),
                            Err(_) => response
                        }
                    } else {
                        response
                    };
                    
                    if let Some(filename) = save {
                        let safe_filename = sanitize_save_filename(&filename)?;
                        std::fs::write(&safe_filename, &output)?;
                        println!("💾 Saved to: {}", safe_filename.display());
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("❌ PUT failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Delete { url, headers, user_agent, save, pretty, verbose } => {
            // Validate URL for SSRF protection
            let _validated_url = UrlValidator::validate(&url)
                .map_err(|e| ApiError::other(format!("URL validation failed: {}", e)))?;
            
            let client = build_client(config, headers, user_agent, verbose).await?;
            
            println!("🗑️ DELETE {}", url);
            
            match client.delete(&url).await {
                Ok(response) => {
                    println!("✅ Success ({} chars)", response.len());
                    
                    let output = if pretty {
                        match serde_json::from_str::<serde_json::Value>(&response) {
                            Ok(parsed) => serde_json::to_string_pretty(&parsed).unwrap_or(response),
                            Err(_) => response
                        }
                    } else {
                        response
                    };
                    
                    if let Some(filename) = save {
                        let safe_filename = sanitize_save_filename(&filename)?;
                        std::fs::write(&safe_filename, &output)?;
                        println!("💾 Saved to: {}", safe_filename.display());
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("❌ DELETE failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Patch { url, data, headers, user_agent, save, pretty, verbose } => {
            // Validate URL for SSRF protection
            let _validated_url = UrlValidator::validate(&url)
                .map_err(|e| ApiError::other(format!("URL validation failed: {}", e)))?;
            
            let client = build_client(config, headers, user_agent, verbose).await?;
            
            println!("🔧 PATCH {}", url);
            
            let json_data: serde_json::Value = serde_json::from_str(&data)
                .map_err(|e| ApiError::other(format!("Invalid JSON: {}", e)))?;
                
            match client.patch_json(&url, &json_data).await {
                Ok(response) => {
                    println!("✅ Success ({} chars)", response.len());
                    
                    let output = if pretty {
                        match serde_json::from_str::<serde_json::Value>(&response) {
                            Ok(parsed) => serde_json::to_string_pretty(&parsed).unwrap_or(response),
                            Err(_) => response
                        }
                    } else {
                        response
                    };
                    
                    if let Some(filename) = save {
                        let safe_filename = sanitize_save_filename(&filename)?;
                        std::fs::write(&safe_filename, &output)?;
                        println!("💾 Saved to: {}", safe_filename.display());
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("❌ PATCH failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}

async fn build_client(
    config: Config,
    headers: Vec<String>,
    user_agent: Option<String>,
    verbose: bool,
) -> Result<ApiClient> {
    let mut plugin_manager = PluginManager::new();
    
    if verbose {
        plugin_manager.register_plugin(Arc::new(LoggingPlugin::new()))?;
    }
    
    let mut builder = ApiClientBuilder::new().with_config(config);
    
    if let Some(ua) = user_agent {
        builder = builder.with_user_agent(ua);
    }
    
    // Parse and validate headers
    for header in headers {
        let (key, value) = HeaderValidator::parse_and_validate(&header)
            .map_err(|e| ApiError::other(format!("Header validation failed: {}", e)))?;
        builder = builder.with_header(key, value);
    }
    
    if !plugin_manager.plugins.is_empty() {
        builder = builder.with_plugin_manager(plugin_manager);
    }
    
    builder.build().await
}

fn format_json(text: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|json| serde_json::to_string_pretty(&json).ok())
}