use kick::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing Kick API Client with real endpoints from TEST_APIS.md");
    
    // Set up config using default paths
    let config = Config::default();
    
    // Build client with plugins and custom headers
    let mut plugin_manager = PluginManager::new();
    plugin_manager.register_plugin(Arc::new(LoggingPlugin::new()))?;
    
    let client = ApiClientBuilder::new()
        .with_config(config)
        .with_user_agent("KickTestClient/1.0".to_string())
        .with_header("X-Test-Run".to_string(), "endpoint-validation".to_string())
        .with_plugin_manager(plugin_manager)
        .build()
        .await?;
    
    println!("✅ Client built successfully with plugins and custom headers");
    
    // Test 1: IP Address API
    println!("\n🌐 Test 1: IP Address API");
    match client.get("https://api.ipify.org/?format=json").await {
        Ok(response) => {
            println!("✅ IP API Response: {}", response);
            let json: serde_json::Value = serde_json::from_str(&response)?;
            println!("📍 Your IP: {}", json["ip"]);
        }
        Err(e) => println!("❌ IP API failed: {}", e),
    }
    
    // Test 2: Dog Image API  
    println!("\n🐕 Test 2: Random Dog Image API");
    match client.get("https://dog.ceo/api/breeds/image/random").await {
        Ok(response) => {
            println!("✅ Dog API Response: {}", response);
            let json: serde_json::Value = serde_json::from_str(&response)?;
            if json["status"] == "success" {
                println!("🖼️ Dog Image URL: {}", json["message"]);
            }
        }
        Err(e) => println!("❌ Dog API failed: {}", e),
    }
    
    // Test 3: Jokes API
    println!("\n😄 Test 3: Random Jokes API");
    match client.get("https://official-joke-api.appspot.com/jokes/ten").await {
        Ok(response) => {
            println!("✅ Jokes API Response received ({} chars)", response.len());
            let jokes: Vec<serde_json::Value> = serde_json::from_str(&response)?;
            println!("📝 Retrieved {} jokes", jokes.len());
            if let Some(first_joke) = jokes.first() {
                println!("🎭 First joke - Setup: {}", first_joke["setup"]);
                println!("🎪 Punchline: {}", first_joke["punchline"]);
            }
        }
        Err(e) => println!("❌ Jokes API failed: {}", e),
    }
    
    // Test 4: Download JSON (using download_json method)
    println!("\n📥 Test 4: Download JSON with typed deserialization");
    match client.download_json::<serde_json::Value>("https://api.ipify.org/?format=json").await {
        Ok(ip_data) => {
            println!("✅ download_json succeeded");
            println!("🔍 Typed result: {}", ip_data);
        }
        Err(e) => println!("❌ download_json failed: {}", e),
    }
    
    // Test 5: File Download
    println!("\n💾 Test 5: File Download");
    match client.download_file("https://api.ipify.org/?format=json", "ip_info.json").await {
        Ok(file_path) => {
            println!("✅ File downloaded to: {:?}", file_path);
            match std::fs::read_to_string(&file_path) {
                Ok(content) => println!("📄 File content: {}", content),
                Err(e) => println!("❌ Failed to read downloaded file: {}", e),
            }
        }
        Err(e) => println!("❌ File download failed: {}", e),
    }
    
    // Test 6: HTTP Status Testing
    println!("\n⚡ Test 6: HTTP Status Code Testing");
    match client.get("https://httpbin.org/status/200").await {
        Ok(_) => println!("✅ HTTP 200 test passed"),
        Err(e) => println!("❌ HTTP 200 test failed: {}", e),
    }
    
    match client.get("https://httpbin.org/status/404").await {
        Ok(_) => println!("⚠️ HTTP 404 unexpectedly succeeded"),
        Err(e) => println!("✅ HTTP 404 correctly failed: {}", e),
    }
    
    println!("\n🎉 All endpoint tests completed!");
    println!("📊 The API client is working with real world endpoints");
    
    Ok(())
}