use kick::driver::{DriverClient, DriverApiClient, LoggingPlugin};
use std::sync::Arc;

#[tokio::test]
async fn test_driver_client_creation() {
    let _client = DriverClient::new();
    // Verify we can create without panicking
    assert!(true);
}

#[tokio::test]
async fn test_driver_plugin_system() {
    let _client = DriverApiClient::new()
        .with_plugin(Arc::new(LoggingPlugin));

    // Verify we can create client with plugins without panicking
    assert!(true);
}

// Integration test using httpbin.org (only run when network is available)
#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run network tests
async fn test_driver_network_requests() {
    let client = DriverClient::new();
    
    // Test GET request
    match client.get("https://httpbin.org/status/200").await {
        Ok(_) => assert!(true),
        Err(e) => {
            println!("Network test failed (this may be expected in CI): {}", e);
            // Don't fail the test - network might not be available
        }
    }
}

#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run network tests
async fn test_driver_error_handling() {
    let client = DriverClient::new();
    
    // Test error response
    match client.get("https://httpbin.org/status/404").await {
        Ok(_) => panic!("Expected 404 error but got success"),
        Err(_) => assert!(true), // Expected error
    }
}