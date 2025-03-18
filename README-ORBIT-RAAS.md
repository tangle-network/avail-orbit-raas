# Avail Orbit RaaS - Rollup as a Service

This project provides a secure, automated system for deploying and managing Arbitrum Orbit rollups with AVAIL Data Availability. It's designed to be operated by rollup providers who manage private keys and infrastructure on behalf of users.

## Features

- **Automated Deployment**: Deploy Arbitrum Orbit rollups with AVAIL DA with a single command
- **Secure Key Management**: Private keys are managed securely by the operator, never exposed in job arguments
- **State-Changing Operations**: Update metadata, restart rollup, update token bridge
- **HTTP API**: Query rollup status, logs, and health
- **Blueprint Integration**: Jobs exposed via Tangle Blueprint for blockchain-based management

## Getting Started

### Prerequisites

- Docker and Docker Compose
- npm and Yarn
- Rust toolchain
- Arbitrum Sepolia testnet ETH
- Avail account and application ID

### Setup

1. Clone this repository:

```bash
git clone https://github.com/tangle-network/avail-orbit-raas.git
cd avail-orbit-raas
```

2. Create a `.env` file:

```bash
cp .env.example .env
```

3. Edit the `.env` file with your configuration:
   - Add your Ethereum private keys (deployer, batch poster, validator)
   - Set your Avail seed and app ID
   - Configure parent chain RPC endpoint
   - Optionally enable and configure S3 fallback

### Building

Build the project with:

```bash
cargo build --release
```

### Running

Run the RaaS (Rollup as a Service):

```bash
./target/release/avail-orbit-raas-blueprint-bin
```

The service will:

1. Load configuration from the `.env` file
2. Check for prerequisites
3. Deploy an Arbitrum Orbit rollup with AVAIL DA
4. Start an HTTP server for querying rollup status
5. Set up job handlers for state-changing operations

## Usage

### HTTP API

The HTTP API is available at `http://localhost:3000` by default:

- `GET /status` - Get rollup deployment status
- `GET /logs` - Get deployment logs
- `GET /health` - Check service health

### State-Changing Operations

State-changing operations are available as jobs that can be called via the Tangle Blueprint system:

- `MODIFY_ROLLUP_METADATA_JOB_ID (1)`: Update public rollup metadata
- `RESTART_ROLLUP_JOB_ID (2)`: Restart the rollup containers
- `UPDATE_BRIDGE_JOB_ID (3)`: Update the token bridge

These job functions only accept public metadata and never expose private keys.

## Security

- Private keys are loaded from the `.env` file and never exposed in job arguments
- Job functions only accept public metadata, with private operations managed by the operator
- The `.env` file should be secured and never committed to version control
- In production, consider using a vault or secret manager for key storage

## Troubleshooting

- Check logs with `GET /logs` to see detailed deployment and operation logs
- Ensure Docker, Docker Compose, npm, and Yarn are installed and working
- Verify your Arbitrum Sepolia ETH balance
- Ensure your Avail account is properly set up

## License

[MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE)
