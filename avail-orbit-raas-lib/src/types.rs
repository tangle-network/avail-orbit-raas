//! Type definitions for Avail Orbit RaaS

use serde::{Deserialize, Serialize};

/// Deployment status for the rollup
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeploymentStatus {
    /// Is the rollup deployed
    pub deployed: bool,
    /// Deployment logs
    pub logs: Vec<String>,
    /// Public rollup metadata
    pub metadata: Option<RollupMetadata>,
    /// Docker container IDs
    pub container_ids: Vec<String>,
}

impl Default for DeploymentStatus {
    fn default() -> Self {
        Self {
            deployed: false,
            logs: Vec::new(),
            metadata: None,
            container_ids: Vec::new(),
        }
    }
}

/// Public metadata about the rollup - contains no private keys
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollupMetadata {
    /// Rollup name
    pub name: String,
    /// Rollup chain ID
    pub chain_id: u64,
    /// Avail app ID
    pub avail_app_id: String,
    /// Parent chain RPC endpoint (public endpoint)
    pub parent_chain_rpc: String,
    /// Whether S3 fallback is enabled
    pub fallback_s3_enable: bool,
    /// Local RPC endpoint for the rollup
    pub local_rpc_endpoint: String,
    /// Explorer URL
    pub explorer_url: String,
}
