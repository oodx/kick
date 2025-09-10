//! Security and safety helpers module
//! 
//! This module contains security validation and sanitization functions
//! to protect against common vulnerabilities while maintaining MVP development velocity.
//!
//! ## Security Backlog (Future Enhancements)
//! 
//! ### Script Download Isolation (High Priority for v2.0)
//! When downloading executable content (JS, Python, shell scripts, etc.), implement
//! safe isolation mechanisms:
//! 
//! **Potential Approaches:**
//! - Content-Type detection and quarantine flagging
//! - Automatic `.quarantine` extension for executable files  
//! - Sandboxed download directories with restricted permissions (chmod 600)
//! - Integration with OS security features (macOS quarantine, Windows SmartScreen)
//! - Hash verification and malware scanning hooks
//! - User confirmation prompts for executable content
//! 
//! **Implementation Strategy:**
//! ```ignore
//! // Future: ContentTypeValidator for script isolation
//! pub struct ContentTypeValidator;
//! impl ContentTypeValidator {
//!     pub fn is_executable_content(content_type: &str, filename: &str) -> bool;
//!     pub fn quarantine_path(original_path: &Path) -> PathBuf;
//!     pub fn apply_quarantine_permissions(path: &Path) -> Result<()>;
//! }
//! ```
//! 
//! This addresses the security risk of users inadvertently downloading and executing
//! malicious scripts through the API client.

use crate::error::{ApiError, Result};
use std::path::{Path, PathBuf};
use url::Url;

/// URL validation and SSRF protection
pub struct UrlValidator;

impl UrlValidator {
    /// Validate URL format and check for SSRF vulnerabilities
    pub fn validate(url_str: &str) -> Result<Url> {
        let url = Url::parse(url_str)
            .map_err(|_| ApiError::other("Invalid URL format"))?;
        
        // Only allow HTTP/HTTPS for MVP
        match url.scheme() {
            "http" | "https" => {},
            _ => return Err(ApiError::other("Only HTTP/HTTPS URLs allowed")),
        }
        
        // SSRF protection disabled for development/testing
        // Enable with feature flag for production use
        #[cfg(feature = "strict-security")]
        if let Some(host) = url.host() {
            match host {
                url::Host::Ipv4(ip) => {
                    if ip.is_private() || ip.is_loopback() || ip.is_link_local() {
                        return Err(ApiError::other("Private IP addresses not allowed"));
                    }
                },
                url::Host::Ipv6(ip) => {
                    if ip.is_loopback() {
                        return Err(ApiError::other("Loopback addresses not allowed"));
                    }
                },
                url::Host::Domain(domain) => {
                    // Block obvious local domains
                    if domain == "localhost" || domain.ends_with(".local") {
                        return Err(ApiError::other("Local domains not allowed"));
                    }
                }
            }
        }
        
        Ok(url)
    }
}

/// HTTP header validation
pub struct HeaderValidator;

impl HeaderValidator {
    /// Validate header key and value for injection attacks
    pub fn validate_header(key: &str, value: &str) -> Result<()> {
        // Validate key
        if key.trim().is_empty() {
            return Err(ApiError::other("Header key cannot be empty"));
        }
        
        // Check for control characters in key
        if key.chars().any(|c| c.is_control()) {
            return Err(ApiError::other("Header key contains invalid characters"));
        }
        
        // Validate value for CRLF injection
        if value.contains('\r') || value.contains('\n') {
            return Err(ApiError::other("Header contains CRLF sequences"));
        }
        
        // Reject other control characters (except tab)
        if value.chars().any(|c| c.is_control() && c != '\t') {
            return Err(ApiError::other("Header contains invalid control characters"));
        }
        
        // Length limits for MVP (generous but not unlimited)
        if key.len() > 1024 {
            return Err(ApiError::other("Header key too long"));
        }
        
        if value.len() > 8192 {
            return Err(ApiError::other("Header value too long"));
        }
        
        Ok(())
    }
    
    /// Parse and validate "Key:Value" format header
    pub fn parse_and_validate(header: &str) -> Result<(String, String)> {
        if let Some((key, value)) = header.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            
            Self::validate_header(&key, &value)?;
            Ok((key, value))
        } else {
            Err(ApiError::other("Invalid header format, expected 'Key:Value'"))
        }
    }
}

/// File path sanitization
pub struct PathValidator;

impl PathValidator {
    /// Sanitize filename to prevent path traversal attacks
    pub fn sanitize_filename(filename: &str) -> Result<String> {
        let path = Path::new(filename);
        
        // Reject absolute paths
        if path.is_absolute() {
            return Err(ApiError::other("Absolute paths not allowed"));
        }
        
        // Reject paths containing parent directory references
        if filename.contains("..") {
            return Err(ApiError::other("Parent directory references not allowed"));
        }
        
        // Reject empty filenames
        if filename.trim().is_empty() {
            return Err(ApiError::other("Empty filename not allowed"));
        }
        
        Ok(filename.to_string())
    }
    
    /// Create safe path within current directory
    pub fn safe_current_dir_path(filename: &str) -> Result<PathBuf> {
        let path = Path::new(filename);
        
        // Reject absolute paths
        if path.is_absolute() {
            return Err(ApiError::other("Absolute paths not allowed"));
        }
        
        // Reject paths containing parent directory references
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(ApiError::other("Parent directory references not allowed"));
        }
        
        // Reject empty filenames
        if filename.trim().is_empty() {
            return Err(ApiError::other("Empty filename not allowed"));
        }
        
        // Ensure path stays within current directory
        let current_dir = std::env::current_dir()
            .map_err(|e| ApiError::other(format!("Cannot get current directory: {}", e)))?;
        let canonical = current_dir.join(path);
        
        if !canonical.starts_with(&current_dir) {
            return Err(ApiError::other("Path escapes current directory"));
        }
        
        Ok(canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        // Valid URLs
        assert!(UrlValidator::validate("https://api.github.com").is_ok());
        assert!(UrlValidator::validate("http://httpbin.org").is_ok());
        
        // Invalid schemes
        assert!(UrlValidator::validate("ftp://example.com").is_err());
        assert!(UrlValidator::validate("file:///etc/passwd").is_err());
        
        // SSRF attempts
        assert!(UrlValidator::validate("http://127.0.0.1").is_err());
        assert!(UrlValidator::validate("https://localhost").is_err());
        assert!(UrlValidator::validate("http://192.168.1.1").is_err());
    }
    
    #[test]
    fn test_header_validation() {
        // Valid headers
        assert!(HeaderValidator::parse_and_validate("Content-Type: application/json").is_ok());
        assert!(HeaderValidator::parse_and_validate("Authorization: Bearer token123").is_ok());
        
        // CRLF injection attempts
        assert!(HeaderValidator::parse_and_validate("Evil: value\r\nInjected: header").is_err());
        assert!(HeaderValidator::parse_and_validate("Bad: value\nAnother: header").is_err());
        
        // Invalid format
        assert!(HeaderValidator::parse_and_validate("NoColonHeader").is_err());
        assert!(HeaderValidator::parse_and_validate("").is_err());
    }
    
    #[test] 
    fn test_path_validation() {
        // Valid filenames
        assert!(PathValidator::sanitize_filename("safe-file.txt").is_ok());
        assert!(PathValidator::sanitize_filename("data.json").is_ok());
        
        // Path traversal attempts
        assert!(PathValidator::sanitize_filename("../../../etc/passwd").is_err());
        assert!(PathValidator::sanitize_filename("/absolute/path").is_err());
        assert!(PathValidator::sanitize_filename("").is_err());
        assert!(PathValidator::sanitize_filename("   ").is_err());
    }
}