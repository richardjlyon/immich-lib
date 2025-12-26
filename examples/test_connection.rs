//! Test connection to a real Immich instance.
//!
//! This example verifies that the ImmichClient can authenticate and
//! fetch duplicate groups from your Immich server.
//!
//! # Setup
//!
//! Create a `.env` file in the project root with:
//! ```
//! IMMICH_URL=https://your-immich-server.com
//! API_KEY=your-api-key-here
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example test_connection
//! ```

use immich_lib::ImmichClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    let base_url = env::var("IMMICH_URL").expect("IMMICH_URL must be set in .env");
    let api_key = env::var("API_KEY").expect("API_KEY must be set in .env");

    println!("Connecting to Immich at: {}", base_url);

    let client = ImmichClient::new(&base_url, &api_key)?;

    println!("Fetching duplicates...");
    let duplicates = client.get_duplicates().await?;

    println!("Found {} duplicate groups", duplicates.len());

    if let Some(first) = duplicates.first() {
        println!(
            "First group ({}) has {} assets",
            first.duplicate_id,
            first.assets.len()
        );

        // Show first asset details
        if let Some(asset) = first.assets.first() {
            println!("  First asset: {} ({})", asset.original_file_name, asset.id);
            if let Some(ref exif) = asset.exif_info {
                if let Some(camera) = exif.make.as_ref().zip(exif.model.as_ref()) {
                    println!("  Camera: {} {}", camera.0, camera.1);
                }
                if exif.latitude.is_some() && exif.longitude.is_some() {
                    println!("  Has GPS coordinates");
                }
            }
        }
    }

    println!("\nConnection test successful!");

    Ok(())
}
