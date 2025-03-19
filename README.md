# Avail Orbit RaaS

A streamlined toolkit for deploying and managing Arbitrum Orbit rollups with Avail Data Availability.

## Overview

This repository provides a Rust-based implementation for deploying, managing, and interacting with Arbitrum Orbit rollups backed by Avail for Data Availability. It encapsulates the complex orchestration of rollup components into a simple, secure API.

## Core Features

- **Secure Key Management**: Environment-based configuration system for sensitive keys
- **Deployment Automation**: One-step rollup deployment with container orchestration
- **Lifecycle Management**: Tools to restart and update your rollup instances
- **Metadata Operations**: Seamlessly update rollup metadata post-deployment
- **Blueprint Integration**: Exposed as a Blueprint library for wider ecosystem integration

## Quick Start

```bash
# Clone the repository
git clone https://github.com/your-org/avail-orbit-raas.git
cd avail-orbit-raas

# Configure your environment
cp .env.example .env
# Edit .env with your actual keys and configuration

# Build the project
cargo build --release

# Try a deployment example
cargo run --example deploy_rollup
```

## Configuration

Configuration is managed through a `.env` file with the following key parameters:

```
# Ethereum Private Keys (KEEP THESE SECURE)
DEPLOYER_PRIVATE_KEY=0x...
BATCH_POSTER_PRIVATE_KEY=0x...
VALIDATOR_PRIVATE_KEY=0x...

# Avail Data Availability Configuration
AVAIL_ADDR_SEED=your-avail-secret-seed
AVAIL_APP_ID=your-avail-app-id

# Parent Chain Configuration
PARENT_CHAIN_RPC=https://sepolia-rollup.arbitrum.io/rpc

# Rollup Configuration
ROLLUP_NAME=MyAvailOrbitRollup
ROLLUP_CHAIN_ID=412346
```

See `.env.example` for the complete configuration reference.

## Usage Examples

The repository includes executable examples demonstrating key functionality:

- **Deploying a Rollup**: `cargo run --example deploy_rollup`
- **Restarting a Rollup**: `cargo run --example restart_rollup`
- **Updating Metadata**: `cargo run --example update_metadata`

Each example provides comprehensive debug logging for troubleshooting.

## Architecture

The codebase is structured around:

- **Core Library** (`avail-orbit-raas-lib`): The library for rollup operations and deployments
- **Binary** (`avail-orbit-raas-bin`): A Tangle Blueprint binary for operator use
- **Blueprint Integration**: Integration with the Tangle ecosystem

## Prerequisites

- Rust toolchain (1.65+)
- Docker and Docker Compose
- Node.js and Yarn
- ETH on Arbitrum Sepolia testnet (for deployment transactions)
- An Avail account with DATA tokens

## Development

```bash
# Run the tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run --example deploy_rollup

# Format code
cargo fmt
```

## Security Considerations

- Never commit `.env` files containing real private keys
- Use environment variables or secure vaults in production
- Review permissions of any containers running with private keys

## License

[MIT License](LICENSE) and [Apache License 2.0](LICENSE-APACHE)
