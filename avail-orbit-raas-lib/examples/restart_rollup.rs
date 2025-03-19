use avail_orbit_raas_blueprint_lib::OrbitContext;
use avail_orbit_raas_blueprint_lib::config::OperatorConfig;
use avail_orbit_raas_blueprint_lib::deployment::restart_containers;
use avail_orbit_raas_blueprint_lib::types::{DeploymentStatus, RollupMetadata};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{Level, debug, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize logging with debug level
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Avail Orbit RaaS - Restart Rollup Example");

    // Load environment from .env file
    match dotenv::dotenv() {
        Ok(_) => debug!("Loaded environment from .env file"),
        Err(e) => {
            error!("Failed to load .env file: {}", e);
            info!("Continuing with environment variables...");
        }
    }

    // Load operator configuration
    info!("Loading operator configuration...");
    let operator_config = load_operator_config()?;

    // Create a rollup context with deployment status
    info!("Initializing rollup context...");
    let deployment_status = create_deployment_status().await?;

    // Create orbit context
    let orbit_ctx = OrbitContext::new(operator_config);

    // Update the deployment status
    {
        let mut status = orbit_ctx.status.lock().await;
        *status = deployment_status;
    }

    // Restart the rollup
    info!("Restarting rollup containers...");
    match restart_containers(&orbit_ctx).await {
        Ok(_) => {
            info!("✅ Rollup containers successfully restarted");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to restart rollup containers: {}", e);
            Err(e)
        }
    }
}

/// Load operator configuration from environment variables
fn load_operator_config() -> Result<OperatorConfig, String> {
    debug!("Loading operator configuration from environment variables");

    let operator_config = OperatorConfig {
        deployer_private_key: env::var("DEPLOYER_PRIVATE_KEY")
            .map_err(|_| "DEPLOYER_PRIVATE_KEY not set".to_string())?,
        batch_poster_private_key: env::var("BATCH_POSTER_PRIVATE_KEY")
            .map_err(|_| "BATCH_POSTER_PRIVATE_KEY not set".to_string())?,
        validator_private_key: env::var("VALIDATOR_PRIVATE_KEY")
            .map_err(|_| "VALIDATOR_PRIVATE_KEY not set".to_string())?,
        avail_addr_seed: env::var("AVAIL_ADDR_SEED")
            .map_err(|_| "AVAIL_ADDR_SEED not set".to_string())?,
        fallback_s3_access_key: env::var("FALLBACKS3_ACCESS_KEY").ok(),
        fallback_s3_secret_key: env::var("FALLBACKS3_SECRET_KEY").ok(),
        fallback_s3_region: env::var("FALLBACKS3_REGION").ok(),
        fallback_s3_object_prefix: env::var("FALLBACKS3_OBJECT_PREFIX").ok(),
        fallback_s3_bucket: env::var("FALLBACKS3_BUCKET").ok(),
    };

    debug!("Operator configuration loaded successfully");
    Ok(operator_config)
}

/// Create deployment status from environment or detect running containers
async fn create_deployment_status() -> Result<DeploymentStatus, String> {
    debug!("Creating deployment status");

    // Read container IDs from environment or detect
    let container_ids_str = env::var("ROLLUP_CONTAINER_IDS").unwrap_or_default();
    let container_ids = if container_ids_str.is_empty() {
        // Try to detect containers using docker ps
        debug!("No container IDs provided, attempting to detect...");
        detect_containers().await?
    } else {
        debug!("Using container IDs from environment");
        container_ids_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    };

    debug!(
        "Found {} container(s): {:?}",
        container_ids.len(),
        container_ids
    );

    if container_ids.is_empty() {
        error!("No container IDs found. Is the rollup deployed?");
        return Err("No container IDs found".to_string());
    }

    // Create deployment status
    let chain_id = env::var("ROLLUP_CHAIN_ID")
        .map(|id| id.parse::<u64>().unwrap_or(412346))
        .unwrap_or(412346);

    let metadata = RollupMetadata {
        name: env::var("ROLLUP_NAME").unwrap_or_else(|_| "Avail Orbit Rollup".to_string()),
        chain_id,
        avail_app_id: env::var("AVAIL_APP_ID").unwrap_or_default(),
        parent_chain_rpc: env::var("PARENT_CHAIN_RPC").unwrap_or_default(),
        fallback_s3_enable: env::var("FALLBACKS3_ENABLE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false),
        local_rpc_endpoint: env::var("ROLLUP_LOCAL_RPC")
            .unwrap_or_else(|_| "http://localhost:8449".to_string()),
        explorer_url: env::var("ROLLUP_EXPLORER_URL")
            .unwrap_or_else(|_| "http://localhost:4000".to_string()),
    };

    let status = DeploymentStatus {
        deployed: true,
        logs: vec!["Deployment status loaded from environment".to_string()],
        metadata: Some(metadata),
        container_ids,
    };

    info!("Deployment status created successfully");
    Ok(status)
}

/// Detect docker containers for the rollup
async fn detect_containers() -> Result<Vec<String>, String> {
    debug!("Attempting to detect containers with docker ps");

    let output = match tokio::process::Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.ID}}",
            "--filter",
            "name=orbit-setup-script",
        ])
        .output()
        .await
    {
        Ok(output) => output,
        Err(e) => {
            error!("Failed to execute docker ps: {}", e);
            return Err(format!("Failed to execute docker ps: {}", e));
        }
    };

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("docker ps command failed: {}", error);
        return Err(format!("docker ps command failed: {}", error));
    }

    let container_ids = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    debug!("Detected {} container(s)", container_ids.len());

    Ok(container_ids)
}
