# EVM compatible Solochain

Base on [Frontier](https://github.com/polkadot-evm/frontier)

## Getting Started

Supported OS:

- Linux
- MacOS
- WSL2 on Windows

Install Rust:
- Method 1: through [rust installation doc](https://www.rust-lang.org/tools/install)
- Method 2: uncomment `rustup` in `flake.nix` file
    
Install Nix:

```sh
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Then enter to development environment from the root of the project folder (where flake.nix exist):

```sh
nix develop
```

### Build

Use the following command to build the node without launching it:

```sh
cargo build --release
```

### Embedded Docs

After you build the project, you can use the following command to explore its
parameters and subcommands:

```sh
./target/release/solochain-template-node -h
```

You can generate and view the [Rust
Docs](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) for this template
with this command:

```sh
cargo doc --open
```

### Single-Node Development Chain

The following command starts a single-node development chain that doesn't
persist state:

```sh
./target/release/solochain-template-node --dev
```

To start the development chain with detailed logging, run the following command:

```sh
RUST_BACKTRACE=1 ./target/release/solochain-template-node -ldebug --dev
```

Development chains:

- Maintain state in a `tmp` folder while the node is running.
- Use the **Alice** and **Bob** (**Bob** is just in test-net, not dev-net) accounts as default validator authorities.
- Use the **Alice** account as the default `sudo` account.
- Are preconfigured with a genesis state (`/node/src/chain_spec.rs`) that
  includes several pre-funded development accounts.


To persist chain state between runs, specify a base path by running a command
similar to the following:

```sh
// Create a folder to use as the db base path
$ mkdir my-chain-state

// Use of that folder to store the chain state
$ ./target/release/solochain-template-node --dev --base-path ./my-chain-state/

// Check the folder structure created inside the base path after running the chain
$ ls ./my-chain-state
chains
$ ls ./my-chain-state/chains/
dev
$ ls ./my-chain-state/chains/dev
db keystore network
```

### Pre-defined accounts
 
Pre-founded with native token

```yaml
Alice:
    pubkey:     0x1f31d7740f9b822edd0ea965f4cfcf1034c450a2
Bob:
    pubkey:     0x8347187707fea1e2418c8534998d9b8d26cd6430
AliceStash:
    pubkey:     0x81593023a7f69346a047421099a8085ed0f44f7e
BobStash:
    pubkey:     0xe1cb409bc4e2002e12875a1b0c528994fe3ef23e
unnamed1:
    pubkey:     0xAE4F6dFcdB369A6C43565b1658Ac759960a60aCb
    privkey:    0x1650222a42e77da936f8c183458814cd144e7648d683b0ab7321bd2032030c08
unnamed2:
    pubkey:     0x4dE6268744b93Ed85b4e2CF683686f4A6086293a
    privkey:    0x096e09370e3b22537c854ffc517514ad7f31639b377488a252868b4276d35826
```

### POA authority accounts

Dev & Test net accounts:

|           |Aura Id                                         |Grandpa Id                                      |
|-----------|------------------------------------------------|------------------------------------------------|
|Alice      |5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY|5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu|
|Bob        |5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty|5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E|

They are integrated with `--alice` and `--bob` switches for testing purpose.

Custom spec accounts: are in the `auth_accounts_example.yml`

### Connect with Polkadot-JS Apps Front-End

After you start the node template locally, you can interact with it using the
hosted version of the [Polkadot/Substrate
Portal](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944)
front-end by connecting to the local node endpoint. A hosted version is also
available on [IPFS](https://dotapps.io/). You can
also find the source code and instructions for hosting your own instance in the
[`polkadot-js/apps`](https://github.com/polkadot-js/apps) repository.

### Run multi-node local testnet

- run alice node: `. ./scripts/local_testnet/run_alice_node.sh`
- run bob node in another teminal: `. ./scripts/local_testnet/run_bob_node.sh`

Reference: [Simulate a network](https://docs.substrate.io/tutorials/build-a-blockchain/simulate-network/).

### Run multi-node with custom spec

- generate keys
    - generate sr25519: `. ./scripts/utils/generate_sr25519_key.sh <a password>` 
    - generate ed25519: `. ./scripts/utils/derive_ed25519_key.sh <previous ss58 address> <previous password>`
    - alternatively you can use `auth_accounts_example.yml` for testing purpose
- customize `customeSpec.json` file
- run `. ./scripts/utils/spec_to_raw.sh` to generate `customSpecRaw.json`
- change `./scripts/custom_chain/node.sh` according to generated keys
- add keys to keystore by running `./scripts/custom_chain/add_keys_to_keystore.sh`
- run node
    - if it is first running node of the blockchain, run `./scripts/custom_chain/run_first_node.sh`
    - otherwise run `./scripts/custom_chain/run_participant_node.sh`

Reference: [Multi-node with custom spec](https://docs.substrate.io/tutorials/build-a-blockchain/add-trusted-nodes/)

### Definitions

#### Node

A blockchain node is an application that allows users to participate in a
blockchain network. Substrate-based blockchain nodes expose a number of
capabilities:

- Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking
  stack to allow the nodes in the network to communicate with one another.
- Consensus: Blockchains must have a way to come to
  [consensus](https://docs.substrate.io/fundamentals/consensus/) on the state of
  the network. Substrate makes it possible to supply custom consensus engines
  and also ships with several consensus mechanisms that have been built on top
  of [Web3 Foundation
  research](https://research.web3.foundation/Polkadot/protocols/NPoS).
- RPC Server: A remote procedure call (RPC) server is used to interact with
  Substrate nodes.

There are several files in the `node` directory. Take special note of the
following:

- [`chain_spec.rs`](./node/src/chain_spec.rs): A [chain
  specification](https://docs.substrate.io/build/chain-spec/) is a source code
  file that defines a Substrate chain's initial (genesis) state. Chain
  specifications are useful for development and testing, and critical when
  architecting the launch of a production chain. Take note of the
  `development_config` and `testnet_genesis` functions. These functions are
  used to define the genesis state for the local development chain
  configuration. These functions identify some [well-known
  accounts](https://docs.substrate.io/reference/command-line-tools/subkey/) and
  use them to configure the blockchain's initial state.
- [`service.rs`](./node/src/service.rs): This file defines the node
  implementation. Take note of the libraries that this file imports and the
  names of the functions it invokes. In particular, there are references to
  consensus-related topics, such as the [block finalization and
  forks](https://docs.substrate.io/fundamentals/consensus/#finalization-and-forks)
  and other [consensus
  mechanisms](https://docs.substrate.io/fundamentals/consensus/#default-consensus-models)
  such as Aura for block authoring and GRANDPA for finality.


#### Runtime

In Substrate, the terms "runtime" and "state transition function" are analogous.
Both terms refer to the core logic of the blockchain that is responsible for
validating blocks and executing the state changes they define. The Substrate
project in this repository uses
[FRAME](https://docs.substrate.io/learn/runtime-development/#frame) to construct
a blockchain runtime. FRAME allows runtime developers to declare domain-specific
logic in modules called "pallets". At the heart of FRAME is a helpful [macro
language](https://docs.substrate.io/reference/frame-macros/) that makes it easy
to create pallets and flexibly compose them to create blockchains that can
address [a variety of needs](https://substrate.io/ecosystem/projects/).

Review the [FRAME runtime implementation](./runtime/src/lib.rs) included in this
template and note the following:

- This file configures several pallets to include in the runtime. Each pallet
  configuration is defined by a code block that begins with `impl
  $PALLET_NAME::Config for Runtime`.
- The pallets are composed into a single runtime by way of the
  [`construct_runtime!`](https://paritytech.github.io/substrate/master/frame_support/macro.construct_runtime.html)
  macro, which is part of the [core FRAME pallet
  library](https://docs.substrate.io/reference/frame-pallets/#system-pallets).

#### Pallets

The runtime in this project is constructed using many FRAME pallets that ship
with [the Substrate
repository](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame) and a
template pallet that is [defined in the
`pallets`](./pallets/template/src/lib.rs) directory.

A FRAME pallet is comprised of a number of blockchain primitives, including:

- Storage: FRAME defines a rich set of powerful [storage
  abstractions](https://docs.substrate.io/build/runtime-storage/) that makes it
  easy to use Substrate's efficient key-value database to manage the evolving
  state of a blockchain.
- Dispatchables: FRAME pallets define special types of functions that can be
  invoked (dispatched) from outside of the runtime in order to update its state.
- Events: Substrate uses
  [events](https://docs.substrate.io/build/events-and-errors/) to notify users
  of significant state changes.
- Errors: When a dispatchable fails, it returns an error.

Each pallet has its own `Config` trait which serves as a configuration interface
to generically define the types and parameters it depends on.
