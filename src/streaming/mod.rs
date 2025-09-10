use crate::config::Config;
use crate::error::{ApiError, Result};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use pin_project::pin_project;
use hyper::Response;
use http_body_util::{combinators::BoxBody, BodyExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::{interval, timeout, Duration, Interval};

pub struct StreamHandler {
    config: Config,
}

impl StreamHandler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// Convert HTTP response body to byte stream with timeout
    pub fn response_to_stream(
        &self,
        response: Response<BoxBody<Bytes, hyper::Error>>,
    ) -> impl Stream<Item = Result<Bytes>> + '_ {
        let timeout_duration = self.config.stream_timeout();
        let (_parts, body) = response.into_parts();
        
        // Convert body to stream using BodyExt::into_data_stream
        let stream = body.into_data_stream();
        
        // Map errors and apply timeout to each chunk
        stream
            .map(move |result| {
                let chunk_result = result.map_err(|e| ApiError::Http(e));
                async move {
                    timeout(timeout_duration, async move { chunk_result })
                        .await
                        .map_err(|_| ApiError::Timeout)
                        .and_then(|r| r)
                }
            })
            .buffered(1)  // Process one future at a time
    }
    
    /// Create a buffered stream that collects chunks until buffer is full
    pub fn create_buffered_stream<S, E>(
        &self,
        stream: S,
    ) -> BufferedStream<S>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        BufferedStream::new(stream, self.config.streaming.buffer_size)
    }
    
    /// Create a chunked stream that yields fixed-size chunks
    pub fn create_chunked_stream<S, E>(
        &self,
        stream: S,
    ) -> ChunkedStream<S>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        ChunkedStream::new(stream, self.config.streaming.chunk_size)
    }
    
    /// Create a rate-limited stream
    pub fn create_rate_limited_stream<S, E>(
        &self,
        stream: S,
        bytes_per_second: u64,
    ) -> RateLimitedStream<S>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        RateLimitedStream::new(stream, bytes_per_second)
    }
    
    /// Collect entire stream into bytes with size limit
    pub async fn collect_stream<S, E>(
        &self,
        mut stream: S,
        max_size: Option<u64>,
    ) -> Result<Bytes>
    where
        S: Stream<Item = std::result::Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut collected = Vec::new();
        let mut total_size = 0u64;
        let limit = max_size.unwrap_or(self.config.storage.max_file_size);
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| ApiError::stream(format!("Stream error: {}", e)))?;
            
            total_size += chunk.len() as u64;
            if total_size > limit {
                return Err(ApiError::storage(format!(
                    "Stream size {} exceeds limit {}",
                    total_size, limit
                )));
            }
            
            collected.extend_from_slice(&chunk);
        }
        
        Ok(Bytes::from(collected))
    }
    
    /// Create a stream progress tracker
    pub fn track_progress<S, E, F>(
        &self,
        stream: S,
        callback: F,
    ) -> ProgressStream<S, F>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: std::error::Error + Send + Sync + 'static,
        F: Fn(u64, Option<u64>) + Send + Sync,
    {
        ProgressStream::new(stream, callback)
    }
}

/// Buffered stream that accumulates data until buffer size is reached
pub struct BufferedStream<S> {
    inner: S,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<S> BufferedStream<S> {
    fn new(stream: S, buffer_size: usize) -> Self {
        Self {
            inner: stream,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }
}

impl<S, E> Stream for BufferedStream<S>
where
    S: Stream<Item = std::result::Result<Bytes, E>> + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<Bytes>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.buffer.extend_from_slice(&chunk);
                    
                    if self.buffer.len() >= self.buffer_size {
                        let data = std::mem::take(&mut self.buffer);
                        return Poll::Ready(Some(Ok(Bytes::from(data))));
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(ApiError::stream(format!("Stream error: {}", e)))));
                }
                Poll::Ready(None) => {
                    if !self.buffer.is_empty() {
                        let data = std::mem::take(&mut self.buffer);
                        return Poll::Ready(Some(Ok(Bytes::from(data))));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Chunked stream that yields fixed-size chunks
pub struct ChunkedStream<S> {
    inner: S,
    buffer: Vec<u8>,
    chunk_size: usize,
}

impl<S> ChunkedStream<S> {
    fn new(stream: S, chunk_size: usize) -> Self {
        Self {
            inner: stream,
            buffer: Vec::new(),
            chunk_size,
        }
    }
}

impl<S, E> Stream for ChunkedStream<S>
where
    S: Stream<Item = std::result::Result<Bytes, E>> + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<Bytes>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // If we have enough data for a chunk, return it
            if self.buffer.len() >= self.chunk_size {
                let chunk_size = self.chunk_size;
                let chunk_data = self.buffer.drain(..chunk_size).collect::<Vec<u8>>();
                return Poll::Ready(Some(Ok(Bytes::from(chunk_data))));
            }
            
            // Otherwise, try to get more data
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(data))) => {
                    self.buffer.extend_from_slice(&data);
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(ApiError::stream(format!("Stream error: {}", e)))));
                }
                Poll::Ready(None) => {
                    if !self.buffer.is_empty() {
                        let remaining = std::mem::take(&mut self.buffer);
                        return Poll::Ready(Some(Ok(Bytes::from(remaining))));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Rate-limited stream
pub struct RateLimitedStream<S> {
    inner: S,
    interval: Interval,
    bytes_per_tick: u64,
    pending_bytes: u64,
}

impl<S> RateLimitedStream<S> {
    fn new(stream: S, bytes_per_second: u64) -> Self {
        // Update every 100ms
        let ticks_per_second = 10;
        let interval = interval(Duration::from_millis(100));
        let bytes_per_tick = bytes_per_second / ticks_per_second;
        
        Self {
            inner: stream,
            interval,
            bytes_per_tick,
            pending_bytes: 0,
        }
    }
}

impl<S, E> Stream for RateLimitedStream<S>
where
    S: Stream<Item = std::result::Result<Bytes, E>> + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<Bytes>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check if we can send more data
        if self.pending_bytes > self.bytes_per_tick {
            // Wait for next tick
            match self.interval.poll_tick(cx) {
                Poll::Ready(_) => {
                    self.pending_bytes = 0;
                }
                Poll::Pending => return Poll::Pending,
            }
        }
        
        // Try to get next item
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(data))) => {
                self.pending_bytes += data.len() as u64;
                Poll::Ready(Some(Ok(data)))
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(ApiError::stream(format!("Stream error: {}", e)))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Progress tracking stream
#[pin_project]
pub struct ProgressStream<S, F> {
    #[pin]
    inner: S,
    callback: F,
    total_bytes: u64,
}

impl<S, F> ProgressStream<S, F> {
    fn new(stream: S, callback: F) -> Self {
        Self {
            inner: stream,
            callback,
            total_bytes: 0,
        }
    }
}

impl<S, E, F> Stream for ProgressStream<S, F>
where
    S: Stream<Item = std::result::Result<Bytes, E>> + Unpin,
    E: std::error::Error + Send + Sync + 'static,
    F: Fn(u64, Option<u64>) + Send + Sync,
{
    type Item = Result<Bytes>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.inner.poll_next(cx) {
            Poll::Ready(Some(Ok(data))) => {
                *this.total_bytes += data.len() as u64;
                (this.callback)(*this.total_bytes, None);
                Poll::Ready(Some(Ok(data)))
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(ApiError::stream(format!("Stream error: {}", e)))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}