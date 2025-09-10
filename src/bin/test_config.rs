use kick::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CONFIGURATION SYSTEM TEST ===");
    
    // Load config from XDG location
    println!("\n1. Loading configuration from XDG location...");
    let config = Config::load()?;
    
    println!("✓ Config loaded successfully");
    println!("User Agent: {}", config.client.user_agent);
    println!("Timeout: {} seconds", config.client.timeout);
    println!("Max Retries: {}", config.client.max_retries);
    println!("Base Path: {:?}", config.storage.base_path);
    
    // Test default headers
    println!("\nDefault Headers:");
    for (key, value) in &config.client.default_headers {
        println!("  {}: {}", key, value);
    }
    
    // Create client with loaded config
    println!("\n2. Creating client with loaded configuration...");
    let client = ApiClient::new(config);
    
    // Test request with configured settings
    println!("\n3. Testing request with custom configuration...");
    match client.get("https://httpbin.org/headers").await {
        Ok(response) => {
            println!("✓ Request successful");
            
            // Parse response to check if our custom headers were sent
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                if let Some(headers) = json.get("headers") {
                    println!("Response headers received:");
                    println!("{}", serde_json::to_string_pretty(headers).unwrap_or_default());
                }
            }
        }
        Err(e) => {
            println!("✗ Request failed: {}", e);
            return Err(e);
        }
    }
    
    println!("\n=== CONFIGURATION TEST COMPLETE ===");
    Ok(())
}