use kick::prelude::*;

#[tokio::test]
async fn test_plugin_loading_from_config() {
    // Create config with enabled plugins
    let mut config = Config::default();
    config.plugins.enabled_plugins = vec!["logging".to_string(), "rate_limiter".to_string()];
    
    // Add plugin settings
    let mut rate_limiter_settings = serde_json::Map::new();
    rate_limiter_settings.insert("requests_per_minute".to_string(), serde_json::Value::Number(serde_json::Number::from(30)));
    config.plugins.plugin_settings.insert("rate_limiter".to_string(), serde_json::Value::Object(rate_limiter_settings));

    // Create client with config-based plugins
    let client = ApiClientBuilder::new()
        .with_config(config)
        .build()
        .await
        .expect("Failed to create client with plugins");

    // Test a simple request to verify plugins are loaded (they shouldn't break anything)
    let response = client.get("https://httpbin.org/get").await
        .expect("GET request should succeed with plugins loaded");
    
    // Just verify we got a valid response
    assert!(response.len() > 0);
    assert!(response.contains("httpbin.org"));
}

#[tokio::test]
async fn test_empty_plugin_config() {
    // Create config with no plugins
    let mut config = Config::default();
    config.plugins.enabled_plugins = vec![];

    // Should work fine with empty plugin list
    let client = ApiClientBuilder::new()
        .with_config(config)
        .build()
        .await
        .expect("Failed to create client with empty plugins");

    // Test a simple request
    let response = client.get("https://httpbin.org/get").await
        .expect("GET request should succeed with no plugins");
    
    assert!(response.len() > 0);
}

#[tokio::test]
async fn test_unknown_plugin_graceful_fallback() {
    // Create config with unknown plugin
    let mut config = Config::default();
    config.plugins.enabled_plugins = vec!["unknown_plugin".to_string()];

    // Should fall back to empty plugin manager (graceful degradation for MVP)
    let client = ApiClientBuilder::new()
        .with_config(config)
        .build()
        .await
        .expect("Client creation should succeed even with unknown plugin");

    // Test should still work
    let response = client.get("https://httpbin.org/get").await
        .expect("GET request should succeed even after plugin loading failure");
    
    assert!(response.len() > 0);
}