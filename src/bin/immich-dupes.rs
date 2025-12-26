//! CLI tool for managing Immich duplicates with metadata-aware selection.

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::Serialize;

use immich_lib::{DuplicateAnalysis, ImmichClient};

/// Immich duplicate manager - prioritizes metadata completeness over file size
#[derive(Parser, Debug)]
#[command(name = "immich-dupes")]
#[command(version, about, long_about = None)]
struct Args {
    /// Immich server URL
    #[arg(short, long, env = "IMMICH_URL")]
    url: String,

    /// API key for authentication
    #[arg(short, long, env = "IMMICH_API_KEY")]
    api_key: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze duplicates and output results to JSON
    Analyze {
        /// Output file path for JSON results
        #[arg(short, long)]
        output: PathBuf,
    },
}

/// Report containing analysis results for all duplicate groups.
#[derive(Debug, Serialize)]
struct AnalysisReport {
    /// Timestamp when the analysis was generated
    generated_at: DateTime<Utc>,

    /// The Immich server URL that was analyzed
    server_url: String,

    /// Total number of duplicate groups found
    total_groups: usize,

    /// Total number of assets across all groups
    total_assets: usize,

    /// Number of groups that need manual review due to conflicts
    needs_review_count: usize,

    /// Analysis results for each duplicate group
    groups: Vec<DuplicateAnalysis>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    match args.command {
        Commands::Analyze { output } => {
            run_analyze(&args.url, &args.api_key, &output).await?;
        }
    }

    Ok(())
}

async fn run_analyze(url: &str, api_key: &str, output: &PathBuf) -> Result<()> {
    println!("Connecting to Immich server at {}...", url);

    // Create client
    let client =
        ImmichClient::new(url, api_key).context("Failed to create Immich client")?;

    // Fetch duplicates
    println!("Fetching duplicate groups...");
    let duplicates = client
        .get_duplicates()
        .await
        .context("Failed to fetch duplicates from Immich")?;

    // Analyze each group
    println!("Analyzing {} duplicate groups...", duplicates.len());
    let groups: Vec<DuplicateAnalysis> = duplicates
        .iter()
        .map(DuplicateAnalysis::from_group)
        .collect();

    // Calculate statistics
    let total_groups = groups.len();
    let total_assets: usize = groups
        .iter()
        .map(|g| 1 + g.losers.len()) // winner + losers
        .sum();
    let needs_review_count = groups.iter().filter(|g| g.needs_review).count();

    // Create report
    let report = AnalysisReport {
        generated_at: Utc::now(),
        server_url: url.to_string(),
        total_groups,
        total_assets,
        needs_review_count,
        groups,
    };

    // Write JSON to file
    let file = File::create(output)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &report)
        .context("Failed to write JSON output")?;

    // Print summary
    println!();
    println!("Analysis complete!");
    println!();
    println!("Duplicate groups: {}", total_groups);
    println!("Total assets: {}", total_assets);
    if needs_review_count > 0 {
        println!(
            "Groups needing review: {} (have metadata conflicts)",
            needs_review_count
        );
    } else {
        println!("Groups needing review: 0");
    }
    println!();
    println!("Output written to: {}", output.display());

    Ok(())
}
