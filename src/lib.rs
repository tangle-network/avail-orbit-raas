pub mod config;
pub mod deployments;
pub mod docker;
pub mod error;
pub mod jobs;

pub use config::*;
pub use docker::*;
pub use error::*;

// Re-export key types for convenience
use gadget_sdk::executor::process::manager::GadgetProcessManager;
use std::collections::HashMap;

pub async fn run_and_focus_multiple<'a>(
    manager: &mut GadgetProcessManager,
    commands: Vec<(&'a str, &'a str)>,
) -> Result<HashMap<String, String>> {
    let mut outputs = HashMap::new();
    for (name, command) in commands {
        let service = manager
            .run(name.to_string(), command)
            .await
            .map_err(|e| OrbitError::Command(e.to_string()))?;
        let output = manager
            .focus_service_to_completion(service)
            .await
            .map_err(|e| OrbitError::Command(e.to_string()))?;
        outputs.insert(name.to_string(), output);
    }
    Ok(outputs)
}
