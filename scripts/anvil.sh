#!/bin/bash
set -a # automatically export all variables
source ../.env
set +a
anvil --steps-tracing &
ANVIL_PID=$!
sleep 2
forge script test.s.sol:MyScript --fork-url http://localhost:8545 --broadcast
cast send 0x5FbDB2315678afecb367f032d93F642f64180aa3 "mint(uint128)" 100 --rpc-url http://127.0.0.1:8545  --private-key $ANVIL_PK
json='{"config":{"enable_memory":true,"disable_memory":null,"disable_stack":false,"disable_storage":null,"enable_return_data":null,"disable_return_data":null,"debug":null,"limit":null},"tracer":null,"tracer_config":{},"timeout":null}'
command="cast rpc debug_traceTransaction 0xa486ce0029f1b7df9847a9832108b0e4063fea00b332ab994c384785cd0d8ac4 $json --rpc-url http://127.0.0.1:8545"
$command > resp.json
echo $command
kill $ANVIL_PID

