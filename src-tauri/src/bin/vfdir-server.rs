/**
 * VFDir API Server Binary
 *
 * Standalone HTTP REST API server for file manager operations.
 * Can be used by web, mobile, and console clients.
 */

use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(name = "vfdir-server")]
#[command(about = "VFDir REST API Server", long_about = None)]
struct Args {
    /// Server host address
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup logging
    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Parse address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("Invalid host:port combination");

    println!("{}", "=".repeat(60));
    println!("🚀 VFDir REST API Server");
    println!("{}", "=".repeat(60));
    println!();

    // Start server
    vfdir_lib::api_server::start_server(addr).await?;

    Ok(())
}
