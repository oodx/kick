use kick::driver::run_driver_tests;

#[tokio::main]
async fn main() {
    println!("Starting KICK Driver Tests...\n");
    
    match run_driver_tests().await {
        Ok(()) => {
            println!("\n🎉 All driver tests completed successfully!");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("\n❌ Driver tests failed: {}", e);
            std::process::exit(1);
        }
    }
}