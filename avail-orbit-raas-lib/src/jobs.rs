//! Job functions for Avail Orbit RaaS
//!
//! This module contains the job functions that can be called via blockchain transactions.
//! These functions only accept public metadata as input, with no private keys or sensitive data.

use crate::OrbitContext;
use crate::deployment::{restart_containers, update_metadata, update_rollup_bridge};
use crate::types::RollupMetadata;
use blueprint_sdk::extract::Context;
use blueprint_sdk::tangle::extract::{TangleArg, TangleResult};

/// Modify rollup metadata
///
/// This job allows updating the public metadata of a deployed rollup.
/// Private keys and sensitive data are managed by the operator and not exposed.
pub async fn modify_rollup_metadata(
    Context(ctx): Context<OrbitContext>,
    TangleArg(metadata): TangleArg<RollupMetadata>,
) -> Result<TangleResult<String>, blueprint_sdk::Error> {
    match update_metadata(&ctx, &metadata).await {
        Ok(_) => Ok(TangleResult(
            "Rollup metadata successfully updated".to_string(),
        )),
        Err(e) => Ok(TangleResult(format!(
            "Failed to update rollup metadata: {}",
            e
        ))),
    }
}

/// Restart the rollup
///
/// This job restarts the rollup with the current configuration.
/// No private data is needed for this operation.
pub async fn restart_rollup(
    Context(ctx): Context<OrbitContext>,
) -> Result<TangleResult<String>, blueprint_sdk::Error> {
    match restart_containers(&ctx).await {
        Ok(_) => Ok(TangleResult("Rollup successfully restarted".to_string())),
        Err(e) => Ok(TangleResult(format!("Failed to restart rollup: {}", e))),
    }
}

/// Update the token bridge
///
/// This job updates the token bridge configuration and redeploys it.
/// Private keys are managed by the operator and not exposed in job parameters.
pub async fn update_bridge(
    Context(ctx): Context<OrbitContext>,
) -> Result<TangleResult<String>, blueprint_sdk::Error> {
    match update_rollup_bridge(&ctx).await {
        Ok(_) => Ok(TangleResult(
            "Token bridge successfully updated".to_string(),
        )),
        Err(e) => Ok(TangleResult(format!(
            "Failed to update token bridge: {}",
            e
        ))),
    }
}
