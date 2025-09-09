use crate::error::{ApiError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client: ClientConfig,
    pub storage: StorageConfig,
    pub plugins: PluginConfig,
    pub streaming: StreamingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub user_agent: String,
    pub timeout: u64, // seconds
    pub max_retries: usize,
    pub retry_delay: u64, // milliseconds
    pub default_headers: HashMap<String, String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_path: PathBuf,
    pub temp_path: PathBuf,
    pub max_file_size: u64, // bytes
    pub cleanup_on_exit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled_plugins: Vec<String>,
    pub plugin_paths: Vec<PathBuf>,
    pub plugin_settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub buffer_size: usize,
    pub chunk_size: usize,
    pub max_concurrent_streams: usize,
    pub stream_timeout: u64, // seconds
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("modular-api-client");
        
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("modular-api-client");

        Self {
            client: ClientConfig {
                user_agent: "ModularApiClient/0.1.0".to_string(),
                timeout: 30,
                max_retries: 3,
                retry_delay: 1000,
                default_headers: HashMap::new(),
                base_url: None,
            },
            storage: StorageConfig {
                base_path: data_dir,
                temp_path: cache_dir,
                max_file_size: 100 * 1024 * 1024, // 100MB
                cleanup_on_exit: true,
            },
            plugins: PluginConfig {
                enabled_plugins: Vec::new(),
                plugin_paths: Vec::new(),
                plugin_settings: HashMap::new(),
            },
            streaming: StreamingConfig {
                buffer_size: 8192,
                chunk_size: 4096,
                max_concurrent_streams: 10,
                stream_timeout: 300,
            },
        }
    }
}

impl Config {
    /// Load configuration from XDG config directory
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| ApiError::config(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save configuration to XDG config directory
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| ApiError::config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(&config_path, content)?;
        Ok(())
    }
    
    /// Get the configuration file path using XDG Base Directory Specification
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("modular-api-client")
            .join("config.toml")
    }
    
    /// Get the data directory path
    pub fn data_dir(&self) -> &PathBuf {
        &self.storage.base_path
    }
    
    /// Get the cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.storage.temp_path
    }
    
    /// Ensure all configured directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.storage.base_path)?;
        std::fs::create_dir_all(&self.storage.temp_path)?;
        
        for plugin_path in &self.plugins.plugin_paths {
            if !plugin_path.exists() {
                std::fs::create_dir_all(plugin_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.client.timeout)
    }
    
    /// Get retry delay as Duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.client.retry_delay)
    }
    
    /// Get stream timeout as Duration
    pub fn stream_timeout(&self) -> Duration {
        Duration::from_secs(self.streaming.stream_timeout)
    }
}