#![no_main]
sp1_zkvm::entrypoint!(main);

use fibonacci_lib::verify_tx_in_block_and_outputs;

pub fn main() {
    // Read inputs from SP1 stdin
    let tx_hex = sp1_zkvm::io::read::<String>();
    let expected_txid = sp1_zkvm::io::read::<String>();
    let merkle_siblings: Vec<String> = sp1_zkvm::io::read::<Vec<String>>();
    let pos = sp1_zkvm::io::read::<usize>();
    let block_header = sp1_zkvm::io::read::<String>();
    let target_address = sp1_zkvm::io::read::<String>();

    // Verify transaction in block and sum outputs to target address
    let result = verify_tx_in_block_and_outputs(
        &tx_hex,
        &expected_txid,
        merkle_siblings,
        pos,
        &block_header,
        &target_address,
    );

    // Verification must pass
    let (block_hash, total_amount) = result.expect("Transaction verification failed");

    // Commit the results to SP1 output
    sp1_zkvm::io::commit(&block_hash);
    sp1_zkvm::io::commit(&total_amount);
}
