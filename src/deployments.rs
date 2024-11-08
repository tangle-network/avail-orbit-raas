use crate::{run_and_focus_multiple, OrbitConfig, OrbitError, Result};
use gadget_sdk::executor::process::manager::GadgetProcessManager;
use serde_json::json;
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

        // Create .env file
        let env_content = format!(
            "ROLLUP_CREATOR_ADDRESS=\"{}\"\nDEVNET_PRIVKEY=\"{}\"",
            "0xE917553b67f630C3982236B6A1d7844B1021B909", // Arbitrum Sepolia Rollup Creator
            self.config.private_key
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
            (
                "deploy_rollup",
                "yarn run deploy-eth-rollup --network arbSepolia",
            ),
        ];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to deploy contracts: {}", e)))?;

        Ok(())
    }

    async fn setup_orbit_script(&mut self) -> Result<()> {
        let orbit_script_dir = self.working_dir.join("orbit-setup-script");
        let clone_cmd = format!(
            "git clone https://github.com/OffchainLabs/orbit-setup-script.git {}",
            orbit_script_dir.display()
        );

        let commands = vec![("clone_setup", clone_cmd.as_str())];

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
        let orbit_script_dir = self.working_dir.join("orbit-setup-script");
        let cd_cmd = format!("cd {}", orbit_script_dir.display());

        let commands = vec![
            ("cd_setup", cd_cmd.as_str()),
            ("start_chain", "docker-compose up -d"),
        ];

        run_and_focus_multiple(&mut self.manager, commands)
            .await
            .map_err(|e| OrbitError::Command(format!("Failed to start chain: {}", e)))?;

        Ok(())
    }

    async fn create_config_ts(&self, dir: &Path) -> Result<()> {
        let config_content = format!(
            r#"
            const config = {{
                rollupConfig: {{
                    confirmPeriodBlocks: ethers.BigNumber.from('{}'),
                    extraChallengeTimeBlocks: ethers.BigNumber.from('{}'),
                    stakeToken: '{}',
                    baseStake: ethers.utils.parseEther('{}'),
                    wasmModuleRoot: '{}',
                    owner: '{}',
                    loserStakeEscrow: '{}',
                    chainId: ethers.BigNumber.from('{}'),
                    chainConfig: '{}',
                    genesisBlockNum: ethers.BigNumber.from('{}'),
                    sequencerInboxMaxTimeVariation: {{
                        delayBlocks: ethers.BigNumber.from('{}'),
                        futureBlocks: ethers.BigNumber.from('{}'),
                        delaySeconds: ethers.BigNumber.from('{}'),
                        futureSeconds: ethers.BigNumber.from('{}'),
                    }},
                }},
                validators: {},
                batchPosters: {},
                batchPosterManager: '{}',
            }};
            
            export default config;
            "#,
            self.config.rollup_config.confirm_period_blocks,
            self.config.rollup_config.extra_challenge_time_blocks,
            self.config.rollup_config.stake_token,
            self.config.rollup_config.base_stake,
            self.config.rollup_config.wasm_module_root,
            self.config.rollup_config.owner,
            self.config.rollup_config.loser_stake_escrow,
            self.config.rollup_config.chain_id,
            self.config.rollup_config.chain_config,
            self.config.rollup_config.genesis_block_num,
            self.config
                .rollup_config
                .sequencer_inbox_max_time_variation
                .delay_blocks,
            self.config
                .rollup_config
                .sequencer_inbox_max_time_variation
                .future_blocks,
            self.config
                .rollup_config
                .sequencer_inbox_max_time_variation
                .delay_seconds,
            self.config
                .rollup_config
                .sequencer_inbox_max_time_variation
                .future_seconds,
            serde_json::to_string(&self.config.rollup_config.validators)?,
            serde_json::to_string(&self.config.rollup_config.batch_posters)?,
            self.config.rollup_config.batch_poster_manager,
        );

        fs::write(dir.join("scripts/config.ts"), config_content).await?;
        Ok(())
    }

    async fn create_node_config(&self, dir: &Path) -> Result<()> {
        let node_config = json!({
            "chain": self.config.node_config.chain,
            "parent-chain": self.config.node_config.parent_chain,
            "http": self.config.node_config.http,
            "node": self.config.node_config.node,
            "execution": self.config.node_config.execution,
            "metrics": self.config.node_config.metrics,
            "pprof": self.config.node_config.pprof,
            "persistent": self.config.node_config.persistent,
            "validation": self.config.node_config.validation,
        });

        fs::write(
            dir.join("nodeConfig.json"),
            serde_json::to_string_pretty(&node_config)?,
        )
        .await?;

        Ok(())
    }

    async fn create_orbit_setup_config(&self, dir: &Path) -> Result<()> {
        let orbit_config = json!({
            "network_fee_receiver": self.config.orbit_setup_config.network_fee_receiver,
            "infrastructure_fee_collector": self.config.orbit_setup_config.infrastructure_fee_collector,
            "staker": self.config.orbit_setup_config.staker,
            "batch_poster": self.config.orbit_setup_config.batch_poster,
            "chain_owner": self.config.orbit_setup_config.chain_owner,
            "chain_id": self.config.orbit_setup_config.chain_id,
            "chain_name": self.config.orbit_setup_config.chain_name,
            "min_l2_base_fee": self.config.orbit_setup_config.min_l2_base_fee,
            "parent_chain_id": self.config.orbit_setup_config.parent_chain_id,
            "parent_chain_node_url": self.config.orbit_setup_config.parent_chain_node_url,
            "utils": self.config.orbit_setup_config.utils,
            "rollup": self.config.orbit_setup_config.rollup,
            "inbox": self.config.orbit_setup_config.inbox,
            "native_token": self.config.orbit_setup_config.native_token,
            "outbox": self.config.orbit_setup_config.outbox,
            "rollup_event_inbox": self.config.orbit_setup_config.rollup_event_inbox,
            "challenge_manager": self.config.orbit_setup_config.challenge_manager,
            "admin_proxy": self.config.orbit_setup_config.admin_proxy,
            "sequencer_inbox": self.config.orbit_setup_config.sequencer_inbox,
            "bridge": self.config.orbit_setup_config.bridge,
            "upgrade_executor": self.config.orbit_setup_config.upgrade_executor,
            "validator_utils": self.config.orbit_setup_config.validator_utils,
            "validator_wallet_creator": self.config.orbit_setup_config.validator_wallet_creator,
            "deployed_at_block_number": self.config.orbit_setup_config.deployed_at_block_number,
        });

        fs::write(
            dir.join("orbitSetupScriptConfig.json"),
            serde_json::to_string_pretty(&orbit_config)?,
        )
        .await?;

        Ok(())
    }

    async fn update_docker_compose(&self, dir: &Path) -> Result<()> {
        let docker_compose_content = r#"
version: "3.9"
services:
  nitro:
    image: availj/avail-nitro-node:v2.1.0-upstream-v3.1.1
    ports:
      - "8449:8449"
    volumes:
      - ./nodeConfig.json:/home/user/.arbitrum/nodeConfig.json
      - ./orbitSetupScriptConfig.json:/home/user/.arbitrum/orbitSetupScriptConfig.json
    command: --conf.file /home/user/.arbitrum/nodeConfig.json
"#;

        fs::write(dir.join("docker-compose.yml"), docker_compose_content).await?;
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
