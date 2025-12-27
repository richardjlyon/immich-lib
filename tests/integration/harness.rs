//! Test harness for integration tests.
//!
//! Provides setup, teardown, and waiting utilities for Docker-based Immich testing.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use serde::Deserialize;

/// Test harness holding connection info for the Docker Immich instance.
pub struct TestHarness {
    /// API key for authentication
    pub api_key: String,

    /// Base URL of the Immich server
    pub base_url: String,

    /// HTTP client for API requests
    client: Client,

    /// Path to the docker directory
    docker_dir: PathBuf,
}

/// Response from the duplicates API endpoint.
#[derive(Debug, Deserialize)]
pub struct DuplicateGroupResponse {
    #[serde(rename = "duplicateId")]
    pub duplicate_id: String,
    pub assets: Vec<DuplicateAsset>,
}

/// Minimal asset info from duplicates API.
#[derive(Debug, Deserialize)]
pub struct DuplicateAsset {
    pub id: String,
    #[serde(rename = "originalFileName")]
    pub original_file_name: String,
}

impl TestHarness {
    /// Set up the test environment.
    ///
    /// Runs bootstrap and seed scripts, waits for Immich to be ready,
    /// and returns a harness for making API calls.
    pub fn setup() -> Result<Self, Box<dyn std::error::Error>> {
        let docker_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("docker");

        // Run bootstrap script
        let bootstrap_output = Command::new("/bin/sh")
            .arg(docker_dir.join("bootstrap.sh"))
            .current_dir(&docker_dir)
            .output()?;

        if !bootstrap_output.status.success() {
            let stderr = String::from_utf8_lossy(&bootstrap_output.stderr);
            return Err(format!("Bootstrap failed: {}", stderr).into());
        }

        // Run seed script
        let seed_output = Command::new("/bin/sh")
            .arg(docker_dir.join("seed-fixtures.sh"))
            .current_dir(&docker_dir)
            .output()?;

        if !seed_output.status.success() {
            let stderr = String::from_utf8_lossy(&seed_output.stderr);
            return Err(format!("Seed failed: {}", stderr).into());
        }

        // Read API key from file
        let api_key_path = docker_dir.join(".api_key");
        let api_key = fs::read_to_string(&api_key_path)?.trim().to_string();

        if api_key.is_empty() {
            return Err("API key file is empty".into());
        }

        let base_url = "http://localhost:2283".to_string();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            api_key,
            base_url,
            client,
            docker_dir,
        })
    }

    /// Tear down the test environment.
    ///
    /// Runs the teardown script to stop containers and remove volumes.
    pub fn teardown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let teardown_output = Command::new("/bin/sh")
            .arg(self.docker_dir.join("teardown.sh"))
            .current_dir(&self.docker_dir)
            .output()?;

        if !teardown_output.status.success() {
            let stderr = String::from_utf8_lossy(&teardown_output.stderr);
            return Err(format!("Teardown failed: {}", stderr).into());
        }

        Ok(())
    }

    /// Wait for duplicate detection to complete.
    ///
    /// Polls the `/api/duplicates` endpoint every 5 seconds until
    /// duplicate groups are available, or times out after 120 seconds.
    pub fn wait_for_duplicates(&self) -> Result<Vec<DuplicateGroupResponse>, Box<dyn std::error::Error>> {
        let timeout = Duration::from_secs(120);
        let poll_interval = Duration::from_secs(5);
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err("Timeout waiting for duplicate detection (120s)".into());
            }

            match self.fetch_duplicates() {
                Ok(groups) if !groups.is_empty() => {
                    return Ok(groups);
                }
                Ok(_) => {
                    // Empty response, keep waiting
                    thread::sleep(poll_interval);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to fetch duplicates: {}", e);
                    thread::sleep(poll_interval);
                }
            }
        }
    }

    /// Fetch duplicate groups from the API.
    fn fetch_duplicates(&self) -> Result<Vec<DuplicateGroupResponse>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/duplicates", self.base_url);

        let response = self.client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()).into());
        }

        let groups: Vec<DuplicateGroupResponse> = response.json()?;
        Ok(groups)
    }

    /// Get the fixtures directory path.
    pub fn fixtures_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_dir_exists() {
        let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");
        assert!(fixtures_dir.exists(), "Fixtures directory should exist");
    }
}
