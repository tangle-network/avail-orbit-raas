use std::path::PathBuf;

use avail_arbitrum_orbit_raas::OrbitConfig;
use avail_arbitrum_orbit_raas::{create_standard_ports, create_standard_volumes, OrbitDocker};
use color_eyre::Result;
use gadget_sdk as sdk;
use gadget_sdk::runners::tangle::TangleConfig;
use gadget_sdk::runners::BlueprintRunner;

async fn run_docker() -> Result<(), gadget_sdk::Error> {
    // Create a new config with defaults
    let mut config = OrbitConfig::default();

    // Set required user-specific values
    config.private_key = "your_private_key_here".to_string();
    config.avail_config.seed = "your_avail_seed_here".to_string();
    config.avail_config.app_id = 1; // Your Avail app ID

    let mut docker = OrbitDocker::new(config)
        .await
        .map_err(Into::<gadget_sdk::Error>::into)?;

    // Pull the Nitro node image
    docker
        .pull_image()
        .await
        .map_err(Into::<gadget_sdk::Error>::into)?;

    // Setup volumes and ports
    let config_dir = PathBuf::from("/tmp/orbit-deployment");
    let volumes = create_standard_volumes(&config_dir);
    let ports = create_standard_ports();

    // Start the container
    docker
        .start_container(&config_dir, &ports, &volumes)
        .await
        .map_err(Into::<gadget_sdk::Error>::into)?;

    // Wait for container to be healthy
    if docker
        .wait_for_healthy(60)
        .await
        .map_err(Into::<gadget_sdk::Error>::into)?
    {
        println!("Container is healthy!");

        // Get logs
        let logs = docker
            .get_container_logs()
            .await
            .map_err(Into::<gadget_sdk::Error>::into)?;
        println!("Container logs: {:?}", logs);
    }

    // Cleanup when done
    docker
        .cleanup()
        .await
        .map_err(Into::<gadget_sdk::Error>::into)?;

    Ok(())
}

#[sdk::main(env)]
async fn main() -> Result<()> {
    // Run the Docker deployment job
    run_docker().await?;

    tracing::info!("Starting the event watcher ...");
    BlueprintRunner::new(TangleConfig::default(), env)
        .run()
        .await?;

    tracing::info!("Exiting...");
    Ok(())
}
