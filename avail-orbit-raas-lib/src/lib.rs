use std::sync::Arc;
use tokio::sync::Mutex;

// Module declarations
pub mod config;
pub mod deployment;
pub mod jobs;
pub mod types;
pub mod util;

pub use config::*;
pub use types::*;

/// Rollup orchestration context
#[derive(Clone)]
pub struct OrbitContext {
    /// Status of the rollup deployment
    pub status: Arc<Mutex<DeploymentStatus>>,
    /// Operator configuration with private keys (not exposed to blockchain)
    pub operator_config: Arc<Mutex<OperatorConfig>>,
}

impl OrbitContext {
    pub fn new(operator_config: OperatorConfig) -> Self {
        Self {
            status: Arc::new(Mutex::new(DeploymentStatus::default())),
            operator_config: Arc::new(Mutex::new(operator_config)),
        }
    }

    /// Add a log message to the deployment status
    pub async fn log(&self, message: &str) {
        let mut status = self.status.lock().await;
        status.logs.push(message.to_string());
    }
}
