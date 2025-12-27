//! CLI tool for managing Immich duplicates with metadata-aware selection.

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use immich_lib::models::ExecutionConfig;
use immich_lib::testing::{all_fixtures, detect_scenarios, format_report, generate_image, ScenarioReport};
use immich_lib::{DuplicateAnalysis, Executor, ImmichClient};

/// Immich duplicate manager - prioritizes metadata completeness over file size
#[derive(Parser, Debug)]
#[command(name = "immich-dupes")]
#[command(version, about, long_about = None)]
struct Args {
    /// Immich server URL (not required for generate-fixtures)
    #[arg(short, long, env = "IMMICH_URL", required = false)]
    url: Option<String>,

    /// API key for authentication (not required for generate-fixtures)
    #[arg(short, long, env = "IMMICH_API_KEY", required = false)]
    api_key: Option<String>,

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

    /// Execute duplicate removal based on analysis JSON
    Execute {
        /// Path to analysis JSON from analyze command
        #[arg(short, long)]
        input: PathBuf,

        /// Directory to download backup files to
        #[arg(short, long)]
        backup_dir: PathBuf,

        /// Permanently delete instead of moving to trash
        #[arg(long, default_value = "false")]
        force: bool,

        /// Max requests per second (default: 10)
        #[arg(long, default_value = "10")]
        rate_limit: u32,

        /// Max concurrent operations (default: 5)
        #[arg(long, default_value = "5")]
        concurrent: usize,

        /// Skip groups that need manual review
        #[arg(long, default_value = "false")]
        skip_review: bool,

        /// Skip confirmation prompt
        #[arg(short, long, default_value = "false")]
        yes: bool,
    },

    /// Find test candidates by scanning duplicate groups and categorizing by scenario
    FindTestCandidates {
        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,

        /// Only show groups matching specific scenario prefix (e.g., "W1", "C", "F")
        #[arg(long)]
        scenario: Option<String>,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate synthetic test fixtures
    GenerateFixtures {
        /// Output directory for fixtures
        #[arg(long, default_value = "tests/fixtures")]
        output_dir: PathBuf,

        /// Only generate specific scenario (e.g., "W1", "C3")
        #[arg(long)]
        scenario: Option<String>,
    },
}

/// Report containing analysis results for all duplicate groups.
#[derive(Debug, Serialize, Deserialize)]
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
            let url = args.url.as_ref().context("IMMICH_URL is required for analyze command")?;
            let api_key = args.api_key.as_ref().context("IMMICH_API_KEY is required for analyze command")?;
            run_analyze(url, api_key, &output).await?;
        }
        Commands::Execute {
            input,
            backup_dir,
            force,
            rate_limit,
            concurrent,
            skip_review,
            yes,
        } => {
            let url = args.url.as_ref().context("IMMICH_URL is required for execute command")?;
            let api_key = args.api_key.as_ref().context("IMMICH_API_KEY is required for execute command")?;
            run_execute(
                url,
                api_key,
                &input,
                &backup_dir,
                force,
                rate_limit,
                concurrent,
                skip_review,
                yes,
            )
            .await?;
        }
        Commands::FindTestCandidates {
            format,
            scenario,
            output,
        } => {
            let url = args.url.as_ref().context("IMMICH_URL is required for find-test-candidates command")?;
            let api_key = args.api_key.as_ref().context("IMMICH_API_KEY is required for find-test-candidates command")?;
            run_find_test_candidates(url, api_key, &format, scenario.as_deref(), output.as_ref())
                .await?;
        }
        Commands::GenerateFixtures { output_dir, scenario } => {
            run_generate_fixtures(&output_dir, scenario.as_deref())?;
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

#[allow(clippy::too_many_arguments)]
async fn run_execute(
    url: &str,
    api_key: &str,
    input: &PathBuf,
    backup_dir: &PathBuf,
    force: bool,
    rate_limit: u32,
    concurrent: usize,
    skip_review: bool,
    yes: bool,
) -> Result<()> {
    // Read and parse analysis JSON
    let file = File::open(input)
        .with_context(|| format!("Failed to open input file: {}", input.display()))?;
    let reader = BufReader::new(file);
    let report: AnalysisReport = serde_json::from_reader(reader)
        .context("Failed to parse analysis JSON")?;

    // Filter groups based on skip_review flag
    let groups: Vec<DuplicateAnalysis> = if skip_review {
        report.groups.into_iter().filter(|g| !g.needs_review).collect()
    } else {
        report.groups
    };

    if groups.is_empty() {
        println!("No groups to process.");
        return Ok(());
    }

    // Calculate assets to process
    let total_assets: usize = groups.iter().map(|g| g.losers.len()).sum();
    let estimated_size: u64 = groups
        .iter()
        .flat_map(|g| g.losers.iter())
        .filter_map(|l| l.file_size)
        .sum();

    // Create backup directory if it doesn't exist
    std::fs::create_dir_all(backup_dir)
        .with_context(|| format!("Failed to create backup directory: {}", backup_dir.display()))?;

    // Print execution summary
    println!();
    println!("Execution Plan");
    println!("==============");
    println!("Groups to process: {}", groups.len());
    println!("Assets to download: {}", total_assets);
    if estimated_size > 0 {
        let size_mb = estimated_size as f64 / 1_048_576.0;
        println!("Estimated disk space: {:.1} MB", size_mb);
    }
    println!("Backup directory: {}", backup_dir.display());
    println!("Force delete: {}", if force { "yes (permanent)" } else { "no (trash)" });
    println!();

    // Confirmation prompt
    if !yes {
        print!("About to download {} assets and delete them from Immich. Continue? [y/N] ", total_assets);
        std::io::stdout().flush()?;

        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        let response = response.trim().to_lowercase();

        if response != "y" && response != "yes" {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!();
    println!("Starting execution...");
    println!();

    // Create client and executor
    let client = ImmichClient::new(url, api_key)
        .context("Failed to create Immich client")?;

    let config = ExecutionConfig {
        requests_per_sec: rate_limit,
        max_concurrent: concurrent,
        backup_dir: backup_dir.clone(),
        force_delete: force,
    };

    let executor = Executor::new(client, config);

    // Execute
    let exec_report = executor.execute_all(&groups).await;

    // Print summary
    println!();
    println!("Execution Complete");
    println!("==================");
    println!("Groups processed: {}", exec_report.total_groups);
    println!("Assets downloaded: {}", exec_report.downloaded);
    println!("Assets deleted: {}", exec_report.deleted);
    println!("Failed operations: {}", exec_report.failed);
    println!("Skipped: {}", exec_report.skipped);

    // Show first few errors if any
    if exec_report.failed > 0 {
        println!();
        println!("First errors:");
        let errors: Vec<_> = exec_report
            .results
            .iter()
            .flat_map(|g| g.download_results.iter())
            .filter_map(|r| {
                if let immich_lib::models::OperationResult::Failed { id, error } = r {
                    Some((id, error))
                } else {
                    None
                }
            })
            .take(5)
            .collect();

        for (id, error) in errors {
            println!("  - {}: {}", id, error);
        }
    }

    // Write execution report to backup directory
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let report_path = backup_dir.join(format!("execution-report-{}.json", timestamp));
    let report_file = File::create(&report_path)
        .with_context(|| format!("Failed to create report file: {}", report_path.display()))?;
    let writer = BufWriter::new(report_file);
    serde_json::to_writer_pretty(writer, &exec_report)
        .context("Failed to write execution report")?;

    println!();
    println!("Execution report: {}", report_path.display());

    Ok(())
}

async fn run_find_test_candidates(
    url: &str,
    api_key: &str,
    format: &str,
    scenario_filter: Option<&str>,
    output: Option<&PathBuf>,
) -> Result<()> {
    println!("Connecting to Immich server at {}...", url);

    // Create client
    let client = ImmichClient::new(url, api_key).context("Failed to create Immich client")?;

    // Fetch duplicates
    println!("Fetching duplicate groups...");
    let duplicates = client
        .get_duplicates()
        .await
        .context("Failed to fetch duplicates from Immich")?;

    println!("Analyzing {} duplicate groups for test scenarios...", duplicates.len());

    // Detect scenarios for each group
    let mut all_matches = Vec::new();
    for group in &duplicates {
        let matches = detect_scenarios(group);
        all_matches.extend(matches);
    }

    // Filter by scenario prefix if specified
    let filtered_matches = if let Some(prefix) = scenario_filter {
        let prefix_upper = prefix.to_uppercase();
        all_matches
            .into_iter()
            .filter(|m| m.scenario.to_string().to_uppercase().starts_with(&prefix_upper))
            .collect()
    } else {
        all_matches
    };

    // Build report
    let report = ScenarioReport::from_matches(filtered_matches, duplicates.len());

    // Format output
    let output_text = match format.to_lowercase().as_str() {
        "json" => serde_json::to_string_pretty(&report)?,
        _ => format_report(&report),
    };

    // Write output
    if let Some(output_path) = output {
        let mut file = File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
        file.write_all(output_text.as_bytes())?;
        println!("\nOutput written to: {}", output_path.display());
    } else {
        println!();
        println!("{}", output_text);
    }

    Ok(())
}

/// Manifest file structure for each scenario fixture
#[derive(Debug, Serialize)]
struct FixtureManifest {
    scenario: String,
    description: String,
    images: Vec<String>,
    expected_winner: String,
}

fn run_generate_fixtures(output_dir: &PathBuf, scenario_filter: Option<&str>) -> Result<()> {
    println!("Loading fixture definitions...");

    let fixtures = all_fixtures();
    let total = fixtures.len();

    // Base images directory (contains real photos for transforms)
    let base_dir = output_dir.join("base");
    if !base_dir.exists() {
        println!("Warning: Base images directory not found: {}", base_dir.display());
        println!("Run the fixture setup first to download base images.");
    }

    // Filter fixtures if scenario specified
    let fixtures: Vec<_> = if let Some(filter) = scenario_filter {
        let filter_upper = filter.to_uppercase();
        fixtures
            .into_iter()
            .filter(|f| f.scenario.to_string().to_uppercase().starts_with(&filter_upper))
            .collect()
    } else {
        fixtures
    };

    if fixtures.is_empty() {
        if let Some(filter) = scenario_filter {
            println!("No fixtures found matching filter: {}", filter);
        } else {
            println!("No fixtures defined.");
        }
        return Ok(());
    }

    println!(
        "Generating {} of {} fixtures...",
        fixtures.len(),
        total
    );

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    let mut generated_count = 0;
    let mut failed_count = 0;

    for fixture in &fixtures {
        let scenario_code = fixture.scenario.code();
        let scenario_dir = output_dir.join(scenario_code);

        // Create scenario subdirectory
        std::fs::create_dir_all(&scenario_dir).with_context(|| {
            format!(
                "Failed to create scenario directory: {}",
                scenario_dir.display()
            )
        })?;

        println!("  {} - {}...", scenario_code.to_uppercase(), fixture.description);

        let mut image_filenames = Vec::new();
        let mut all_success = true;

        for image in &fixture.images {
            match generate_image(image, &base_dir, &scenario_dir) {
                Ok(path) => {
                    image_filenames.push(image.filename.clone());
                    println!("    ✓ {}", path.file_name().unwrap_or_default().to_string_lossy());
                }
                Err(e) => {
                    eprintln!("    ✗ {} - {}", image.filename, e);
                    all_success = false;
                }
            }
        }

        // Write manifest
        let manifest = FixtureManifest {
            scenario: scenario_code.to_uppercase(),
            description: fixture.description.clone(),
            images: image_filenames.clone(),
            expected_winner: fixture
                .images
                .get(fixture.expected_winner_index)
                .map(|i| i.filename.clone())
                .unwrap_or_default(),
        };

        let manifest_path = scenario_dir.join("manifest.json");
        let manifest_file = File::create(&manifest_path)
            .with_context(|| format!("Failed to create manifest: {}", manifest_path.display()))?;
        serde_json::to_writer_pretty(manifest_file, &manifest)
            .context("Failed to write manifest JSON")?;

        if all_success {
            generated_count += 1;
        } else {
            failed_count += 1;
        }
    }

    println!();
    println!("Generation complete!");
    println!("  Successful: {}", generated_count);
    if failed_count > 0 {
        println!("  Failed: {}", failed_count);
    }
    println!("  Output directory: {}", output_dir.display());

    Ok(())
}
