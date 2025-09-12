pub mod client;
pub mod config;
pub mod error;
pub mod plugin;
pub mod storage;
pub mod streaming;

pub use client::ApiClient;
pub use config::Config;
pub use error::{ApiError, Result};
pub use plugin::{Plugin, PluginManager};
pub use storage::StorageManager;
pub use streaming::StreamHandler;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{ApiClient, Config, ApiError, Result, Plugin, PluginManager, StorageManager, StreamHandler};
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use std::collections::HashMap;
}