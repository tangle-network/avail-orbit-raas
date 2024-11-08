use crate::{OrbitConfig, OrbitError, Result};
use bollard::container::{
    Config as DockerConfig, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::HostConfig;
use bollard::Docker;
use gadget_sdk::docker::bollard;
use gadget_sdk::docker::bollard::container::LogOutput;
use gadget_sdk::futures::StreamExt;
use gadget_sdk::futures::TryStreamExt;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

pub struct OrbitDocker {
    docker: Docker,
    config: OrbitConfig,
    container_id: Option<String>,
}

impl OrbitDocker {
    pub async fn new(config: OrbitConfig) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults().map_err(OrbitError::Docker)?;

        Ok(Self {
            docker,
            config,
            container_id: None,
        })
    }

    pub async fn pull_image(&self) -> Result<()> {
        println!("Pulling Nitro node image...");

        let options = CreateImageOptions {
            from_image: "availj/avail-nitro-node",
            tag: "v2.1.0-upstream-v3.1.1",
            ..Default::default()
        };

        let mut image_stream = self.docker.create_image(Some(options), None, None);
        while let Some(result) = image_stream.next().await {
            match result {
                Ok(output) => {
                    if let Some(status) = output.status {
                        println!("Pull status: {}", status);
                    }
                }
                Err(e) => return Err(OrbitError::Docker(e)),
            }
        }

        Ok(())
    }

    pub async fn start_container(
        &mut self,
        config_dir: &Path,
        ports: &[(u16, u16)],          // (host_port, container_port)
        volumes: &[(PathBuf, String)], // (host_path, container_path)
    ) -> Result<()> {
        // Prepare port bindings
        let mut port_bindings = HashMap::new();
        for (host_port, container_port) in ports {
            port_bindings.insert(
                format!("{}/tcp", container_port),
                Some(vec![bollard::models::PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.to_string()),
                }]),
            );
        }

        // Prepare volume bindings
        let mut volume_bindings = Vec::new();
        for (host_path, container_path) in volumes {
            volume_bindings.push(format!("{}:{}", host_path.display(), container_path));
        }

        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(volume_bindings),
            ..Default::default()
        };

        // Create container configuration
        let container_config = DockerConfig {
            image: Some("availj/avail-nitro-node:v2.1.0-upstream-v3.1.1"),
            cmd: Some(vec!["--conf.file", "/home/user/.arbitrum/nodeConfig.json"]),
            host_config: Some(host_config),
            ..Default::default()
        };

        // Create container
        let container_name = format!("nitro-node-{}", self.config.chain_id);
        let options = CreateContainerOptions {
            name: container_name.as_str(),
            platform: Some("linux/amd64"),
        };

        let container = self
            .docker
            .create_container(Some(options), container_config)
            .await
            .map_err(OrbitError::Docker)?;

        self.container_id = Some(container.id);

        // Start container
        if let Some(id) = &self.container_id {
            self.docker
                .start_container(id, None::<StartContainerOptions<String>>)
                .await
                .map_err(OrbitError::Docker)?;
        }

        Ok(())
    }

    pub async fn stop_container(&self) -> Result<()> {
        if let Some(id) = &self.container_id {
            self.docker
                .stop_container(id, None::<StopContainerOptions>)
                .await
                .map_err(OrbitError::Docker)?;
        }
        Ok(())
    }

    pub async fn remove_container(&self) -> Result<()> {
        if let Some(id) = &self.container_id {
            let options = RemoveContainerOptions {
                force: true,
                ..Default::default()
            };

            self.docker
                .remove_container(id, Some(options))
                .await
                .map_err(OrbitError::Docker)?;
        }
        Ok(())
    }

    pub async fn get_container_logs(&self) -> Result<Vec<String>> {
        let mut logs = Vec::new();

        if let Some(id) = &self.container_id {
            let options = bollard::container::LogsOptions::<String> {
                stdout: true,
                stderr: true,
                follow: false,
                tail: "100".to_string(),
                ..Default::default()
            };

            let mut log_stream = self.docker.logs(id, Some(options));
            while let Some(result) = log_stream.next().await {
                match result {
                    Ok(output) => match output {
                        bollard::container::LogOutput::StdOut { message } => {
                            logs.push(String::from_utf8_lossy(&message).to_string());
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            logs.push(String::from_utf8_lossy(&message).to_string());
                        }
                        _ => {}
                    },
                    Err(e) => return Err(OrbitError::Docker(e)),
                }
            }
        }

        Ok(logs)
    }

    pub async fn wait_for_healthy(&self, timeout_secs: u64) -> Result<bool> {
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start_time.elapsed() < timeout {
            if let Some(id) = &self.container_id {
                let options = ListContainersOptions::<String> {
                    filters: {
                        let mut filters = HashMap::new();
                        filters.insert("id".to_string(), vec![id.clone()]);
                        filters
                    },
                    ..Default::default()
                };

                let containers = self
                    .docker
                    .list_containers(Some(options))
                    .await
                    .map_err(OrbitError::Docker)?;

                if let Some(container) = containers.first() {
                    if let Some(health) = &container.state {
                        if health == "running" {
                            return Ok(true);
                        }
                    }
                }
            }

            sleep(Duration::from_secs(1)).await;
        }

        Ok(false)
    }

    pub async fn execute_command(&self, cmd: &str) -> Result<String> {
        if let Some(id) = &self.container_id {
            let exec_config = bollard::exec::CreateExecOptions {
                cmd: Some(vec!["/bin/sh", "-c", cmd]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };

            let exec = self
                .docker
                .create_exec(id, exec_config)
                .await
                .map_err(OrbitError::Docker)?;

            let output = self
                .docker
                .start_exec(&exec.id, None)
                .await
                .map_err(OrbitError::Docker)?;

            if let bollard::exec::StartExecResults::Attached { output, .. } = output {
                let mut result = String::new();
                let mut stream = Box::pin(output);
                while let Ok(Some(line)) = stream.try_next().await {
                    match line {
                        LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
                            result.push_str(&String::from_utf8_lossy(&message));
                            result.push('\n');
                        }
                        LogOutput::Console { message } => {
                            result.push_str(&String::from_utf8_lossy(&message));
                            result.push('\n');
                        }
                        _ => {}
                    }
                }
                return Ok(result);
            }
        }

        Err(OrbitError::Config("Container not found".to_string()))
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        self.stop_container().await?;
        self.remove_container().await?;
        self.container_id = None;
        Ok(())
    }
}

// Helper function to create standard volume mappings
pub fn create_standard_volumes(config_dir: &Path) -> Vec<(PathBuf, String)> {
    vec![
        (
            config_dir.join("nodeConfig.json"),
            "/home/user/.arbitrum/nodeConfig.json".to_string(),
        ),
        (
            config_dir.join("orbitSetupScriptConfig.json"),
            "/home/user/.arbitrum/orbitSetupScriptConfig.json".to_string(),
        ),
    ]
}

// Helper function to create standard port mappings
pub fn create_standard_ports() -> Vec<(u16, u16)> {
    vec![
        (8449, 8449), // Main RPC port
        (6070, 6070), // Metrics port
        (6060, 6060), // pprof port
    ]
}
