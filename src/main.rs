use clierp::cli::app::CLIApp;
use std::process;

#[tokio::main]
async fn main() {
    // Initialize and run the CLI application
    match CLIApp::new() {
        Ok(mut app) => {
            if let Err(e) = app.run().await {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize CLIERP: {}", e);
            process::exit(1);
        }
    }
}
