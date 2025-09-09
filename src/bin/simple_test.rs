use kick::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing basic API requests...");
    
    let config = Config::default();
    let client = ApiClient::new(config);
    
    // Test 1: Simple GET
    println!("\n🌐 Testing GET request to ipify...");
    match client.get("https://api.ipify.org/?format=json").await {
        Ok(response) => println!("✅ Got response: {}", response),
        Err(e) => println!("❌ Failed: {}", e),
    }
    
    // Test 2: Dog API
    println!("\n🐕 Testing dog API...");
    match client.get("https://dog.ceo/api/breeds/image/random").await {
        Ok(response) => println!("✅ Got response: {}", response),
        Err(e) => println!("❌ Failed: {}", e),
    }
    
    Ok(())
}