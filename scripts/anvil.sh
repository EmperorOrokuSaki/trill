#!/bin/bash
set -eo pipefail

anvil --port 8545 --steps-tracing --block-base-fee-per-gas 0 &
pid=$!
sleep 0.1

run() {
    acc=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    #code=0x608060405234801561000f575f80fd5b506102648061001d5f395ff3fe608060405234801561000f575f80fd5b506004361061003f575f3560e01c806325427cd414610043578063d39fa23314610058578063e5691f5e1461007d575b5f80fd5b610056610051366004610171565b610090565b005b61006b6100663660046101e0565b6100e6565b60405190815260200160405180910390f35b61005661008b366004610171565b610104565b6040516372b48faf60e11b8152309063e5691f5e906100b590859085906004016101f7565b5f604051808303815f87803b1580156100cc575f80fd5b505af11580156100de573d5f803e3d5ffd5b505050505050565b5f81815481106100f4575f80fd5b5f91825260209091200154905081565b61010f5f8383610114565b505050565b828054828255905f5260205f2090810192821561014d579160200282015b8281111561014d578235825591602001919060010190610132565b5061015992915061015d565b5090565b5b80821115610159575f815560010161015e565b5f8060208385031215610182575f80fd5b823567ffffffffffffffff80821115610199575f80fd5b818501915085601f8301126101ac575f80fd5b8135818111156101ba575f80fd5b8660208260051b85010111156101ce575f80fd5b60209290920196919550909350505050565b5f602082840312156101f0575f80fd5b5035919050565b602080825281018290525f6001600160fb1b03831115610215575f80fd5b8260051b8085604085013791909101604001939250505056fea26469706673582212206e660ae6395432ad4503646962c0f1debd499eca771fdc0518b62ef27f0fcbde64736f6c63430008180033
    code=6080604052348015600e575f80fd5b506101588061001c5f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c80636537214714610038578063b74413ea14610056575b5f80fd5b610040610060565b60405161004d9190610109565b60405180910390f35b61005e610065565b005b5f5481565b608051806101005260ff610110535961012051806101305260116101405360226101415360336101425360446101435360886102475361015051610160518160705280610180528161019052806102a052303b605560445360666101455360776112c053805f80303c836102a552365f6102b0373d5f6102d03e836102e0526102e05150505050505050565b5f819050919050565b610103816100f1565b82525050565b5f60208201905061011c5f8301846100fa565b9291505056fea2646970667358221220c380c45d7330268c2b1d0e42eafe937a5468dae2872e1fd9b700c0e82235c9e164736f6c63430008190033
    # data=""
    # for i in $(seq 1 1000); do
    #     data+="$i,"
    # done

    contract=$(cast send --json --unlocked --from $acc --create $code | jq -r '.contractAddress')
    hash=$(cast send --json --unlocked --from $acc "$contract" "memoryOperations()" | jq -r '.transactionHash')
    echo $hash
    ../trill --transaction $hash
    # cast rpc debug_traceTransaction "$hash" '{"enableMemory":true, "disableStack": true}' | jq > resp.json
}

run || true

