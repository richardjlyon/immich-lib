//! CLI tool for managing Immich duplicates with metadata-aware selection.

use clap::Parser;

/// Immich duplicate manager - prioritizes metadata completeness over file size
#[derive(Parser, Debug)]
#[command(name = "immich-dupes")]
#[command(version, about, long_about = None)]
struct Args {
    /// Immich server URL
    #[arg(short, long, env = "IMMICH_URL")]
    url: Option<String>,

    /// API key for authentication
    #[arg(short, long, env = "IMMICH_API_KEY")]
    api_key: Option<String>,
}

fn main() {
    let args = Args::parse();
    println!("immich-dupes v{}", env!("CARGO_PKG_VERSION"));

    if args.url.is_some() {
        println!("Server: {}", args.url.unwrap());
    }
}
