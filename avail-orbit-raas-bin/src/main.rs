use avail_orbit_raas_blueprint_lib::config::OperatorConfig;
use avail_orbit_raas_blueprint_lib::types::RollupMetadata;
use avail_orbit_raas_blueprint_lib::{
    DeploymentStatus, OperatorConfig, OrbitContext, deployment, jobs, util,
};
use axum::{Extension, Json, Router as AxumRouter, routing::get};
use blueprint_sdk::Router;
use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::crypto::sp_core::SpSr25519;
use blueprint_sdk::crypto::tangle_pair_signer::TanglePairSigner;
use blueprint_sdk::keystore::backends::Backend;
use blueprint_sdk::runner::BlueprintRunner;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::runner::tangle::config::TangleConfig;
use blueprint_sdk::tangle::consumer::TangleConsumer;
use blueprint_sdk::tangle::filters::MatchesServiceId;
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::tangle::producer::TangleProducer;
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::filter::FilterLayer;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing::{error, info, warn};

const MODIFY_ROLLUP_METADATA_JOB_ID: u32 = 1;
const RESTART_ROLLUP_JOB_ID: u32 = 2;
const UPDATE_BRIDGE_JOB_ID: u32 = 3;

/// HTTP server state
struct AppState {
    deployment_status: Arc<Mutex<DeploymentStatus>>,
}

#[tokio::main]
async fn main() -> Result<(), blueprint_sdk::Error> {
    // Load environment variables from .env file
    dotenv().ok();

    setup_log();
    info!("Starting Avail Orbit RaaS");

    // Check prerequisites
    check_prerequisites().await;

    // Load operator configuration from environment variables
    let operator_config = load_operator_config()?;

    // Initialize the orbit context with the operator config
    let orbit_ctx = OrbitContext::new(operator_config.clone());
    let deployment_status = orbit_ctx.status.clone();

    // Load rollup metadata from environment variables
    let rollup_metadata = load_rollup_metadata()?;

    // Create the deployment configuration by combining operator config (private) with metadata (public)
    let config = AvailOrbitConfig::new(operator_config, rollup_metadata.clone());

    // Deploy the rollup in a separate task to avoid blocking the main thread
    let ctx_clone = orbit_ctx.clone();
    tokio::spawn(async move {
        info!("Deploying Avail Orbit rollup...");
        match deployment::deploy_rollup(config).await {
            Ok(status) => {
                info!("Rollup deployed successfully!");
                // Update the shared status
                *ctx_clone.status.lock().await = status;
            }
            Err(e) => {
                error!("Failed to deploy rollup: {}", e);
                // Continue with job setup anyway - the user can deploy later via API or job
            }
        }
    });

    // Start the HTTP server in a separate task
    let app_state = AppState {
        deployment_status: deployment_status.clone(),
    };

    tokio::spawn(start_http_server(app_state));

    // Set up Tangle integration for job processing
    let env = BlueprintEnvironment::load()?;
    let sr25519_signer = env.keystore().first_local::<SpSr25519>()?;
    let sr25519_pair = env.keystore().get_secret::<SpSr25519>(&sr25519_signer)?;
    let st25519_signer = TanglePairSigner::new(sr25519_pair.0);

    let tangle_client = env.tangle_client().await?;
    let tangle_producer =
        TangleProducer::finalized_blocks(tangle_client.rpc_client.clone()).await?;
    let tangle_consumer = TangleConsumer::new(tangle_client.rpc_client.clone(), st25519_signer);

    let tangle_config = TangleConfig::default();

    let service_id = env.protocol_settings.tangle()?.service_id.unwrap();
    let result = BlueprintRunner::builder(tangle_config, env)
        .router(
            // Define job routes for state-changing operations only
            // These job functions accept only public data, with no private keys
            Router::new()
                .route(
                    MODIFY_ROLLUP_METADATA_JOB_ID,
                    jobs::modify_rollup_metadata.layer(TangleLayer),
                )
                .route(
                    RESTART_ROLLUP_JOB_ID,
                    jobs::restart_rollup.layer(TangleLayer),
                )
                .route(UPDATE_BRIDGE_JOB_ID, jobs::update_bridge.layer(TangleLayer))
                .layer(FilterLayer::new(MatchesServiceId(service_id)))
                // Use our orbit context (which contains the operator config securely)
                .with_context(orbit_ctx),
        )
        .producer(tangle_producer)
        .consumer(tangle_consumer)
        .with_shutdown_handler(async {
            info!("Shutting down Avail Orbit RaaS...");
        })
        .run()
        .await;

    if let Err(e) = result {
        error!("Runner failed! {e:?}");
    }

    Ok(())
}

/// Load operator configuration from environment variables
fn load_operator_config() -> Result<OperatorConfig, blueprint_sdk::Error> {
    let operator_config = OperatorConfig {
        deployer_private_key: env::var("DEPLOYER_PRIVATE_KEY").map_err(|_| {
            blueprint_sdk::Error::Custom("DEPLOYER_PRIVATE_KEY not set".to_string())
        })?,
        batch_poster_private_key: env::var("BATCH_POSTER_PRIVATE_KEY").map_err(|_| {
            blueprint_sdk::Error::Custom("BATCH_POSTER_PRIVATE_KEY not set".to_string())
        })?,
        validator_private_key: env::var("VALIDATOR_PRIVATE_KEY").map_err(|_| {
            blueprint_sdk::Error::Custom("VALIDATOR_PRIVATE_KEY not set".to_string())
        })?,
        avail_addr_seed: env::var("AVAIL_ADDR_SEED")
            .map_err(|_| blueprint_sdk::Error::Custom("AVAIL_ADDR_SEED not set".to_string()))?,
        fallback_s3_access_key: env::var("FALLBACKS3_ACCESS_KEY").ok(),
        fallback_s3_secret_key: env::var("FALLBACKS3_SECRET_KEY").ok(),
        fallback_s3_region: env::var("FALLBACKS3_REGION").ok(),
        fallback_s3_object_prefix: env::var("FALLBACKS3_OBJECT_PREFIX").ok(),
        fallback_s3_bucket: env::var("FALLBACKS3_BUCKET").ok(),
    };

    info!("Loaded operator configuration from environment");
    Ok(operator_config)
}

/// Load rollup metadata from environment variables
fn load_rollup_metadata() -> Result<RollupMetadata, blueprint_sdk::Error> {
    // Parse chain ID from env var with a fallback value
    let chain_id = env::var("ROLLUP_CHAIN_ID")
        .map(|id| id.parse::<u64>().unwrap_or(412346))
        .unwrap_or(412346);

    // Parse S3 fallback flag
    let fallback_s3_enable = env::var("FALLBACKS3_ENABLE")
        .map(|enable| enable.to_lowercase() == "true")
        .unwrap_or(false);

    let rollup_metadata = RollupMetadata {
        name: env::var("ROLLUP_NAME").unwrap_or_else(|_| "Avail Orbit Rollup".to_string()),
        chain_id,
        avail_app_id: env::var("AVAIL_APP_ID")
            .map_err(|_| blueprint_sdk::Error::Custom("AVAIL_APP_ID not set".to_string()))?,
        parent_chain_rpc: env::var("PARENT_CHAIN_RPC")
            .map_err(|_| blueprint_sdk::Error::Custom("PARENT_CHAIN_RPC not set".to_string()))?,
        fallback_s3_enable,
        local_rpc_endpoint: env::var("ROLLUP_LOCAL_RPC")
            .unwrap_or_else(|_| "http://localhost:8449".to_string()),
        explorer_url: env::var("ROLLUP_EXPLORER_URL")
            .unwrap_or_else(|_| "http://localhost:4000".to_string()),
    };

    info!("Loaded rollup metadata from environment");
    Ok(rollup_metadata)
}

/// Check prerequisites for running the service
async fn check_prerequisites() {
    // Check for Docker
    match util::check_docker_available().await {
        Ok(true) => info!("Docker is available"),
        Ok(false) => warn!("Docker is installed but not responding correctly"),
        Err(e) => error!("Docker check failed: {}", e),
    }

    // Check for Docker Compose
    match util::check_docker_compose_available().await {
        Ok(true) => info!("Docker Compose is available"),
        Ok(false) => warn!("Docker Compose is installed but not responding correctly"),
        Err(e) => error!("Docker Compose check failed: {}", e),
    }

    // Check for NPM
    match util::check_npm_available().await {
        Ok(true) => info!("NPM is available"),
        Ok(false) => warn!("NPM is installed but not responding correctly"),
        Err(e) => error!("NPM check failed: {}", e),
    }

    // Check for Yarn
    match util::check_yarn_available().await {
        Ok(true) => info!("Yarn is available"),
        Ok(false) => warn!("Yarn is installed but not responding correctly"),
        Err(e) => error!("Yarn check failed: {}", e),
    }
}

// Start an HTTP server for querying rollup status
async fn start_http_server(state: AppState) {
    let app = AxumRouter::new()
        // Endpoints for querying rollup state (read-only operations)
        .route("/status", get(get_rollup_status))
        .route("/logs", get(get_deployment_logs))
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state.deployment_status));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("HTTP server listening on {}", addr);

    match axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        Ok(_) => {}
        Err(e) => error!("HTTP server error: {}", e),
    }
}

// HTTP handlers

async fn get_rollup_status(
    Extension(status): Extension<Arc<Mutex<DeploymentStatus>>>,
) -> Json<DeploymentStatus> {
    Json(status.lock().await.clone())
}

async fn get_deployment_logs(
    Extension(status): Extension<Arc<Mutex<DeploymentStatus>>>,
) -> Json<Vec<String>> {
    Json(status.lock().await.logs.clone())
}

async fn health_check() -> &'static str {
    "OK"
}

// Logging setup
fn setup_log() {
    use tracing_subscriber::util::SubscriberInitExt;

    let _ = tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .try_init();
}
