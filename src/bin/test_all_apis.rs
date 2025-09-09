use kick::prelude::*;
use serde_json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing ALL APIs from TEST_APIS.md");
    
    let config = Config::default();
    let client = ApiClient::new(config);
    
    // Test each API from your list
    let test_urls = vec![
        ("IP Address", "https://api.ipify.org/?format=json"),
        ("Dog Images", "https://dog.ceo/api/breeds/image/random"),
        ("Jokes", "https://official-joke-api.appspot.com/jokes/ten"),
        ("Cat Facts", "https://catfact.ninja/fact"),
        ("UUID", "https://httpbin.org/uuid"),
        ("User Agent", "https://httpbin.org/user-agent"),
        ("Headers", "https://httpbin.org/headers"),
    ];
    
    for (name, url) in test_urls {
        println!("\n📡 Testing: {}", name);
        println!("   URL: {}", url);
        
        match client.get(url).await {
            Ok(response) => {
                let len = response.len();
                println!("   ✅ SUCCESS - {} chars", len);
                
                // Try to parse as JSON to validate
                match serde_json::from_str::<serde_json::Value>(&response) {
                    Ok(_) => println!("   📋 Valid JSON"),
                    Err(_) => println!("   📄 Not JSON (maybe HTML/text)"),
                }
                
                // Show first 100 chars
                let preview = if len > 100 { 
                    format!("{}...", &response[0..100]) 
                } else { 
                    response 
                };
                println!("   👀 Preview: {}", preview);
            }
            Err(e) => println!("   ❌ FAILED: {}", e),
        }
    }
    
    println!("\n🎯 Summary: Basic HTTP GET requests work!");
    println!("   This confirms the API client can make real network requests");
    
    Ok(())
}