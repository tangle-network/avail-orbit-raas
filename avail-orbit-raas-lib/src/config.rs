//! Configuration for Avail Orbit RaaS
//!
//! This module contains the configuration structures for the Avail Orbit RaaS system.
//! OperatorConfig contains sensitive information like private keys and is never exposed in job arguments.
//! The AvailOrbitConfig is derived from operator config + rollup metadata for deployment.

use crate::types::RollupMetadata;
use serde::{Deserialize, Serialize};

/// Operator configuration containing private keys
///
/// This configuration is kept secure on the operator's system and is never
/// exposed through job arguments or public interfaces
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatorConfig {
    /// Deployer private key
    pub deployer_private_key: String,
    /// Batch poster private key
    pub batch_poster_private_key: String,
    /// Validator private key
    pub validator_private_key: String,
    /// Avail address seed
    pub avail_addr_seed: String,
    /// S3 Fallback credentials (optional)
    pub fallback_s3_access_key: Option<String>,
    pub fallback_s3_secret_key: Option<String>,
    pub fallback_s3_region: Option<String>,
    pub fallback_s3_object_prefix: Option<String>,
    pub fallback_s3_bucket: Option<String>,
}

/// Configuration for deploying an Avail Orbit rollup
///
/// This is constructed by combining the operator config with public rollup metadata.
/// Used internally for deployment but not directly exposed in job arguments.
#[derive(Clone, Debug)]
pub struct AvailOrbitConfig {
    /// Private keys and sensitive data from operator config
    operator_config: OperatorConfig,
    /// Public rollup metadata
    metadata: RollupMetadata,
}

impl AvailOrbitConfig {
    /// Create a new config by combining operator config with rollup metadata
    pub fn new(operator_config: OperatorConfig, metadata: RollupMetadata) -> Self {
        Self {
            operator_config,
            metadata,
        }
    }

    /// Generate environment content for this configuration
    pub fn generate_env_content(&self) -> String {
        let mut content = String::new();

        // Add deployment keys from operator config
        content.push_str(&format!(
            "DEPLOYER_PRIVATE_KEY={}\n",
            self.operator_config.deployer_private_key
        ));
        content.push_str(&format!(
            "BATCH_POSTER_PRIVATE_KEY={}\n",
            self.operator_config.batch_poster_private_key
        ));
        content.push_str(&format!(
            "VALIDATOR_PRIVATE_KEY={}\n",
            self.operator_config.validator_private_key
        ));

        // Add Avail params
        content.push_str(&format!(
            "AVAIL_ADDR_SEED={}\n",
            self.operator_config.avail_addr_seed
        ));
        content.push_str(&format!("AVAIL_APP_ID={}\n", self.metadata.avail_app_id));

        // Add S3 fallback if enabled
        content.push_str(&format!(
            "FALLBACKS3_ENABLE={}\n",
            self.metadata.fallback_s3_enable
        ));
        if self.metadata.fallback_s3_enable {
            if let Some(val) = &self.operator_config.fallback_s3_access_key {
                content.push_str(&format!("FALLBACKS3_ACCESS_KEY={}\n", val));
            }
            if let Some(val) = &self.operator_config.fallback_s3_secret_key {
                content.push_str(&format!("FALLBACKS3_SECRET_KEY={}\n", val));
            }
            if let Some(val) = &self.operator_config.fallback_s3_region {
                content.push_str(&format!("FALLBACKS3_REGION={}\n", val));
            }
            if let Some(val) = &self.operator_config.fallback_s3_object_prefix {
                content.push_str(&format!("FALLBACKS3_OBJECT_PREFIX={}\n", val));
            }
            if let Some(val) = &self.operator_config.fallback_s3_bucket {
                content.push_str(&format!("FALLBACKS3_BUCKET={}\n", val));
            }
        }

        // Add parent chain RPC
        content.push_str(&format!(
            "PARENT_CHAIN_RPC={}\n",
            self.metadata.parent_chain_rpc
        ));

        content
    }

    /// Get the deployer private key
    pub fn get_deployer_private_key(&self) -> &str {
        &self.operator_config.deployer_private_key
    }

    /// Get the Avail app ID
    pub fn get_avail_app_id(&self) -> &str {
        &self.metadata.avail_app_id
    }

    /// Get the parent chain RPC endpoint
    pub fn get_parent_chain_rpc(&self) -> &str {
        &self.metadata.parent_chain_rpc
    }

    /// Check if S3 fallback is enabled
    pub fn is_fallback_s3_enabled(&self) -> bool {
        self.metadata.fallback_s3_enable
    }

    /// Get the rollup metadata
    pub fn get_metadata(&self) -> &RollupMetadata {
        &self.metadata
    }
}
