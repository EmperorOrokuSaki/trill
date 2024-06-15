# Trill

![Trill Banner](./TrillBanner.png)

Trill is a TUI memory profiler for EVM smart contracts with a focus on efficiency. It is written in Rust and envisioned to be an open-source tool used by programmers who are interested in observing how transactions modify a smart contract's dynamic storage.

## Introduction

Memory profiling is crucial for understanding and optimizing the performance of smart contracts on EVM chains. Trill provides developers with a powerful tool to visualize and analyze memory usage during transaction execution, helping them identify potential bottlenecks and improve contract efficiency.

If you are interested in the story of how Trill was written, check [this blog post](https://0xnimara.substack.com/p/building-trill). You can also check [feature idea discussions](https://github.com/EmperorOrokuSaki/trill/discussions) on the repository to contribute to Trill's future!

> [!NOTE]
> This project is currently undergoing tests and could potentially have issues. If you find any problem while building or running it, please open an issue.

## Installation
![Trill running in terminal](./assets/trill.gif)

> [!IMPORTANT]
> Trill has not been tested on MacOS. If you encounter any issues with the MacOS build, please share the details in this [issue](https://github.com/EmperorOrokuSaki/trill/issues/2)

Prior to installing Trill, ensure that you have the following dependencies installed:

- [Rust](https://www.rust-lang.org/tools/install): Trill is written in Rust.
- [Foundry](https://book.getfoundry.sh/getting-started/installation): Trill uses the Anvil local environment by default for simulating transaction trace calls. If you wish to use a RPC endpoint that supports `debug_traceTransaction` calls, you may not need this dependency.

### Manual Installation

```sh
$ git clone https://github.com/EmperorOrokuSaki/trill.git
$ cd trill
$ cargo install --path .
```

## Usage
### Commands
```
A TUI memory profiler tool for EVM smart contracts

Usage: trill [OPTIONS] --transaction <TRANSACTION>

Options:
  -t, --transaction <TRANSACTION>  Transaction hash
  -f, --fps <FPS>                  Frames per second [default: 4]
  -i, --iteration <ITERATION>      Operations to process with each frame [default: 1]
  -r, --rpc <RPC>                  The JSON-RPC endpoint URL [default: http://127.0.0.1:8545]
  -h, --help                       Print help
  -V, --version                    Print version
```

### Custom transactions

You can use the following command. Please, make sure your RPC supports `debug_traceTransaction` calls:

```$ trill --transaction <TX_HASH> --rpc <RPC_URL>```

#### Anvil

To use Trill with custom transactions in the local Anvil environment, follow these steps:

1. Start Anvil with tracing enabled:

```$ anvil --port 8545 --steps-tracing```

2. Deploy your contract and submit your transaction.

3. Use the transaction hash to start Trill:

```$ trill --transaction <TX_HASH>```

### Launch Script
The [launch.sh](./launch.sh) shell script demonstrates how to set up the Anvil environment and use Trill for profiling a smart contract's memory propagation during a transaction:

`$ chmod +x launch.sh && ./launch.sh`

### Test Scripts
The [scripts](./scripts) directory contains several shell scripts that deploy and interact with different well-known smart contracts.

### Opcode Argument Parsing
Trill uses the [opcode-parser](https://github.com/EmperorOrokuSaki/opcode-parser) crate to parse the arguments for the supported opcodes.

## Troubleshooting
If you encounter any issues during installation or usage, please open an issue on this repository.

## Contributing
This version of Trill can potentially (and most likely) have many bugs. All community contributions are welcome, even if they are simple documentation improvements.

Additionally, if you are interested in contributing to the code of the project, please check [these](https://github.com/EmperorOrokuSaki/trill/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22+label%3A%22help+wanted%22+) issues.

## Acknowledgements
Trill was inspired by this [tweet](https://twitter.com/0xkarmacoma/status/1773385937323786662) from [karma](https://twitter.com/0xkarmacoma)!

## Contact
For questions, feedback, or collaboration opportunities, please start a [discussion](https://github.com/EmperorOrokuSaki/trill/discussions) or [issue](https://github.com/EmperorOrokuSaki/trill/issues) on GitHub.

