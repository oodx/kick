use kick::prelude::*;
use kick::sec::{HeaderValidator, UrlValidator, PathValidator};
use std::sync::Arc;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kick")]
#[command(about = "Kick API Client - A lightweight HTTP client with plugin support")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(long_about = None)]
#[command(disable_version_flag = true)]
#[command(help_template = r#"
{before-help}{name} v{version}
{about}

{usage-heading} {usage}

{all-args}{after-help}
"#)]
struct Cli {
    /// Show version and license information
    #[arg(short = 'V', long = "version", action = clap::ArgAction::SetTrue)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
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

/// Load ASCII logo from logo.txt file
fn load_logo() -> String {
    match std::fs::read_to_string("logo.txt") {
        Ok(logo) => logo,
        Err(_) => "KICK".to_string(), // Fallback if logo.txt not found
    }
}

/// Show comprehensive help or help for specific command
fn show_help(command: Option<String>) {
    match command.as_deref() {
        Some("get") => {
            println!("KICK GET Command Help\n");
            println!("Make a GET request to the specified URL\n");
            println!("Usage: kick get [OPTIONS] <URL>\n");
            println!("Arguments:");
            println!("  <URL>  URL to request\n");
            println!("Options:");
            println!("  -H, --header <HEADER>     Custom headers (format: \"Key:Value\")");
            println!("  -A, --user-agent <AGENT>  User agent string");
            println!("  -s, --save <FILE>         Save response to file");
            println!("  -p, --pretty              Pretty print JSON responses");
            println!("  -v, --verbose             Verbose output with plugin logging\n");
            println!("Examples:");
            println!("  kick get https://api.example.com/users");
            println!("  kick get -H \"Authorization: Bearer token\" https://api.example.com/protected");
            println!("  kick get -p -s response.json https://api.example.com/data");
        },
        Some("post") => {
            println!("KICK POST Command Help\n");
            println!("Make a POST request with JSON data\n");
            println!("Usage: kick post [OPTIONS] --data <DATA> <URL>\n");
            println!("Arguments:");
            println!("  <URL>  URL to request\n");
            println!("Options:");
            println!("  -d, --data <DATA>         JSON data to post");
            println!("  -H, --header <HEADER>     Custom headers (format: \"Key:Value\")");
            println!("  -A, --user-agent <AGENT>  User agent string");
            println!("  -s, --save <FILE>         Save response to file");
            println!("  -p, --pretty              Pretty print JSON responses");
            println!("  -v, --verbose             Verbose output with plugin logging\n");
            println!("Examples:");
            println!("  kick post -d '{{\"name\": \"John\"}}' https://api.example.com/users");
            println!("  kick post -H \"Content-Type: application/json\" -d '{{\"data\": \"value\"}}' https://api.example.com/create");
        },
        Some("put") => {
            println!("KICK PUT Command Help\n");
            println!("Make a PUT request with JSON data\n");
            println!("Usage: kick put [OPTIONS] --data <DATA> <URL>\n");
            println!("Similar to POST but uses PUT method for updates.");
        },
        Some("patch") => {
            println!("KICK PATCH Command Help\n");
            println!("Make a PATCH request with JSON data\n");
            println!("Usage: kick patch [OPTIONS] --data <DATA> <URL>\n");
            println!("Similar to POST but uses PATCH method for partial updates.");
        },
        Some("delete") => {
            println!("KICK DELETE Command Help\n");
            println!("Make a DELETE request\n");
            println!("Usage: kick delete [OPTIONS] <URL>\n");
            println!("Sends DELETE request to remove resources.");
        },
        Some("download") => {
            println!("KICK DOWNLOAD Command Help\n");
            println!("Download file from URL\n");
            println!("Usage: kick download [OPTIONS] --output <FILE> <URL>\n");
            println!("Arguments:");
            println!("  <URL>  URL to download from\n");
            println!("Options:");
            println!("  -o, --output <FILE>       Output filename");
            println!("  -l, --local               Download to ./.downloads/ directory");
            println!("  -H, --header <HEADER>     Custom headers");
            println!("  -A, --user-agent <AGENT>  User agent string");
            println!("  -v, --verbose             Verbose output\n");
            println!("Examples:");
            println!("  kick download -o file.zip https://example.com/file.zip");
            println!("  kick download -l -o local-file.txt https://example.com/data.txt");
        },
        Some("help") => {
            println!("KICK HELP Command Help\n");
            println!("Show help information for commands\n");
            println!("Usage: kick help [COMMAND]\n");
            println!("Show general help or help for specific command.");
        },
        Some("version") => {
            println!("KICK VERSION Command Help\n");
            println!("Show version and license information\n");
            println!("Usage: kick version\n");
            println!("Displays version, logo, and license details.");
        },
        _ => {
            println!("{}", load_logo());
            println!("KICK - Lightweight HTTP client with plugin support\n");
            println!("Usage: kick [OPTIONS] <COMMAND>\n");
            println!("Commands:");
            println!("  get       Make a GET request");
            println!("  post      Make a POST request with JSON data");
            println!("  put       Make a PUT request with JSON data");
            println!("  patch     Make a PATCH request with JSON data");
            println!("  delete    Make a DELETE request");
            println!("  download  Download file from URL");
            println!("  help      Show help information [aliases: -h, --help]");
            println!("  version   Show version and license information [aliases: -v, --version]\n");
            println!("Options:");
            println!("  -h, --help     Print help");
            println!("  -V, --version  Print version\n");
            println!("Use 'kick help <command>' for detailed help on specific commands.");
            println!("\nExample:");
            println!("  kick get https://httpbin.org/get");
            println!("  kick help post");
        }
    }
}

/// Show version with logo and license information
fn show_version() {
    println!("{}", load_logo());
    println!("KICK v{}", env!("CARGO_PKG_VERSION"));
    println!("\nA lightweight HTTP client with plugin support");
    println!("Built with the REBEL philosophy of developer ergonomics\n");
    println!("LICENSE INFORMATION:");
    println!("{}", "-".repeat(50));
    match std::fs::read_to_string("LICENSE") {
        Ok(license) => {
            // Show first few lines of license
            let lines: Vec<&str> = license.lines().take(10).collect();
            for line in &lines {
                println!("{}", line);
            }
            if license.lines().count() > 10 {
                println!("\n... (see LICENSE file for complete terms)");
            }
        },
        Err(_) => {
            println!("Licensed under the Apache License, Version 2.0");
            println!("See LICENSE file for complete terms");
        }
    }
    println!("\nRepository: {}", env!("CARGO_PKG_REPOSITORY"));
}

/// Sanitize save filename to prevent path traversal attacks
fn sanitize_save_filename(filename: &str) -> Result<std::path::PathBuf> {
    PathValidator::safe_current_dir_path(filename)
        .map_err(|e| ApiError::other(format!("Save path validation failed: {}", e)).into())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle version flag
    if cli.version {
        show_version();
        return Ok(());
    }
    
    // Handle case where no command is provided
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            show_help(None);
            return Ok(());
        }
    };
    
    let config = Config::default();
    
    match command {
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