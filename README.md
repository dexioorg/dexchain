# Dexchain

An Ethereum compatible Parachain built with Substrate.

### Rust Setup

First, complete the [basic Rust setup instructions](./docs/rust-setup.md).

### Makefile

This project uses a [Makefile](Makefile) to document helpful commands and make it easier to execute
them. Get started by running these [`make`](https://www.gnu.org/software/make/manual/make.html)
targets:

1. `make init` - Run the [init script](scripts/init.sh) to configure the Rust toolchain for
   [WebAssembly compilation](https://substrate.dev/docs/en/knowledgebase/getting-started/#webassembly-compilation).
1. `make run` - Build and launch this project in development mode.

The init script and Makefile both specify the version of the
[Rust nightly compiler](https://substrate.dev/docs/en/knowledgebase/getting-started/#rust-nightly-toolchain)
that this project depends on.

### Build

The `make run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
make build
```

> If this error message appears when compiling the source code: 
>
>  Rust WASM toolchain not installed, please install it!

```sh

# Plesae install toolchain nightly
rustup toolchain install nightly-2021-01-13
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-01-13

```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/dexchain -h
```


### Connect to Testnet

```bash

./target/release/dexchain \
  --base-path ./.testnet/ \
  --chain ./specs/testnet.json \
  --pruning archive \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --ws-external \
  --rpc-cors all \
  --rpc-external \
  --name MyNode

```

Let's look at those flags in detail:

| Flags             | Descriptions                                                                                                  |
|-------------------|---------------------------------------------------------------------------------------------------------------|
| `--base-path`     | Specifies a directory where Dexchain should store all the data related to this chain.                         |
| `--chain`         | Specifies which chain specification to use. `testnet.json` means the node will connect to testnet.            |
| `--pruning`       | Specify the state pruning mode, a number of blocks to keep or 'archive'.                                      |
| `--port`          | Specifies the port that your node will listen for p2p traffic on. `30333` is the default.                     |
| `--ws-port`       | Specifies the port that your node will listen for incoming WebSocket traffic on. The default value is `9944`. |
| `--rpc-port`      | Specifies the port that your node will listen for incoming RPC traffic on. `9933` is the default.             |
| `--telemetry-url` | Tells the node to send telemetry data to a particular server.                                                 |
| `--ws-external`   | Listen to all Websocket interfaces.                                                                           |
| `--rpc-cors`      | Specify browser Origins allowed to access the HTTP & WS RPC servers.                                          |
| `--rpc-external`  | Listen to all RPC interfaces.                                                                                 |
| `--name`          | The human-readable name for this node.                                                                        |


```sh

# Show the synchronization status of the current node
curl http://{IP:Port} -H "Content-Type:application/json" -X POST --data '{"jsonrpc":"2.0","id":1,"method":"system_syncState", "params": []}'

```

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/dexchain --dev
```

Purge the development chain's state:

```bash
./target/release/dexchain purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/dexchain -lruntime=debug --dev
```

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

## Genesis Configuration

The development [chain spec](node/src/chain_spec.rs) included with this project defines a genesis block that has been pre-configured with an EVM account for [Alice](https://substrate.dev/docs/en/knowledgebase/integrate/subkey#well-known-keys). When [a development chain is started](https://github.com/substrate-developer-hub/substrate-node-template#run), Alice's EVM account will be funded with a large amount of Ether.
The [Polkadot UI](https://polkadot.js.org/apps/#?rpc=ws://127.0.0.1:9944) can be used to see the details of Alice's EVM account.
In order to view an EVM account, use the `Developer` tab of the Polkadot UI `Settings` app to define the EVM `Account` type as below.
It is also necessary to define the `Address` and `LookupSource` to send transaction, and `Transaction` and `Signature` to be able to inspect blocks:

```json

{
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "EvmAddress": "H160",
  "BlockNumber": "u32",
  "BalanceOf": "u128",
  "Account": {
    "nonce": "U256",
    "balance": "U256"
  },
  "Signature": {
    "v": "u64",
    "r": "H256",
    "s": "H256"
  },
  "TransactionAction": {
    "_enum": {
      "Call": "H160",
      "Create": null
    }
  },
  "Transaction": {
    "nonce": "U256",
    "action": "TransactionAction",
    "gas_price": "U256",
    "gas_limit": "U256",
    "value": "U256",
    "input": "Vec<u8>",
    "signature": "Signature"
  },
  "ReturnValue": {
    "_enum": {
      "Bytes": "Vec<u8>",
      "Hash": "H160"
    }
  },
  "EthereumStorageSchema": {
    "_enum": [
      "Undefined",
      "V1"
    ]
  },
  "GenesisAccount": {
    "nonce": "U256",
    "balance": "U256",
    "storage": "BTreeMap<H256, H256>",
    "code": "Vec<u8>"
  }
}

```

## License

[GPL-3.0-or-later](./LICENSE)
