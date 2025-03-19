use avail_orbit_raas_blueprint_lib::config::{AvailOrbitConfig, OperatorConfig};
use avail_orbit_raas_blueprint_lib::deployment;
use avail_orbit_raas_blueprint_lib::types::RollupMetadata;
use avail_orbit_raas_blueprint_lib::util;
use std::env;
use std::process::exit;
use tracing::{Level, debug, error, info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize logging with debug level
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Avail Orbit RaaS - Direct Deployment Example");

    // Load environment from .env file
    match dotenv::dotenv() {
        Ok(_) => debug!("Loaded environment from .env file"),
        Err(e) => {
            error!("Failed to load .env file: {}", e);
            info!("Continuing with environment variables...");
        }
    }

    // Check prerequisites with detailed logging
    info!("Checking prerequisites...");
    check_prerequisites().await;

    // Load configuration
    info!("Loading configuration...");
    let operator_config = load_operator_config()?;
    debug!(
        "Operator config loaded with deployer key: {}",
        mask_key(&operator_config.deployer_private_key)
    );

    let rollup_metadata = load_rollup_metadata()?;
    debug!(
        "Rollup metadata loaded for chain ID: {}",
        rollup_metadata.chain_id
    );

    // Create deployment config
    let config = AvailOrbitConfig::new(operator_config, rollup_metadata.clone());

    // Execute deployment with detailed logging
    info!("Starting rollup deployment...");
    match deployment::deploy_rollup(config).await {
        Ok(status) => {
            info!("✅ Deployment successful!");
            info!("Deployment status: is_deployed={}", status.deployed);
            info!("Container IDs: {:?}", status.container_ids);

            if let Some(metadata) = status.metadata {
                info!(
                    "Deployed rollup: {} (chain ID: {})",
                    metadata.name, metadata.chain_id
                );
                info!("RPC endpoint: {}", metadata.local_rpc_endpoint);
                info!("Explorer URL: {}", metadata.explorer_url);
            }

            info!("Deployment logs:");
            for (i, log) in status.logs.iter().enumerate() {
                info!("[{}] {}", i + 1, log);
            }

            Ok(())
        }
        Err(e) => {
            error!("❌ Deployment failed: {}", e);
            Err(e)
        }
    }
}

/// Load operator configuration from environment variables
fn load_operator_config() -> Result<OperatorConfig, String> {
    debug!("Loading operator configuration from environment variables");

    let deployer_key = match env::var("DEPLOYER_PRIVATE_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("DEPLOYER_PRIVATE_KEY environment variable not set");
            return Err("DEPLOYER_PRIVATE_KEY not set".to_string());
        }
    };

    let batch_poster_key = match env::var("BATCH_POSTER_PRIVATE_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("BATCH_POSTER_PRIVATE_KEY environment variable not set");
            return Err("BATCH_POSTER_PRIVATE_KEY not set".to_string());
        }
    };

    let validator_key = match env::var("VALIDATOR_PRIVATE_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("VALIDATOR_PRIVATE_KEY environment variable not set");
            return Err("VALIDATOR_PRIVATE_KEY not set".to_string());
        }
    };

    let avail_seed = match env::var("AVAIL_ADDR_SEED") {
        Ok(seed) => seed,
        Err(_) => {
            error!("AVAIL_ADDR_SEED environment variable not set");
            return Err("AVAIL_ADDR_SEED not set".to_string());
        }
    };

    let operator_config = OperatorConfig {
        deployer_private_key: deployer_key,
        batch_poster_private_key: batch_poster_key,
        validator_private_key: validator_key,
        avail_addr_seed: avail_seed,
        fallback_s3_access_key: env::var("FALLBACKS3_ACCESS_KEY").ok(),
        fallback_s3_secret_key: env::var("FALLBACKS3_SECRET_KEY").ok(),
        fallback_s3_region: env::var("FALLBACKS3_REGION").ok(),
        fallback_s3_object_prefix: env::var("FALLBACKS3_OBJECT_PREFIX").ok(),
        fallback_s3_bucket: env::var("FALLBACKS3_BUCKET").ok(),
    };

    debug!("Operator configuration loaded successfully");
    Ok(operator_config)
}

/// Load rollup metadata from environment variables
fn load_rollup_metadata() -> Result<RollupMetadata, String> {
    debug!("Loading rollup metadata from environment variables");

    // Parse chain ID with fallback value and detailed error handling
    let chain_id = match env::var("ROLLUP_CHAIN_ID") {
        Ok(id_str) => match id_str.parse::<u64>() {
            Ok(id) => {
                debug!("Parsed chain ID: {}", id);
                id
            }
            Err(e) => {
                warn!("Invalid ROLLUP_CHAIN_ID, using default: {}", e);
                412346
            }
        },
        Err(_) => {
            debug!("ROLLUP_CHAIN_ID not set, using default: 412346");
            412346
        }
    };

    // Get required AVAIL_APP_ID with error handling
    let avail_app_id = match env::var("AVAIL_APP_ID") {
        Ok(id) => id,
        Err(_) => {
            error!("AVAIL_APP_ID environment variable not set");
            return Err("AVAIL_APP_ID not set".to_string());
        }
    };

    // Get required parent chain RPC with error handling
    let parent_chain_rpc = match env::var("PARENT_CHAIN_RPC") {
        Ok(rpc) => rpc,
        Err(_) => {
            error!("PARENT_CHAIN_RPC environment variable not set");
            return Err("PARENT_CHAIN_RPC not set".to_string());
        }
    };

    // Parse S3 fallback flag with detailed logging
    let fallback_s3_enable = match env::var("FALLBACKS3_ENABLE") {
        Ok(enable) => {
            let enabled = enable.to_lowercase() == "true";
            debug!("S3 fallback enabled: {}", enabled);
            if enabled {
                debug!("S3 fallback credentials will be required");
            }
            enabled
        }
        Err(_) => {
            debug!("FALLBACKS3_ENABLE not set, defaulting to false");
            false
        }
    };

    let rollup_metadata = RollupMetadata {
        name: env::var("ROLLUP_NAME").unwrap_or_else(|_| {
            debug!("ROLLUP_NAME not set, using default");
            "Avail Orbit Rollup".to_string()
        }),
        chain_id,
        avail_app_id,
        parent_chain_rpc,
        fallback_s3_enable,
        local_rpc_endpoint: env::var("ROLLUP_LOCAL_RPC").unwrap_or_else(|_| {
            debug!("ROLLUP_LOCAL_RPC not set, using default");
            "http://localhost:8449".to_string()
        }),
        explorer_url: env::var("ROLLUP_EXPLORER_URL").unwrap_or_else(|_| {
            debug!("ROLLUP_EXPLORER_URL not set, using default");
            "http://localhost:4000".to_string()
        }),
    };

    debug!("Rollup metadata loaded successfully");
    Ok(rollup_metadata)
}

/// Check prerequisites with detailed logging and status
async fn check_prerequisites() {
    debug!("Checking Docker availability...");
    let docker_available = match util::check_docker_available().await {
        Ok(true) => {
            info!("✅ Docker is available");
            true
        }
        Ok(false) => {
            error!("❌ Docker is installed but not responding correctly");
            false
        }
        Err(e) => {
            error!("❌ Docker check failed: {}", e);
            false
        }
    };

    debug!("Checking Docker Compose availability...");
    let compose_available = match util::check_docker_compose_available().await {
        Ok(true) => {
            info!("✅ Docker Compose is available");
            true
        }
        Ok(false) => {
            error!("❌ Docker Compose is installed but not responding correctly");
            false
        }
        Err(e) => {
            error!("❌ Docker Compose check failed: {}", e);
            false
        }
    };

    debug!("Checking NPM availability...");
    let npm_available = match util::check_npm_available().await {
        Ok(true) => {
            info!("✅ NPM is available");
            true
        }
        Ok(false) => {
            error!("❌ NPM is installed but not responding correctly");
            false
        }
        Err(e) => {
            error!("❌ NPM check failed: {}", e);
            false
        }
    };

    debug!("Checking Yarn availability...");
    let yarn_available = match util::check_yarn_available().await {
        Ok(true) => {
            info!("✅ Yarn is available");
            true
        }
        Ok(false) => {
            error!("❌ Yarn is installed but not responding correctly");
            false
        }
        Err(e) => {
            error!("❌ Yarn check failed: {}", e);
            false
        }
    };

    // Exit if prerequisites aren't met
    if !docker_available || !compose_available || !npm_available || !yarn_available {
        error!("❌ Prerequisites check failed. Please install missing dependencies.");
        exit(1);
    }

    info!("✅ All prerequisites are met");
}

/// Mask private key for secure logging
fn mask_key(key: &str) -> String {
    if key.len() <= 10 {
        return "[MASKED]".to_string();
    }

    let start = &key[0..6];
    let end = &key[key.len() - 4..];
    format!("{}...{}", start, end)
}
