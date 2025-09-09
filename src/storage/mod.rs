use crate::config::Config;
use crate::error::{ApiError, Result};
use bytes::Bytes;
use futures::Stream;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

pub struct StorageManager {
    config: Config,
}

impl StorageManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// Save bytes to a file
    pub async fn save_bytes(&self, data: &[u8], filename: &str) -> Result<PathBuf> {
        let path = self.config.data_dir().join(filename);
        self.ensure_parent_dir(&path).await?;
        
        // Check file size limit
        if data.len() as u64 > self.config.storage.max_file_size {
            return Err(ApiError::storage(format!(
                "File size {} exceeds limit {}",
                data.len(),
                self.config.storage.max_file_size
            )));
        }
        
        fs::write(&path, data).await?;
        Ok(path)
    }
    
    /// Save string content to a file
    pub async fn save_string(&self, content: &str, filename: &str) -> Result<PathBuf> {
        self.save_bytes(content.as_bytes(), filename).await
    }
    
    /// Save JSON data to a file
    pub async fn save_json<T: serde::Serialize>(&self, data: &T, filename: &str) -> Result<PathBuf> {
        let json = serde_json::to_string_pretty(data)?;
        self.save_string(&json, filename).await
    }
    
    /// Save a stream to a file with progress tracking
    pub async fn save_stream<S, E>(
        &self,
        stream: S,
        filename: &str,
        progress_callback: Option<Box<dyn Fn(u64, Option<u64>) + Send + Sync>>,
    ) -> Result<PathBuf>
    where
        S: Stream<Item = std::result::Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let path = self.config.data_dir().join(filename);
        self.ensure_parent_dir(&path).await?;
        
        let mut file = fs::File::create(&path).await?;
        let mut total_bytes = 0u64;
        let mut stream = Box::pin(stream);
        
        while let Some(chunk_result) = futures::StreamExt::next(&mut stream).await {
            let chunk = chunk_result
                .map_err(|e| ApiError::stream(format!("Stream error: {}", e)))?;
            
            // Check file size limit
            if total_bytes + chunk.len() as u64 > self.config.storage.max_file_size {
                return Err(ApiError::storage(format!(
                    "File size would exceed limit {}",
                    self.config.storage.max_file_size
                )));
            }
            
            file.write_all(&chunk).await?;
            total_bytes += chunk.len() as u64;
            
            if let Some(ref callback) = progress_callback {
                callback(total_bytes, None);
            }
        }
        
        file.flush().await?;
        Ok(path)
    }
    
    /// Create a temporary file with unique name
    pub async fn create_temp_file(&self, extension: Option<&str>) -> Result<PathBuf> {
        let filename = if let Some(ext) = extension {
            format!("{}.{}", Uuid::new_v4(), ext)
        } else {
            Uuid::new_v4().to_string()
        };
        
        let path = self.config.cache_dir().join(filename);
        self.ensure_parent_dir(&path).await?;
        
        // Create empty file
        fs::File::create(&path).await?;
        Ok(path)
    }
    
    /// Load file as bytes
    pub async fn load_bytes(&self, filename: &str) -> Result<Vec<u8>> {
        let path = self.config.data_dir().join(filename);
        let data = fs::read(&path).await?;
        Ok(data)
    }
    
    /// Load file as string
    pub async fn load_string(&self, filename: &str) -> Result<String> {
        let path = self.config.data_dir().join(filename);
        let content = fs::read_to_string(&path).await?;
        Ok(content)
    }
    
    /// Load JSON file
    pub async fn load_json<T: serde::de::DeserializeOwned>(&self, filename: &str) -> Result<T> {
        let content = self.load_string(filename).await?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }
    
    /// Delete a file
    pub async fn delete_file(&self, filename: &str) -> Result<()> {
        let path = self.config.data_dir().join(filename);
        if path.exists() {
            fs::remove_file(&path).await?;
        }
        Ok(())
    }
    
    /// List files in storage directory
    pub async fn list_files(&self, pattern: Option<&str>) -> Result<Vec<PathBuf>> {
        let mut entries = fs::read_dir(self.config.data_dir()).await?;
        let mut files = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(pattern) = pattern {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.contains(pattern) {
                            files.push(path);
                        }
                    }
                } else {
                    files.push(path);
                }
            }
        }
        
        Ok(files)
    }
    
    /// Get file metadata
    pub async fn file_metadata(&self, filename: &str) -> Result<FileMetadata> {
        let path = self.config.data_dir().join(filename);
        let metadata = fs::metadata(&path).await?;
        
        Ok(FileMetadata {
            size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            is_file: metadata.is_file(),
        })
    }
    
    /// Clean up temporary files
    pub async fn cleanup_temp_files(&self) -> Result<()> {
        let mut entries = fs::read_dir(self.config.cache_dir()).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Err(e) = fs::remove_file(&path).await {
                    tracing::warn!("Failed to remove temp file {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Ensure parent directory exists
    async fn ensure_parent_dir(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }
        Ok(())
    }
    
    /// Get storage statistics
    pub async fn storage_stats(&self) -> Result<StorageStats> {
        let data_size = self.directory_size(self.config.data_dir()).await?;
        let cache_size = self.directory_size(self.config.cache_dir()).await?;
        
        Ok(StorageStats {
            data_size,
            cache_size,
            total_size: data_size + cache_size,
        })
    }
    
    /// Calculate directory size
    async fn directory_size(&self, dir: &Path) -> Result<u64> {
        let mut total_size = 0u64;
        let mut entries = fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += Box::pin(self.directory_size(&entry.path())).await?;
            }
        }
        
        Ok(total_size)
    }
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub created: Option<std::time::SystemTime>,
    pub modified: Option<std::time::SystemTime>,
    pub is_file: bool,
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub data_size: u64,
    pub cache_size: u64,
    pub total_size: u64,
}

/// Utility for streaming file writer with automatic chunking
pub struct StreamingFileWriter {
    file: tokio::fs::File,
    buffer: Vec<u8>,
    buffer_size: usize,
    total_written: u64,
}

impl StreamingFileWriter {
    pub async fn new(path: &Path, buffer_size: usize) -> Result<Self> {
        let file = fs::File::create(path).await?;
        Ok(Self {
            file,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
            total_written: 0,
        })
    }
    
    pub async fn write_chunk(&mut self, data: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(data);
        
        if self.buffer.len() >= self.buffer_size {
            self.flush_buffer().await?;
        }
        
        Ok(())
    }
    
    pub async fn finish(mut self) -> Result<u64> {
        if !self.buffer.is_empty() {
            self.flush_buffer().await?;
        }
        self.file.flush().await?;
        Ok(self.total_written)
    }
    
    async fn flush_buffer(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.file.write_all(&self.buffer).await?;
            self.total_written += self.buffer.len() as u64;
            self.buffer.clear();
        }
        Ok(())
    }
    
    pub fn total_written(&self) -> u64 {
        self.total_written
    }
}