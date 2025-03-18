//! Deployment functions for Avail Orbit RaaS
//!
//! This module contains the functions for deploying and managing Arbitrum Orbit rollups
//! with AVAIL data availability.

use crate::config::AvailOrbitConfig;
use crate::types::{DeploymentStatus, RollupMetadata};
use std::path::Path;
use tokio::process::Command as TokioCommand;

const DEPLOYMENT_DIR: &str = "orbit-deployment";
const DOCKER_IMAGE: &str = "availj/avail-nitro-node:v2.2.1-upstream-v3.2.1";
const ORBIT_SDK_REPO: &str = "https://github.com/availproject/arbitrum-orbit-sdk.git";
const ORBIT_SDK_BRANCH: &str = "avail-develop-upstream-v0.20.1";
const SETUP_SCRIPT_REPO: &str = "https://github.com/availproject/orbit-setup-script.git";

/// Deploy an Avail Orbit rollup
///
/// This function handles the full deployment of an Arbitrum Orbit rollup with AVAIL DA.
/// It's designed to be called from the binary, not as a job function.
pub async fn deploy_rollup(config: AvailOrbitConfig) -> Result<DeploymentStatus, String> {
    let mut status = DeploymentStatus::default();
    status.metadata = Some(RollupMetadata {
        name: "orbit-rollup".to_string(),
        chain_id: 412346,
        avail_app_id: config.get_avail_app_id().to_string(),
        parent_chain_rpc: config.get_parent_chain_rpc().to_string(),
        fallback_s3_enable: config.is_fallback_s3_enabled(),
        local_rpc_endpoint: "http://localhost:8449".to_string(),
        explorer_url: "http://localhost:4000".to_string(),
    });

    // Step 1: Pull Docker image
    pull_docker_image(&mut status).await?;

    // Step 2: Clone and set up repositories
    clone_repositories(&mut status).await?;

    // Step 3: Create configuration files
    create_config_files(&config, &mut status).await?;

    // Step 4: Deploy rollup contracts
    deploy_contracts(&mut status).await?;

    // Step 5: Set up and start the chain
    setup_and_start_chain(&mut status).await?;

    // Step 6: Deploy token bridge
    deploy_token_bridge(&config, &mut status).await?;

    status.deployed = true;
    Ok(status)
}

/// Pull the Avail Nitro Node Docker image
async fn pull_docker_image(status: &mut DeploymentStatus) -> Result<(), String> {
    let pull_result = TokioCommand::new("docker")
        .args(["pull", DOCKER_IMAGE])
        .output()
        .await;

    if let Err(e) = pull_result {
        return Err(format!("Failed to pull Docker image: {}", e));
    }

    status
        .logs
        .push("Successfully pulled avail-nitro-node Docker image".to_string());
    Ok(())
}

/// Clone the necessary repositories
async fn clone_repositories(status: &mut DeploymentStatus) -> Result<(), String> {
    // Create deployment directory
    if let Err(e) = std::fs::create_dir_all(DEPLOYMENT_DIR) {
        return Err(format!("Failed to create deployment directory: {}", e));
    }

    // Clone Arbitrum Orbit SDK
    let orbit_sdk_dir = format!("{}/arbitrum-orbit-sdk", DEPLOYMENT_DIR);
    let clone_result = TokioCommand::new("git")
        .args(["clone", ORBIT_SDK_REPO, &orbit_sdk_dir])
        .output()
        .await;

    if let Err(e) = clone_result {
        return Err(format!("Failed to clone arbitrum-orbit-sdk: {}", e));
    }

    // Checkout specific branch
    let checkout_result = TokioCommand::new("git")
        .current_dir(&orbit_sdk_dir)
        .args(["checkout", ORBIT_SDK_BRANCH])
        .output()
        .await;

    if let Err(e) = checkout_result {
        return Err(format!("Failed to checkout branch: {}", e));
    }

    // Clone setup script repository
    let setup_script_dir = format!("{}/orbit-setup-script", DEPLOYMENT_DIR);
    let clone_setup_result = TokioCommand::new("git")
        .args(["clone", SETUP_SCRIPT_REPO, &setup_script_dir])
        .output()
        .await;

    if let Err(e) = clone_setup_result {
        return Err(format!("Failed to clone orbit-setup-script: {}", e));
    }

    status
        .logs
        .push("Successfully cloned required repositories".to_string());
    Ok(())
}

/// Create configuration files for deployment
async fn create_config_files(
    config: &AvailOrbitConfig,
    status: &mut DeploymentStatus,
) -> Result<(), String> {
    let rollup_dir = format!(
        "{}/arbitrum-orbit-sdk/examples/create-avail-rollup-eth",
        DEPLOYMENT_DIR
    );

    // Create directories if they don't exist
    if let Err(e) = std::fs::create_dir_all(&rollup_dir) {
        return Err(format!("Failed to create directories: {}", e));
    }

    // Generate and write .env file
    let env_content = config.generate_env_content();
    if let Err(e) = std::fs::write(format!("{}/{}", &rollup_dir, ".env"), env_content) {
        return Err(format!("Failed to write .env file: {}", e));
    }

    status
        .logs
        .push("Successfully created configuration files".to_string());
    Ok(())
}

/// Deploy rollup contracts
async fn deploy_contracts(status: &mut DeploymentStatus) -> Result<(), String> {
    let rollup_dir = format!(
        "{}/arbitrum-orbit-sdk/examples/create-avail-rollup-eth",
        DEPLOYMENT_DIR
    );

    let deploy_result = TokioCommand::new("npm")
        .current_dir(&rollup_dir)
        .arg("run")
        .arg("deploy-avail-orbit-rollup")
        .output()
        .await;

    if let Err(e) = deploy_result {
        return Err(format!("Failed to deploy rollup contracts: {}", e));
    }

    // Verify generated files exist
    let node_config_path = Path::new(&rollup_dir).join("nodeConfig.json");
    let orbit_config_path = Path::new(&rollup_dir).join("orbitSetupScriptConfig.json");

    if !node_config_path.exists() || !orbit_config_path.exists() {
        return Err("Deployment did not generate required configuration files".to_string());
    }

    status
        .logs
        .push("Successfully deployed rollup contracts".to_string());
    Ok(())
}

/// Set up and start the rollup chain
async fn setup_and_start_chain(status: &mut DeploymentStatus) -> Result<(), String> {
    let rollup_dir = format!(
        "{}/arbitrum-orbit-sdk/examples/create-avail-rollup-eth",
        DEPLOYMENT_DIR
    );
    let setup_dir = format!("{}/orbit-setup-script", DEPLOYMENT_DIR);
    let config_dir = format!("{}/config", setup_dir);

    // Create config directory
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        return Err(format!("Failed to create config directory: {}", e));
    }

    // Copy configuration files
    if let Err(e) = std::fs::copy(
        format!("{}/nodeConfig.json", rollup_dir),
        format!("{}/nodeConfig.json", config_dir),
    ) {
        return Err(format!("Failed to copy nodeConfig.json: {}", e));
    }

    if let Err(e) = std::fs::copy(
        format!("{}/orbitSetupScriptConfig.json", rollup_dir),
        format!("{}/orbitSetupScriptConfig.json", config_dir),
    ) {
        return Err(format!("Failed to copy orbitSetupScriptConfig.json: {}", e));
    }

    // Start the chain
    let start_result = TokioCommand::new("docker-compose")
        .current_dir(&setup_dir)
        .arg("up")
        .arg("-d")
        .output()
        .await;

    if let Err(e) = start_result {
        return Err(format!("Failed to start the rollup chain: {}", e));
    }

    // Get container IDs
    let containers_result = TokioCommand::new("docker-compose")
        .current_dir(&setup_dir)
        .args(["ps", "-q"])
        .output()
        .await;

    if let Ok(output) = containers_result {
        let container_list = String::from_utf8_lossy(&output.stdout);
        status.container_ids = container_list.lines().map(|s| s.to_string()).collect();
    }

    status
        .logs
        .push("Successfully started the chain".to_string());
    Ok(())
}

/// Deploy token bridge
async fn deploy_token_bridge(
    config: &AvailOrbitConfig,
    status: &mut DeploymentStatus,
) -> Result<(), String> {
    let setup_dir = format!("{}/orbit-setup-script", DEPLOYMENT_DIR);

    let bridge_result = TokioCommand::new("yarn")
        .current_dir(&setup_dir)
        .env("PRIVATE_KEY", config.get_deployer_private_key())
        .env("L2_RPC_URL", "https://sepolia-rollup.arbitrum.io/rpc")
        .env("L3_RPC_URL", "http://localhost:8449")
        .arg("run")
        .arg("setup")
        .output()
        .await;

    if let Err(e) = bridge_result {
        return Err(format!("Failed to deploy token bridge: {}", e));
    }

    status
        .logs
        .push("Successfully deployed token bridge".to_string());
    Ok(())
}

/// Update the rollup metadata
pub async fn update_metadata(
    context: &crate::OrbitContext,
    metadata: &RollupMetadata,
) -> Result<(), String> {
    let mut status = context.status.lock().await;

    if !status.deployed {
        return Err("Cannot update metadata - rollup not deployed".to_string());
    }

    // Update the metadata
    status.metadata = Some(metadata.clone());

    Ok(())
}

/// Restart the rollup containers
pub async fn restart_containers(context: &crate::OrbitContext) -> Result<(), String> {
    let status = context.status.lock().await;

    if !status.deployed {
        return Err("Cannot restart - rollup not deployed".to_string());
    }

    // Stop containers
    for container_id in &status.container_ids {
        let stop_result = std::process::Command::new("docker")
            .args(["stop", container_id])
            .output();

        if let Err(e) = stop_result {
            return Err(format!("Failed to stop container {}: {}", container_id, e));
        }
    }

    // Start containers again
    let setup_dir = format!("{}/orbit-setup-script", DEPLOYMENT_DIR);
    let start_result = std::process::Command::new("docker-compose")
        .current_dir(setup_dir)
        .arg("up")
        .arg("-d")
        .output();

    if let Err(e) = start_result {
        return Err(format!("Failed to restart rollup: {}", e));
    }

    Ok(())
}

/// Update the token bridge
pub async fn update_rollup_bridge(context: &crate::OrbitContext) -> Result<(), String> {
    let status = context.status.lock().await;

    if !status.deployed {
        return Err("Cannot update bridge - rollup not deployed".to_string());
    }

    let operator_config = context.operator_config.lock().await;
    let setup_dir = format!("{}/orbit-setup-script", DEPLOYMENT_DIR);

    let result = TokioCommand::new("yarn")
        .current_dir(setup_dir)
        .env("PRIVATE_KEY", &operator_config.deployer_private_key)
        .env("L2_RPC_URL", "https://sepolia-rollup.arbitrum.io/rpc")
        .env("L3_RPC_URL", "http://localhost:8449")
        .arg("run")
        .arg("setup")
        .output()
        .await;

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(format!(
                    "Failed to update token bridge: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }
        Err(e) => Err(format!("Failed to execute bridge update command: {}", e)),
    }
}
