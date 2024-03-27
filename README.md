# Trill

Trill is a TUI memory profiler for EVM chains with a focus on efficiency. It is written in Rust and envisioned to be an open-source tool used by programmers who are interested in observing how transactions modify a smart contract's dynamic storage.

## Installation

Prior to installing the application please make sure that you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install): Trill is written in the Rust programming language and the current version does not include an executable.
- [Foundry](https://book.getfoundry.sh/getting-started/installation): The current version of Trill uses the Anvil local environment for simulating transaction trace calls.

### Custom transactions

You can use Trill with any transaction made in the local Anvil environment. However, during the initialization of Anvil it is required to include the `--steps-tracing` flag to make trace RPC calls possible:

`$ anvil --port 8545 --steps-tracing`

After submission of the transaction, you can use the transaction hash to start Trill:

`$ cargo run -- --transaction <TX_HASH>`

### Launch script

The [launch.sh](./launch.sh) shell script is intended to be used as an example of how the Anvil environment can be set-up and Trill be used for profiling the memory propagation of a smart contract during a transaction. You can run this script by executing the commands below:

`$ chmod +x launch.sh && ./launch.sh`

## Supported opcodes

All supported opcodes are listed below. If a certain opcode is missing, please open an issue about it.

- MSTORE
- MSTORE8
- MLOAD
- CALLDATACOPY
- MSIZE
- EXTCODECOPY
- CODECOPY
- RETURNDATACOPY
- MCOPY
