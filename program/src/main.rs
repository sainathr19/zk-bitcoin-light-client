#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use fibonacci_lib::{verify_bitcoin_tx_hash, verify_merkle_proof, PublicValuesStruct};

pub fn main() {
    // Read inputs from SP1 stdin
    let tx_hash = sp1_zkvm::io::read::<String>();
    let tx = sp1_zkvm::io::read::<String>();
    let merkle: Vec<[u8; 32]> = sp1_zkvm::io::read::<Vec<[u8; 32]>>();
    let pos = sp1_zkvm::io::read::<i32>();
    let merkle_root = sp1_zkvm::io::read::<[u8; 32]>();

    // Verify that the transaction hash is correct
    let hash_valid = verify_bitcoin_tx_hash(&tx_hash, &tx);

    // Verify Merkle inclusion proof
    let merkle_valid = verify_merkle_proof(
        hex::decode(tx_hash).unwrap().as_slice().try_into().unwrap(),
        &merkle,
        pos,
        merkle_root,
    );

    // Both verifications must pass
    let overall_valid = hash_valid && merkle_valid;

    // Encode the result
    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct {
        valid: overall_valid,
    });

    // Commit the result to SP1 output
    sp1_zkvm::io::commit_slice(&bytes);
}
