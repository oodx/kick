use kick::prelude::*;
use serde_json::json;

#[tokio::test]
async fn test_default_headers_integration() {
    // Create config with default headers
    let mut config = Config::default();
    config
        .client
        .default_headers
        .insert("X-Test-Header".to_string(), "test-value".to_string());
    config
        .client
        .default_headers
        .insert("Authorization".to_string(), "Bearer test-token".to_string());

    // Create client with config
    let client = ApiClientBuilder::new()
        .with_config(config)
        .build()
        .await
        .expect("Failed to create client");

    // Test GET request with httpbin.org which echoes headers back
    let response = client
        .get("https://httpbin.org/get")
        .await
        .expect("GET request failed");

    // Parse response to check headers were included
    let response_json: serde_json::Value =
        serde_json::from_str(&response).expect("Failed to parse JSON response");

    let headers = response_json["headers"]
        .as_object()
        .expect("No headers object in response");

    // Verify our default headers were included
    assert!(
        headers.contains_key("X-Test-Header"),
        "X-Test-Header not found in request"
    );
    assert!(
        headers.contains_key("Authorization"),
        "Authorization header not found in request"
    );

    assert_eq!(headers["X-Test-Header"], "test-value");
    assert_eq!(headers["Authorization"], "Bearer test-token");
}

#[tokio::test]
async fn test_custom_headers_override_defaults() {
    // Create config with default headers
    let mut config = Config::default();
    config
        .client
        .default_headers
        .insert("X-Custom".to_string(), "default-value".to_string());

    // Create client with custom header that overrides default
    let client = ApiClientBuilder::new()
        .with_config(config)
        .with_header("X-Custom".to_string(), "override-value".to_string())
        .unwrap()
        .build()
        .await
        .expect("Failed to create client");

    // Test GET request
    let response = client
        .get("https://httpbin.org/get")
        .await
        .expect("GET request failed");

    let response_json: serde_json::Value =
        serde_json::from_str(&response).expect("Failed to parse JSON response");

    let headers = response_json["headers"]
        .as_object()
        .expect("No headers object in response");

    // Verify custom header overrode default
    assert_eq!(headers["X-Custom"], "override-value");
}

#[tokio::test]
async fn test_post_request_includes_default_headers() {
    // Create config with default headers
    let mut config = Config::default();
    config
        .client
        .default_headers
        .insert("X-API-Key".to_string(), "secret-key".to_string());

    let client = ApiClientBuilder::new()
        .with_config(config)
        .build()
        .await
        .expect("Failed to create client");

    // Test POST request
    let test_data = json!({"test": "data"});
    let response = client
        .post_json("https://httpbin.org/post", &test_data)
        .await
        .expect("POST request failed");

    let response_json: serde_json::Value =
        serde_json::from_str(&response).expect("Failed to parse JSON response");

    let headers = response_json["headers"]
        .as_object()
        .expect("No headers object in response");

    // Verify default header was included in POST
    assert!(
        headers.contains_key("X-Api-Key"),
        "X-API-Key not found in POST request"
    );
    assert_eq!(headers["X-Api-Key"], "secret-key");
}
