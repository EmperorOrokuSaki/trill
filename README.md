# Trill

Trill is a TUI memory profiler for EVM chains with a focus on efficiency. It is written in Rust and envisioned to be an open-source tool used by programmers who are interested in observing how transactions modify a smart contract's dynamic storage.

## Introduction

Memory profiling is crucial for understanding and optimizing the performance of smart contracts on EVM chains. Trill provides developers with a powerful tool to visualize and analyze memory usage during transaction execution, helping them identify potential bottlenecks and improve contract efficiency.

## Installation

Prior to installing Trill, ensure that you have the following dependencies installed:

- [Rust](https://www.rust-lang.org/tools/install): Trill is written in Rust.
- [Foundry](https://book.getfoundry.sh/getting-started/installation): Trill uses the Anvil local environment for simulating transaction trace calls.

### Manual Installation

```sh
$ git clone https://github.com/EmperorOrokuSaki/trill.git
$ cd trill
$ cargo install --path .
```

## Usage
### Custom transactions

To use Trill with custom transactions in the local Anvil environment, follow these steps:

1. Start Anvil with tracing enabled:

```$ anvil --port 8545 --steps-tracing```

2. Deploy your contract and submit your transaction.

3. Use the transaction hash to start Trill:

```$ cargo run -- --transaction <TX_HASH>```

### Launch Script
The [launch.sh](./launch.sh) shell script demonstrates how to set up the Anvil environment and use Trill for profiling a smart contract's memory propagation during a transaction:

`$ chmod +x launch.sh && ./launch.sh`

## Supported Opcodes
Trill supports the following opcodes for memory profiling:

- MSTORE
- MSTORE8
- MLOAD
- CALLDATACOPY
- MSIZE
- EXTCODECOPY
- CODECOPY
- RETURNDATACOPY
- MCOPY

### Opcode Argument Parsing
Trill uses the [opcode-parser](https://github.com/EmperorOrokuSaki/opcode-parser) crate to parse the arguments for the supported opcodes.

## Troubleshooting
If you encounter any issues during installation or usage, please open an issue on this repository.

## Contributing
This version of Trill can potentially (and most likely) have many bugs. All community contributions are welcome, even if they are simple documentation improvements.

Additionally, if you are interested in contributing to the code of the project, please check [these](https://github.com/EmperorOrokuSaki/trill/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22+label%3A%22help+wanted%22+) issues.

## Contact
For questions, feedback, or collaboration opportunities, please start a [discussion](https://github.com/EmperorOrokuSaki/trill/discussions) or [issue](https://github.com/EmperorOrokuSaki/trill/issues) on GitHub.

