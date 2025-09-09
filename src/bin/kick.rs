use kick::prelude::*;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let config = Config::default();
    
    match cli.command {
        Commands::Get { url, headers, user_agent, save, pretty, verbose } => {
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
                        std::fs::write(&filename, &output)?;
                        println!("💾 Saved to: {}", filename);
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
                        std::fs::write(&filename, &output)?;
                        println!("💾 Saved to: {}", filename);
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
        
        Commands::Download { url, output, headers, user_agent, verbose } => {
            let client = build_client(config, headers, user_agent, verbose).await?;
            
            println!("📥 Downloading {}", url);
            
            match client.download_file(&url, &output).await {
                Ok(path) => {
                    println!("✅ Downloaded to: {:?}", path);
                }
                Err(e) => {
                    eprintln!("❌ Download failed: {}", e);
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
    
    // Parse headers
    for header in headers {
        if let Some((key, value)) = header.split_once(':') {
            builder = builder.with_header(key.trim().to_string(), value.trim().to_string());
        } else {
            return Err(ApiError::other(format!("Invalid header format: {}. Use 'Key:Value'", header)));
        }
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