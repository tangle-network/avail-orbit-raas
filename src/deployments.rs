use crate::{run_and_focus_multiple, OrbitConfig, OrbitError, Result};
use gadget_sdk::executor::process::manager::GadgetProcessManager;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct OrbitDeployment {
    config: OrbitConfig,
    manager: GadgetProcessManager,
    working_dir: PathBuf,
}

impl OrbitDeployment {
    pub fn new(config: OrbitConfig) -> Self {
        Self {
            config,
            manager: GadgetProcessManager::new(),
            working_dir: PathBuf::from("/tmp/orbit-deployment"),
        }
    }

    pub async fn deploy(&mut self) -> Result<()> {
        // Create working directory
        fs::create_dir_all(&self.working_dir).await?;

        // Step 1: Pull Docker image
        self.pull_docker_image().await?;

        // Step 2: Clone and setup nitro-contracts
        self.setup_nitro_contracts().await?;

        // Step 3: Deploy contracts
        self.deploy_contracts().await?;

        // Step 4: Setup orbit-setup-script
        self.setup_orbit_script().await?;

        // Step 5: Start the chain
        self.start_chain().await?;

        Ok(())
    }

    async fn pull_docker_image(&mut self) -> Result<()> {
        let commands = vec![(
            "pull_image",
            "docker pull availj/avail-nitro-node:v2.1.0-upstream-v3.1.1",
        )];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to pull Docker image: {}", e)))?;

        Ok(())
    }

    async fn setup_nitro_contracts(&mut self) -> Result<()> {
        let nitro_contracts_dir = self.working_dir.join("nitro-contracts");
        let clone_cmd = format!(
            "git clone https://github.com/availproject/nitro-contracts.git {}",
            nitro_contracts_dir.display()
        );
        let cd_cmd = format!("cd {}", nitro_contracts_dir.display());

        let commands = vec![
            ("clone_contracts", clone_cmd.as_str()),
            ("cd_contracts", cd_cmd.as_str()),
            ("checkout_version", "git checkout v2.1.0-upstream-v2.1.0"),
            ("install_deps", "yarn install"),
            ("build", "yarn build"),
        ];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to setup nitro-contracts: {}", e)))?;

        // Create .env file and set the Rollup Creator Address
        let env_content = format!(
            "ROLLUP_CREATOR_ADDRESS=\"{}\"\nDEVNET_PRIVKEY=\"{}\"",
            self.config.creator_address, self.config.private_key
        );
        fs::write(nitro_contracts_dir.join(".env"), env_content).await?;

        Ok(())
    }

    async fn deploy_contracts(&mut self) -> Result<()> {
        let nitro_contracts_dir = self.working_dir.join("nitro-contracts");

        // Create config.ts
        self.create_config_ts(&nitro_contracts_dir).await?;

        let cd_cmd = format!("cd {}", nitro_contracts_dir.display());
        let commands = vec![
            ("cd_contracts", cd_cmd.as_str()),
            ("build_forge_yul", "yarn build:forge:yul"),
            (
                "deploy_rollup",
                "yarn run deploy-eth-rollup --network arbSepolia",
            ),
        ];

        let output: HashMap<String, String> = run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to deploy contracts: {}", e)))?;

        // Helper closure to extract address from output
        let extract_address = |pattern: &str| {
            output.get("deploy_rollup").and_then(|out| {
                out.lines()
                    .find(|line| line.contains(pattern))
                    .and_then(|line| line.split_whitespace().last())
                    .map(String::from)
            })
        };

        // Extract all contract addresses
        let rollup = extract_address("RollupProxy Contract created at address:");
        let inbox = extract_address("Inbox (proxy) Contract created at address:");
        let outbox = extract_address("Outbox (proxy) Contract created at address:");
        let rollup_event_inbox =
            extract_address("rollupEventInbox (proxy) Contract created at address:");
        let challenge_manager =
            extract_address("challengeManager (proxy) Contract created at address:");
        let admin_proxy = extract_address("AdminProxy Contract created at address:");
        let sequencer_inbox = extract_address("SequencerInbox (proxy) created at address:");
        let bridge = extract_address("Bridge (proxy) Contract created at address:");
        let validator_utils = extract_address("ValidatorUtils Contract created at address:");
        let validator_wallet_creator =
            extract_address("ValidatorWalletCreator Contract created at address:");

        let deployed_block_number = output.get("deploy_rollup").and_then(|out| {
            out.lines()
                .find(|line| line.contains("All deployed at block number:"))
                .and_then(|line| line.split_whitespace().last())
                .and_then(|num| num.parse::<u64>().ok())
        });

        // Update config with extracted contract addresses
        self.config.orbit_setup_config.rollup = rollup;
        self.config.orbit_setup_config.inbox = inbox;
        self.config.orbit_setup_config.outbox = outbox;
        self.config.orbit_setup_config.rollup_event_inbox = rollup_event_inbox;
        self.config.orbit_setup_config.challenge_manager = challenge_manager;
        self.config.orbit_setup_config.admin_proxy = admin_proxy;
        self.config.orbit_setup_config.sequencer_inbox = sequencer_inbox;
        self.config.orbit_setup_config.bridge = bridge;
        self.config.orbit_setup_config.validator_utils = validator_utils;
        self.config.orbit_setup_config.validator_wallet_creator = validator_wallet_creator;
        self.config.orbit_setup_config.deployed_at_block_number = deployed_block_number;

        Ok(())
    }

    async fn setup_orbit_script(&mut self) -> Result<()> {
        let orbit_script_dir = self.working_dir.join("orbit-setup-script");
        let clone_cmd = format!(
            "git clone https://github.com/OffchainLabs/orbit-setup-script.git {}",
            orbit_script_dir.display()
        );

        let cd_cmd = format!("cd {}", orbit_script_dir.display());
        let commands = vec![
            ("clone_setup", clone_cmd.as_str()),
            ("cd_setup", cd_cmd.as_str()),
        ];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to setup orbit script: {}", e)))?;

        // Create configuration files
        self.create_node_config(&orbit_script_dir).await?;
        self.create_orbit_setup_config(&orbit_script_dir).await?;
        self.update_docker_compose(&orbit_script_dir).await?;

        Ok(())
    }

    async fn start_chain(&mut self) -> Result<()> {
        let commands = vec![("start_chain", "docker-compose up -d")];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to start chain: {}", e)))?;

        Ok(())
    }

    async fn create_config_ts(&self, dir: &Path) -> Result<()> {
        const CONFIG_TS_TEMPLATE: &str = include_str!("templates/config.ts.template");

        let config_content = CONFIG_TS_TEMPLATE
            .replace(
                "${confirm_period_blocks}",
                &self.config.rollup_config.confirm_period_blocks.to_string(),
            )
            .replace(
                "${extra_challenge_time_blocks}",
                &self
                    .config
                    .rollup_config
                    .extra_challenge_time_blocks
                    .to_string(),
            )
            .replace("${base_stake}", &self.config.rollup_config.base_stake)
            .replace(
                "${wasm_module_root}",
                &self.config.rollup_config.wasm_module_root,
            )
            .replace("${owner}", &self.config.rollup_config.owner)
            .replace(
                "${chain_id}",
                &self.config.rollup_config.chain_id.to_string(),
            )
            .replace("${chain_config}", &self.config.rollup_config.chain_config)
            .replace(
                "${genesis_block_num}",
                &self.config.rollup_config.genesis_block_num.to_string(),
            )
            .replace(
                "${delay_blocks}",
                &self
                    .config
                    .rollup_config
                    .sequencer_inbox_max_time_variation
                    .delay_blocks
                    .to_string(),
            )
            .replace(
                "${future_blocks}",
                &self
                    .config
                    .rollup_config
                    .sequencer_inbox_max_time_variation
                    .future_blocks
                    .to_string(),
            )
            .replace(
                "${delay_seconds}",
                &self
                    .config
                    .rollup_config
                    .sequencer_inbox_max_time_variation
                    .delay_seconds
                    .to_string(),
            )
            .replace(
                "${future_seconds}",
                &self
                    .config
                    .rollup_config
                    .sequencer_inbox_max_time_variation
                    .future_seconds
                    .to_string(),
            )
            .replace("${validator_1}", &self.config.rollup_config.validators[0])
            .replace("${validator_2}", &self.config.rollup_config.validators[1])
            .replace(
                "${batch_poster}",
                &self.config.rollup_config.batch_poster_manager,
            );

        fs::write(dir.join("scripts/config.ts"), config_content).await?;
        Ok(())
    }

    async fn create_node_config(&self, dir: &Path) -> Result<()> {
        const NODE_CONFIG_TEMPLATE: &str = include_str!("templates/node_config.json.template");

        let config_content = NODE_CONFIG_TEMPLATE
            .replace("${chain_id}", &self.config.chain_id.to_string())
            .replace("${chain_name}", &self.config.chain_name)
            .replace(
                "${chain_owner}",
                &self.config.orbit_setup_config.chain_owner,
            )
            .replace(
                "${bridge}",
                self.config
                    .orbit_setup_config
                    .bridge
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${inbox}",
                self.config
                    .orbit_setup_config
                    .inbox
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${sequencer_inbox}",
                self.config
                    .orbit_setup_config
                    .sequencer_inbox
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${rollup}",
                self.config
                    .orbit_setup_config
                    .rollup
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${validator_utils}",
                self.config
                    .orbit_setup_config
                    .validator_utils
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${validator_wallet_creator}",
                self.config
                    .orbit_setup_config
                    .validator_wallet_creator
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${deployed_at}",
                &self
                    .config
                    .orbit_setup_config
                    .deployed_at_block_number
                    .unwrap_or(0)
                    .to_string(),
            )
            .replace("${private_key}", &self.config.private_key)
            .replace("${avail_seed}", &self.config.avail_config.seed)
            .replace(
                "${avail_app_id}",
                &self.config.avail_config.app_id.to_string(),
            )
            .replace(
                "${arb_sepolia_rpc}",
                &self.config.avail_config.arb_sepolia_rpc,
            );

        fs::write(dir.join("config/nodeConfig.json"), config_content).await?;
        Ok(())
    }

    async fn create_orbit_setup_config(&self, dir: &Path) -> Result<()> {
        const ORBIT_SETUP_CONFIG_TEMPLATE: &str =
            include_str!("templates/orbit_setup_config.json.template");

        let config_content = ORBIT_SETUP_CONFIG_TEMPLATE
            .replace(
                "${network_fee_receiver}",
                &self.config.orbit_setup_config.network_fee_receiver,
            )
            .replace(
                "${infrastructure_fee_collector}",
                &self.config.orbit_setup_config.infrastructure_fee_collector,
            )
            .replace("${staker}", &self.config.orbit_setup_config.staker)
            .replace(
                "${batch_poster}",
                &self.config.orbit_setup_config.batch_poster,
            )
            .replace(
                "${chain_owner}",
                &self.config.orbit_setup_config.chain_owner,
            )
            .replace(
                "${chain_id}",
                &self.config.orbit_setup_config.chain_id.to_string(),
            )
            .replace("${chain_name}", &self.config.orbit_setup_config.chain_name)
            .replace(
                "${min_l2_base_fee}",
                &self.config.orbit_setup_config.min_l2_base_fee.to_string(),
            )
            .replace(
                "${utils}",
                self.config
                    .orbit_setup_config
                    .utils
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${rollup}",
                self.config
                    .orbit_setup_config
                    .rollup
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${inbox}",
                self.config
                    .orbit_setup_config
                    .inbox
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${native_token}",
                self.config
                    .orbit_setup_config
                    .native_token
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${outbox}",
                self.config
                    .orbit_setup_config
                    .outbox
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${rollup_event_inbox}",
                self.config
                    .orbit_setup_config
                    .rollup_event_inbox
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${challenge_manager}",
                self.config
                    .orbit_setup_config
                    .challenge_manager
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${admin_proxy}",
                self.config
                    .orbit_setup_config
                    .admin_proxy
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${sequencer_inbox}",
                self.config
                    .orbit_setup_config
                    .sequencer_inbox
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${bridge}",
                self.config
                    .orbit_setup_config
                    .bridge
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${upgrade_executor}",
                self.config
                    .orbit_setup_config
                    .upgrade_executor
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${validator_utils}",
                self.config
                    .orbit_setup_config
                    .validator_utils
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${validator_wallet_creator}",
                self.config
                    .orbit_setup_config
                    .validator_wallet_creator
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )
            .replace(
                "${deployed_at_block_number}",
                &self
                    .config
                    .orbit_setup_config
                    .deployed_at_block_number
                    .unwrap_or(0)
                    .to_string(),
            );

        fs::write(
            dir.join("config/orbitSetupScriptConfig.json"),
            config_content,
        )
        .await?;
        Ok(())
    }

    async fn update_docker_compose(&self, dir: &Path) -> Result<()> {
        // Read the existing docker-compose.yml
        let docker_compose_path = dir.join("docker-compose.yml");
        let content = fs::read_to_string(&docker_compose_path).await?;

        // Create a regex pattern to match the nitro service section
        let nitro_regex = regex::Regex::new(r"(?ms)  nitro:\n.*?image: .*?\n").unwrap();

        // Replace with our Avail nitro image
        let updated_content = nitro_regex.replace(
            &content,
            "  nitro:\n    image: availj/avail-nitro-node:v2.1.0-upstream-v3.1.1\n",
        );

        // Write the updated content back to the file
        fs::write(docker_compose_path, updated_content.as_ref()).await?;

        Ok(())
    }

    pub async fn deposit_eth(&mut self, amount: &str) -> Result<()> {
        let cmd = format!(
            "PRIVATE_KEY=\"{}\" L2_RPC_URL=\"https://sepolia-rollup.arbitrum.io/rpc\" L3_RPC_URL=\"http://localhost:8449\" AMOUNT=\"{}\" yarn run deposit",
            self.config.private_key, amount
        );

        let commands = vec![("deposit_eth", cmd.as_str())];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to deposit ETH: {}", e)))?;

        Ok(())
    }

    pub async fn refund(&mut self, target_address: &str) -> Result<()> {
        let cmd = format!(
            "PRIVATE_KEY=\"{}\" L2_RPC_URL=\"https://sepolia-rollup.arbitrum.io/rpc\" TARGET_ADDRESS=\"{}\" yarn run refund",
            self.config.private_key, target_address
        );

        let commands = vec![("refund", cmd.as_str())];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to process refund: {}", e)))?;

        Ok(())
    }

    pub async fn view_logs(&mut self) -> Result<()> {
        let commands = vec![("view_logs", "docker-compose logs -f nitro")];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to view logs: {}", e)))?;

        Ok(())
    }
}
