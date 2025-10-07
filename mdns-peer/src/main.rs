use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get peer identifier from command line argument
    let args: Vec<String> = env::args().collect();
    let identifier = if args.len() > 1 {
        args[1].clone()
    } else {
        eprintln!("Usage: mdns-peer <identifier>");
        eprintln!("Example: mdns-peer alice");
        std::process::exit(1);
    };

    // Set as env var for the shared implementation
    env::set_var("PEER_ID", &identifier);

    mdns_peer::run_desktop().await
}
