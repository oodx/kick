use thiserror::Error;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),
    
    #[error("HTTP status error: {status}")]
    HttpStatus { status: hyper::StatusCode },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Stream error: {0}")]
    Stream(String),
    
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Invalid response format")]
    InvalidResponse,
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Unknown error: {0}")]
    Other(String),
}

impl ApiError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    
    pub fn plugin(msg: impl Into<String>) -> Self {
        Self::Plugin(msg.into())
    }
    
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }
    
    pub fn stream(msg: impl Into<String>) -> Self {
        Self::Stream(msg.into())
    }
    
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}