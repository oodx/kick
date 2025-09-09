// Progressive re-enabling of modules using driver patterns
pub mod client;  // Phase 1: Re-enabling with driver HTTP patterns
pub mod config;
pub mod error;
pub mod plugin;  // Phase 1: Re-enabled with driver patterns
// pub mod storage;
// pub mod streaming;
pub mod driver;

pub use client::{ApiClient, ApiClientBuilder};
pub use config::Config;
pub use error::{ApiError, Result};
pub use plugin::{Plugin, PluginManager, LoggingPlugin};
// pub use storage::StorageManager;
// pub use streaming::StreamHandler;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{ApiClient, ApiClientBuilder, Config, ApiError, Result, Plugin, PluginManager, LoggingPlugin};
    // pub use crate::{StorageManager, StreamHandler};
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use std::collections::HashMap;
}