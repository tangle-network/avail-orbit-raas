use avail_orbit_raas_blueprint_lib::OrbitContext;
use avail_orbit_raas_blueprint_lib::config::OperatorConfig;
use avail_orbit_raas_blueprint_lib::deployment::update_metadata;
use avail_orbit_raas_blueprint_lib::types::{DeploymentStatus, RollupMetadata};
use std::env;
use tracing::{Level, debug, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize logging with debug level
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Avail Orbit RaaS - Update Rollup Metadata Example");

    // Load environment from .env file
    match dotenv::dotenv() {
        Ok(_) => debug!("Loaded environment from .env file"),
        Err(e) => {
            error!("Failed to load .env file: {}", e);
            info!("Continuing with environment variables...");
        }
    }

    // Check if a rollup is already deployed
    info!("Checking for existing deployment...");
    let mut deployment_status = DeploymentStatus {
        deployed: true, // Assume deployed for update_metadata to work
        logs: vec![],
        metadata: None,
        container_ids: vec![],
    };

    // Create a basic operator config
    let operator_config = load_operator_config()?;

    // Create orbit context
    let orbit_ctx = OrbitContext::new(operator_config);

    // Set deployment status
    {
        let mut status = orbit_ctx.status.lock().await;
        *status = deployment_status;
    }

    // Create new metadata
    info!("Creating new rollup metadata...");
    let new_metadata = create_new_metadata()?;

    // Log the changes
    info!("Updating rollup metadata:");
    info!("  Name: {}", new_metadata.name);
    info!("  Chain ID: {}", new_metadata.chain_id);
    info!("  Avail App ID: {}", new_metadata.avail_app_id);
    info!("  RPC Endpoint: {}", new_metadata.local_rpc_endpoint);
    info!("  Explorer URL: {}", new_metadata.explorer_url);

    // Update the metadata
    info!("Applying metadata update...");
    match update_metadata(&orbit_ctx, &new_metadata).await {
        Ok(_) => {
            info!("✅ Rollup metadata successfully updated!");

            // Verify the update
            let status = orbit_ctx.status.lock().await;
            if let Some(metadata) = &status.metadata {
                info!("Updated metadata:");
                info!("  Name: {}", metadata.name);
                info!("  Chain ID: {}", metadata.chain_id);
                info!("  RPC: {}", metadata.local_rpc_endpoint);
                info!("  Explorer: {}", metadata.explorer_url);
            } else {
                error!("❓ Metadata was not properly updated");
            }

            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to update rollup metadata: {}", e);
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

/// Create new metadata from environment variables and any overrides
fn create_new_metadata() -> Result<RollupMetadata, String> {
    debug!("Creating new rollup metadata");

    // Parse chain ID with error handling
    let chain_id_str = env::var("NEW_ROLLUP_CHAIN_ID").unwrap_or_else(|_| {
        debug!("Using default or existing ROLLUP_CHAIN_ID");
        env::var("ROLLUP_CHAIN_ID").unwrap_or_else(|_| "412346".to_string())
    });

    let chain_id = match chain_id_str.parse::<u64>() {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid chain ID: {}", e);
            return Err(format!("Invalid chain ID: {}", e));
        }
    };

    // Get required avail app ID
    let avail_app_id = env::var("AVAIL_APP_ID").map_err(|_| "AVAIL_APP_ID not set".to_string())?;

    // Get parent chain RPC
    let parent_chain_rpc =
        env::var("PARENT_CHAIN_RPC").map_err(|_| "PARENT_CHAIN_RPC not set".to_string())?;

    // Create metadata
    let metadata = RollupMetadata {
        name: env::var("NEW_ROLLUP_NAME").unwrap_or_else(|_| {
            env::var("ROLLUP_NAME").unwrap_or_else(|_| {
                // Get a timestamp-based name if no override
                let timestamp = chrono::Local::now().format("%Y%m%d%H%M");
                format!("Updated Orbit Rollup {}", timestamp)
            })
        }),
        chain_id,
        avail_app_id,
        parent_chain_rpc,
        fallback_s3_enable: env::var("FALLBACKS3_ENABLE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false),
        local_rpc_endpoint: env::var("NEW_ROLLUP_LOCAL_RPC").unwrap_or_else(|_| {
            env::var("ROLLUP_LOCAL_RPC").unwrap_or_else(|_| "http://localhost:8449".to_string())
        }),
        explorer_url: env::var("NEW_ROLLUP_EXPLORER_URL").unwrap_or_else(|_| {
            env::var("ROLLUP_EXPLORER_URL").unwrap_or_else(|_| "http://localhost:4000".to_string())
        }),
    };

    debug!("New rollup metadata created successfully");
    Ok(metadata)
}
