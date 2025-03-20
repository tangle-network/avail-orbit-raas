//! Utility functions for Avail Orbit RaaS

use std::path::Path;
use tokio::process::Command;

/// Check if Docker is installed and available
pub async fn check_docker_available() -> Result<bool, String> {
    let result = Command::new("docker")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute docker command: {}", e))?;

    Ok(result.status.success())
}

/// Check if Docker Compose is installed and available
pub async fn check_docker_compose_available() -> Result<bool, String> {
    let docker_compose = Command::new("docker")
        .arg("compose")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute docker compose command: {}", e));

    let docker_compose_old = Command::new("docker-compose")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute docker-compose command: {}", e));

    let result = match (docker_compose, docker_compose_old) {
        (Ok(docker_compose), Ok(docker_compose_old)) => {
            docker_compose.status.success() || docker_compose_old.status.success()
        }
        (Err(_), Ok(docker_compose_old)) => docker_compose_old.status.success(),
        (Ok(docker_compose), Err(_)) => docker_compose.status.success(),
        (Err(e1), Err(e2)) => {
            return Err(format!(
                "Failed to execute docker compose command: {} and docker-compose command: {}",
                e1, e2
            ));
        }
    };

    Ok(result)
}

/// Check if a directory exists
pub fn dir_exists(path: &str) -> bool {
    Path::new(path).is_dir()
}

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    Path::new(path).is_file()
}

/// Check if npm is installed and available
pub async fn check_npm_available() -> Result<bool, String> {
    let result = Command::new("npm")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute npm command: {}", e))?;

    Ok(result.status.success())
}

/// Check if Yarn is installed and available
pub async fn check_yarn_available() -> Result<bool, String> {
    let result = Command::new("yarn")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute yarn command: {}", e))?;

    Ok(result.status.success())
}
