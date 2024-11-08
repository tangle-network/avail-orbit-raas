# <h1 align="center"> Arbitrum Orbit with Avail DA 🌐 </h1>

**Deploy your own Arbitrum Orbit L2 chain using Avail's data availability layer**

## 📚 Overview

This template helps you deploy an Arbitrum Orbit chain - a customizable L2/L3 solution that inherits Arbitrum's security while using Avail for data availability.

Arbitrum is a leading Ethereum L2 scaling solution that uses optimistic rollups to enable faster and cheaper transactions while maintaining Ethereum's security guarantees. Orbit chains extend this by allowing you to deploy your own L2 on top of Arbitrum.

Avail provides a specialized data availability layer that ensures transaction data remains accessible and verifiable without requiring full node operation. By combining Arbitrum's proven L2/L3 technology with Avail's data availability solution, you get:

- Lower costs through off-chain data availability
- Strong security inherited from Arbitrum
- Customizable L2/L3 chain with your own parameters
- Scalable and verifiable data availability through Avail

## 🏗️ Architecture

The template implements an Arbitrum Orbit chain with the following components:

- **Arbitrum Nitro Node**: The core L2/L3 chain node that processes transactions and maintains state
- **Avail Data Availability**: Transaction data is posted to Avail instead of Ethereum/Arbitrum
- **Tangle Network**: Used for coordinating the deployment and management of the chain
- **Docker**: Containerized deployment of the Nitro node and related services

The flow works as follows:

1. Deploy the Orbit chain contracts using the Arbitrum SDK
2. Configure the Nitro node to use Avail for data availability 
3. Start the node container and initialize the chain
4. Post transaction batches to Avail instead of the parent chain
5. Manage the chain through the Tangle Network blueprint

This implementation follows:
- [Avail's Arbitrum Nitro Stack Tutorial](https://docs.availproject.org/docs/build-with-avail/Optimium/arbitrum-nitro/nitro-stack#step-2-create-env-file)
- [Arbitrum's Official Orbit Chain Deployment Guide](https://docs.arbitrum.io/launch-orbit-chain/how-tos/orbit-sdk-deploying-rollup-chain)

## 📚 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Forge](https://getfoundry.sh)
- [Tangle](https://github.com/tangle-network/tangle?tab=readme-ov-file#-getting-started-)

You will also need to install [cargo-tangle](https://crates.io/crates/cargo-tangle), our CLI tool for creating and
deploying Tangle Blueprints:

To install the Tangle CLI, run the following command:

> Supported on Linux, MacOS, and Windows (WSL2)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tangle-network/gadget/releases/download/cargo-tangle-v0.1.2/cargo-tangle-installer.sh | sh
```

Or, if you prefer to install the CLI from crates.io:

```bash
cargo install cargo-tangle --force # to get the latest version.
```

## 🛠️ Development

Once you have created a new project, you can run the following command to start the project:

```sh
cargo build
```

to build the project, and

```sh
cargo tangle blueprint deploy
```

to deploy the blueprint to the Tangle network.

## 📜 License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 📬 Feedback and Contributions

We welcome feedback and contributions to improve this blueprint.
Please open an issue or submit a pull request on
our [GitHub repository](https://github.com/tangle-network/blueprint-template/issues).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
