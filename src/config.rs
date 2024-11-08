use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitConfig {
    pub chain_id: u64,
    pub chain_name: String,
    pub creator_address: Address,
    pub private_key: String,
    pub rollup_config: RollupConfig,
    pub node_config: NodeConfig,
    pub orbit_setup_config: OrbitSetupConfig,
    pub avail_config: AvailConfig,
    pub working_dir: PathBuf,
}

impl Default for OrbitConfig {
    fn default() -> Self {
        Self {
            chain_id: 20121999,
            chain_name: "Avail-Orbit-Testnet".to_string(),
            creator_address: Address::default(),
            private_key: "".to_string(), // Must be provided by user
            rollup_config: RollupConfig {
                confirm_period_blocks: 150,
                extra_challenge_time_blocks: 0,
                stake_token: "0x0000000000000000000000000000000000000000".to_string(),
                base_stake: "0.0001".to_string(),
                wasm_module_root: "0x3f3b4da7b5c231e6faf91ff723d235728b05c9074f2ae3cc4b3e54dd5139d34f".to_string(),
                owner: "0x1234123412341234123412341234123412341234".to_string(),
                loser_stake_escrow: "0x0000000000000000000000000000000000000000".to_string(),
                chain_id: 20121999,
                chain_config: r#"{
                    "chainId":20121999,
                    "homesteadBlock":0,
                    "daoForkBlock":null,
                    "daoForkSupport":true,
                    "eip150Block":0,
                    "eip150Hash":"0x0000000000000000000000000000000000000000000000000000000000000000",
                    "eip155Block":0,
                    "eip158Block":0,
                    "byzantiumBlock":0,
                    "constantinopleBlock":0,
                    "petersburgBlock":0,
                    "istanbulBlock":0,
                    "muirGlacierBlock":0,
                    "berlinBlock":0,
                    "londonBlock":0,
                    "clique":{"period":0,"epoch":0},
                    "arbitrum":{
                        "EnableArbOS":true,
                        "AllowDebugPrecompiles":false,
                        "DataAvailabilityCommittee":false,
                        "InitialArbOSVersion":10,
                        "InitialChainOwner":"0xd41996ED89bb5BF7dBfB181D8D93E8067446200B",
                        "GenesisBlockNum":0
                    }
                }"#.to_string(),
                genesis_block_num: 0,
                sequencer_inbox_max_time_variation: SequencerInboxTimeVariation {
                    delay_blocks: 5760,
                    future_blocks: 12,
                    delay_seconds: 86400,
                    future_seconds: 3600,
                },
                validators: vec![
                    "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
                ],
                batch_posters: vec![
                    "0xB0Ad5AE0a78025613F17B7e4644CE5752487B9d6".to_string(),
                ],
                batch_poster_manager: "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
            },
            node_config: NodeConfig {
                chain: ChainInfo {
                    info_json: "".to_string(),
                    name: "Avail-Orbit-Testnet".to_string(),
                },
                parent_chain: ParentChainConfig {
                    connection: ConnectionConfig {
                        url: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
                    },
                },
                http: HttpConfig {
                    addr: "0.0.0.0".to_string(),
                    port: 8449,
                    vhosts: vec!["*".to_string()],
                    corsdomain: vec!["*".to_string()],
                    api: vec![
                        "eth".to_string(),
                        "net".to_string(),
                        "web3".to_string(),
                        "arb".to_string(),
                        "debug".to_string(),
                    ],
                },
                node: NodeSettings {
                    forwarding_target: None,
                    sequencer: SequencerConfig {
                        enable: true,
                        dangerous: Some(DangerousConfig {
                            without_block_validator: true,
                        }),
                    },
                    staker: StakerConfig {
                        enable: true,
                        strategy_config: StrategyConfig {
                            stake_coin_base: true,
                        },
                    },
                    batch_poster: BatchPosterConfig {
                        enable: true,
                        max_items: 10000,
                    },
                    delayed_sequencer: DelayedSequencerConfig {
                        enable: false,
                    },
                },
                execution: ExecutionConfig {
                    forwarding_target: None,
                },
                metrics: MetricsConfig {
                    enable: true,
                    addr: "0.0.0.0".to_string(),
                    port: 6070,
                },
                pprof: PprofConfig {
                    enable: true,
                    addr: "0.0.0.0".to_string(),
                    port: 6060,
                },
                persistent: PersistentConfig {
                    chain: "/home/user/chain".to_string(),
                },
                validation: ValidationConfig {
                    validate_chains: vec![],
                },
            },
            orbit_setup_config: OrbitSetupConfig {
                network_fee_receiver: "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
                infrastructure_fee_collector: "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
                staker: "0xDF819f9Fc3c28FEDFb73374C7B60A4f9BCdE6710".to_string(),
                batch_poster: "0xB0Ad5AE0a78025613F17B7e4644CE5752487B9d6".to_string(),
                chain_owner: "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
                chain_id: 20121999,
                chain_name: "Avail-Orbit-Testnet".to_string(),
                min_l2_base_fee: 100000000,
                parent_chain_id: 421614, // Arbitrum Sepolia
                parent_chain_node_url: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
                // Contract addresses for Arbitrum Sepolia
                utils: None,
                rollup: None,
                inbox: None,
                native_token: None,
                outbox: None,
                rollup_event_inbox: None,
                challenge_manager: None,
                admin_proxy: None,
                sequencer_inbox: None,
                bridge: None,
                upgrade_executor: None,
                validator_utils: None,
                validator_wallet_creator: None,
                deployed_at_block_number: None,
            },
            avail_config: AvailConfig {
                seed: "".to_string(), // Must be provided by user
                api_url: "wss://turing-rpc.avail.so/ws".to_string(),
                app_id: 0, // Must be provided by user
                timeout: "100s".to_string(),
                vectorx: "0xA712dfec48AF3a78419A8FF90fE8f97Ae74680F0".to_string(),
                arb_sepolia_rpc: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
            },
            working_dir: PathBuf::from("/tmp/orbit-deployment"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    pub confirm_period_blocks: u64,
    pub extra_challenge_time_blocks: u64,
    pub stake_token: String,
    pub base_stake: String,
    pub wasm_module_root: String,
    pub owner: String,
    pub loser_stake_escrow: String,
    pub chain_id: u64,
    pub chain_config: String,
    pub genesis_block_num: u64,
    pub sequencer_inbox_max_time_variation: SequencerInboxTimeVariation,
    pub validators: Vec<String>,
    pub batch_posters: Vec<String>,
    pub batch_poster_manager: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerInboxTimeVariation {
    pub delay_blocks: u64,
    pub future_blocks: u64,
    pub delay_seconds: u64,
    pub future_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub chain: ChainInfo,
    pub parent_chain: ParentChainConfig,
    pub http: HttpConfig,
    pub node: NodeSettings,
    pub execution: ExecutionConfig,
    pub metrics: MetricsConfig,
    pub pprof: PprofConfig,
    pub persistent: PersistentConfig,
    pub validation: ValidationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    #[serde(rename = "info-json")]
    pub info_json: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentChainConfig {
    pub connection: ConnectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub addr: String,
    pub port: u16,
    pub vhosts: Vec<String>,
    pub corsdomain: Vec<String>,
    pub api: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSettings {
    pub forwarding_target: Option<String>,
    pub sequencer: SequencerConfig,
    pub staker: StakerConfig,
    pub batch_poster: BatchPosterConfig,
    pub delayed_sequencer: DelayedSequencerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerConfig {
    pub enable: bool,
    pub dangerous: Option<DangerousConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousConfig {
    #[serde(rename = "without-block-validator")]
    pub without_block_validator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakerConfig {
    pub enable: bool,
    pub strategy_config: StrategyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    #[serde(rename = "stake-coin-base")]
    pub stake_coin_base: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPosterConfig {
    pub enable: bool,
    #[serde(rename = "max-items")]
    pub max_items: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayedSequencerConfig {
    pub enable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(rename = "forwarding-target")]
    pub forwarding_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enable: bool,
    pub addr: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PprofConfig {
    pub enable: bool,
    pub addr: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentConfig {
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub validate_chains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitSetupConfig {
    pub network_fee_receiver: String,
    pub infrastructure_fee_collector: String,
    pub staker: String,
    pub batch_poster: String,
    pub chain_owner: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub min_l2_base_fee: u64,
    pub parent_chain_id: u64,
    pub parent_chain_node_url: String,
    pub utils: Option<String>,
    pub rollup: Option<String>,
    pub inbox: Option<String>,
    pub native_token: Option<String>,
    pub outbox: Option<String>,
    pub rollup_event_inbox: Option<String>,
    pub challenge_manager: Option<String>,
    pub admin_proxy: Option<String>,
    pub sequencer_inbox: Option<String>,
    pub bridge: Option<String>,
    pub upgrade_executor: Option<String>,
    pub validator_utils: Option<String>,
    pub validator_wallet_creator: Option<String>,
    pub deployed_at_block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailConfig {
    pub seed: String,
    pub api_url: String,
    pub app_id: u32,
    pub timeout: String,
    pub vectorx: String,
    pub arb_sepolia_rpc: String,
}

impl OrbitConfig {
    pub fn new(
        chain_id: u64,
        chain_name: impl Into<String>,
        creator_address: Address,
        private_key: impl Into<String>,
        avail_config: AvailConfig,
        working_dir: impl Into<PathBuf>,
    ) -> Self {
        let chain_name = chain_name.into();
        Self {
            chain_id,
            chain_name: chain_name.clone(),
            private_key: private_key.into(),
            rollup_config: RollupConfig::default(chain_id),
            node_config: NodeConfig::default(),
            orbit_setup_config: OrbitSetupConfig::default(chain_id, &chain_name),
            creator_address,
            avail_config,
            working_dir: working_dir.into(),
        }
    }
}

impl RollupConfig {
    fn default(chain_id: u64) -> Self {
        Self {
            confirm_period_blocks: 150,
            extra_challenge_time_blocks: 0,
            stake_token: "0x0000000000000000000000000000000000000000".to_string(),
            base_stake: "0.0001".to_string(),
            wasm_module_root: "0x3f3b4da7b5c231e6faf91ff723d235728b05c9074f2ae3cc4b3e54dd5139d34f"
                .to_string(),
            owner: "0x1234123412341234123412341234123412341234".to_string(),
            loser_stake_escrow: "0x0000000000000000000000000000000000000000".to_string(),
            chain_id,
            chain_config: format!(
                r#"{{"chainId":{},"homesteadBlock":0,"daoForkBlock":null,"daoForkSupport":true,"eip150Block":0,"eip150Hash":"0x0000000000000000000000000000000000000000000000000000000000000000","eip155Block":0,"eip158Block":0,"byzantiumBlock":0,"constantinopleBlock":0,"petersburgBlock":0,"istanbulBlock":0,"muirGlacierBlock":0,"berlinBlock":0,"londonBlock":0,"clique":{{"period":0,"epoch":0}},"arbitrum":{{"EnableArbOS":true,"AllowDebugPrecompiles":false,"DataAvailabilityCommittee":false,"InitialArbOSVersion":10,"InitialChainOwner":"0xd41996ED89bb5BF7dBfB181D8D93E8067446200B","GenesisBlockNum":0}}}}"#,
                chain_id
            ),
            genesis_block_num: 0,
            sequencer_inbox_max_time_variation: SequencerInboxTimeVariation {
                delay_blocks: 5760,
                future_blocks: 12,
                delay_seconds: 86400,
                future_seconds: 3600,
            },
            validators: vec!["0x1234123412341234123412341234123412341234".to_string()],
            batch_posters: vec!["0x1234123412341234123412341234123412341234".to_string()],
            batch_poster_manager: "0x1234123412341234123412341234123412341234".to_string(),
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            chain: ChainInfo {
                info_json: "".to_string(),
                name: "".to_string(),
            },
            parent_chain: ParentChainConfig {
                connection: ConnectionConfig {
                    url: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
                },
            },
            http: HttpConfig {
                addr: "0.0.0.0".to_string(),
                port: 8449,
                vhosts: vec!["*".to_string()],
                corsdomain: vec!["*".to_string()],
                api: vec![
                    "eth".to_string(),
                    "net".to_string(),
                    "web3".to_string(),
                    "arb".to_string(),
                    "debug".to_string(),
                ],
            },
            node: NodeSettings {
                forwarding_target: None,
                sequencer: SequencerConfig {
                    enable: true,
                    dangerous: Some(DangerousConfig {
                        without_block_validator: true,
                    }),
                },
                staker: StakerConfig {
                    enable: true,
                    strategy_config: StrategyConfig {
                        stake_coin_base: true,
                    },
                },
                batch_poster: BatchPosterConfig {
                    enable: true,
                    max_items: 10000,
                },
                delayed_sequencer: DelayedSequencerConfig { enable: false },
            },
            execution: ExecutionConfig {
                forwarding_target: None,
            },
            metrics: MetricsConfig {
                enable: true,
                addr: "0.0.0.0".to_string(),
                port: 6070,
            },
            pprof: PprofConfig {
                enable: true,
                addr: "0.0.0.0".to_string(),
                port: 6060,
            },
            persistent: PersistentConfig {
                chain: "/home/user/chain".to_string(),
            },
            validation: ValidationConfig {
                validate_chains: vec![],
            },
        }
    }
}

impl OrbitSetupConfig {
    pub fn default(chain_id: u64, chain_name: &str) -> Self {
        Self {
            network_fee_receiver: "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
            infrastructure_fee_collector: "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
            staker: "0xDF819f9Fc3c28FEDFb73374C7B60A4f9BCdE6710".to_string(),
            batch_poster: "0xB0Ad5AE0a78025613F17B7e4644CE5752487B9d6".to_string(),
            chain_owner: "0xd41996ED89bb5BF7dBfB181D8D93E8067446200B".to_string(),
            chain_id,
            chain_name: chain_name.to_string(),
            min_l2_base_fee: 100000000,
            parent_chain_id: 421614,
            parent_chain_node_url: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
            utils: Some("0xB11EB62DD2B352886A4530A9106fE427844D515f".to_string()),
            rollup: Some("0xd30eCcf27A6f351EfA4fc9D17e7ec20354309aE3".to_string()),
            inbox: Some("0x4512e40a1ec8555f9e93E3B6a06af60F13538087".to_string()),
            native_token: Some("0x0000000000000000000000000000000000000000".to_string()),
            outbox: Some("0x2209755fA3470ED1AFFB4407d1e3B1f7dFC13ce9".to_string()),
            rollup_event_inbox: Some("0x1e58240B2D769de25B4811354819C901317D0894".to_string()),
            challenge_manager: Some("0xfC5BbC40d24EcD6FcC247EfFDc87E7D074E9B67D".to_string()),
            admin_proxy: Some("0xf488b25e6736Ed74E8d37EA434892129E4d62E3B".to_string()),
            sequencer_inbox: Some("0xD15347309854F1290c9a382ea2719AB5462c7719".to_string()),
            bridge: Some("0xC83ee8e28B7b258f41aF8ef4279c02f901288029".to_string()),
            upgrade_executor: Some("0x805bB07B88dDA56030eC48644E0C276e2e5E3949".to_string()),
            validator_utils: Some("0xB11EB62DD2B352886A4530A9106fE427844D515f".to_string()),
            validator_wallet_creator: Some(
                "0xEb9885B6c0e117D339F47585cC06a2765AaE2E0b".to_string(),
            ),
            deployed_at_block_number: Some(11274529),
        }
    }
}
