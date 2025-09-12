use alloy_sol_types::sol;
use sha2::{Digest, Sha256};

sol! {
    struct PublicValuesStruct {
        bool valid;
    }
}

/// Double SHA-256 hash function used by Bitcoin
fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    second.into()
}

/// Convert hash bytes to txid format (little-endian for display)
fn to_txid(hash: [u8; 32]) -> [u8; 32] {
    let mut rev = hash;
    rev.reverse();
    rev
}

/// Verify that the computed transaction hash matches the expected txid
pub fn verify_bitcoin_tx_hash(tx_hash: &str, tx_hex: &str) -> bool {
    // Decode the expected txid (in little-endian format)
    let expected_txid = match hex::decode(tx_hash) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Decode the raw transaction
    let tx = match hex::decode(tx_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Compute double SHA-256 of the transaction
    let hash = sha256d(&tx);

    // Convert to txid format (little-endian) for comparison
    let computed_txid = to_txid(hash);

    computed_txid.to_vec() == expected_txid
}

/// Verify Merkle inclusion proof
/// Note: This function expects the txid in the internal hash format (big-endian)
pub fn verify_merkle_proof(
    tx_hash: [u8; 32],
    merkle: &[[u8; 32]],
    mut pos: i32,
    merkle_root: [u8; 32],
) -> bool {
    let mut current_hash = hex_rev32(hex::encode(tx_hash).as_str());

    // Traverse up the Merkle tree
    for sibling in merkle {
        let mut combined = Vec::with_capacity(64);

        if pos % 2 == 0 {
            // Current hash is on the left
            combined.extend_from_slice(&current_hash);
            combined.extend_from_slice(sibling);
        } else {
            // Current hash is on the right
            combined.extend_from_slice(sibling);
            combined.extend_from_slice(&current_hash);
        }

        current_hash = sha256d(&combined);
        pos >>= 1; // Move up one level in the tree
    }
    // Compare with the expected Merkle root
    current_hash == merkle_root
}

fn hex_rev32(s: &str) -> [u8; 32] {
    let mut bytes: [u8; 32] = hex::decode(s).unwrap().as_slice().try_into().unwrap();
    bytes.reverse(); // flip from RPC little-endian to internal big-endian
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_rev32(s: &str) -> [u8; 32] {
        let mut bytes: [u8; 32] = hex::decode(s).unwrap().as_slice().try_into().unwrap();
        bytes.reverse(); // flip from RPC little-endian to internal big-endian
        bytes
    }

    #[test]
    fn test_sha256d() {
        let data = b"hello";
        let result = sha256d(data);
        assert_eq!(result.len(), 32);

        // Test with known Bitcoin transaction
        let tx_hex = "010000000536a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0c0000006b483045022100bcdf40fb3b5ebfa2c158ac8d1a41c03eb3dba4e180b00e81836bafd56d946efd022005cc40e35022b614275c1e485c409599667cbd41f6e5d78f421cb260a020a24f01210255ea3f53ce3ed1ad2c08dfc23b211b15b852afb819492a9a0f3f99e5747cb5f0ffffffffee08cb90c4e84dd7952b2cfad81ed3b088f5b32183da2894c969f6aa7ec98405020000006a47304402206332beadf5302281f88502a53cc4dd492689057f2f2f0f82476c1b5cd107c14a02207f49abc24fc9d94270f53a4fb8a8fbebf872f85fff330b72ca91e06d160dcda50121027943329cc801a8924789dc3c561d89cf234082685cbda90f398efa94f94340f2ffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f060000006b4830450221009c97a25ae70e208b25306cc870686c1f0c238100e9100aa2599b3cd1c010d8ff0220545b34c80ed60efcfbd18a7a22f00b5f0f04cfe58ca30f21023b873a959f1bd3012102e54cd4a05fe29be75ad539a80e7a5608a15dffbfca41bec13f6bf4a32d92e2f4ffffffff73cabea6245426bf263e7ec469a868e2e12a83345e8d2a5b0822bc7f43853956050000006b483045022100b934aa0f5cf67f284eebdf4faa2072345c2e448b758184cee38b7f3430129df302200dffac9863e03e08665f3fcf9683db0000b44bf1e308721eb40d76b180a457ce012103634b52718e4ddf125f3e66e5a3cd083765820769fd7824fd6aa38eded48cd77fffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0b0000006a47304402206348e277f65b0d23d8598944cc203a477ba1131185187493d164698a2b13098a02200caaeb6d3847b32568fd58149529ef63f0902e7d9c9b4cc5f9422319a8beecd50121025af6ba0ccd2b7ac96af36272ae33fa6c793aa69959c97989f5fa397eb8d13e69ffffffff0400e6e849000000001976a91472d52e2f5b88174c35ee29844cce0d6d24b921ef88ac20aaa72e000000001976a914c15b731d0116ef8192f240d4397a8cdbce5fe8bc88acf02cfa51000000001976a914c7ee32e6945d7de5a4541dd2580927128c11517488acf012e39b000000001976a9140a59837ccd4df25adc31cdad39be6a8d97557ed688ac00000000";
        let tx_bytes = hex::decode(tx_hex).unwrap();
        let hash = sha256d(&tx_bytes);

        // The expected hash should be the txid in big-endian format
        let expected_hash =
            hex::decode("15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521")
                .unwrap();
        let mut expected_array = [0u8; 32];
        expected_array.copy_from_slice(&expected_hash);
        expected_array.reverse(); // Convert from little-endian txid to big-endian hash

        assert_eq!(hash, expected_array);
    }

    #[test]
    fn test_to_txid() {
        let hash = [1u8; 32];
        let txid = to_txid(hash);
        assert_eq!(txid[0], 1);
        assert_eq!(txid[31], 1);

        // Test with actual Bitcoin transaction hash
        let tx_hex = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f5c551a4dd2414c459dd2c6688b7ba0d0618e1253a0520ac000000000000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac0000000000000000";
        let tx_bytes = hex::decode(tx_hex).unwrap();
        let hash = sha256d(&tx_bytes);
        let txid = to_txid(hash);

        // Expected txid: f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16
        let expected_txid =
            hex::decode("f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16")
                .unwrap();
        assert_eq!(txid.to_vec(), expected_txid);
    }

    #[test]
    fn test_verify_bitcoin_tx_hash() {
        // This is the first Bitcoin transaction ever made (Genesis transaction)
        let tx_hex = "010000000536a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0c0000006b483045022100bcdf40fb3b5ebfa2c158ac8d1a41c03eb3dba4e180b00e81836bafd56d946efd022005cc40e35022b614275c1e485c409599667cbd41f6e5d78f421cb260a020a24f01210255ea3f53ce3ed1ad2c08dfc23b211b15b852afb819492a9a0f3f99e5747cb5f0ffffffffee08cb90c4e84dd7952b2cfad81ed3b088f5b32183da2894c969f6aa7ec98405020000006a47304402206332beadf5302281f88502a53cc4dd492689057f2f2f0f82476c1b5cd107c14a02207f49abc24fc9d94270f53a4fb8a8fbebf872f85fff330b72ca91e06d160dcda50121027943329cc801a8924789dc3c561d89cf234082685cbda90f398efa94f94340f2ffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f060000006b4830450221009c97a25ae70e208b25306cc870686c1f0c238100e9100aa2599b3cd1c010d8ff0220545b34c80ed60efcfbd18a7a22f00b5f0f04cfe58ca30f21023b873a959f1bd3012102e54cd4a05fe29be75ad539a80e7a5608a15dffbfca41bec13f6bf4a32d92e2f4ffffffff73cabea6245426bf263e7ec469a868e2e12a83345e8d2a5b0822bc7f43853956050000006b483045022100b934aa0f5cf67f284eebdf4faa2072345c2e448b758184cee38b7f3430129df302200dffac9863e03e08665f3fcf9683db0000b44bf1e308721eb40d76b180a457ce012103634b52718e4ddf125f3e66e5a3cd083765820769fd7824fd6aa38eded48cd77fffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0b0000006a47304402206348e277f65b0d23d8598944cc203a477ba1131185187493d164698a2b13098a02200caaeb6d3847b32568fd58149529ef63f0902e7d9c9b4cc5f9422319a8beecd50121025af6ba0ccd2b7ac96af36272ae33fa6c793aa69959c97989f5fa397eb8d13e69ffffffff0400e6e849000000001976a91472d52e2f5b88174c35ee29844cce0d6d24b921ef88ac20aaa72e000000001976a914c15b731d0116ef8192f240d4397a8cdbce5fe8bc88acf02cfa51000000001976a914c7ee32e6945d7de5a4541dd2580927128c11517488acf012e39b000000001976a9140a59837ccd4df25adc31cdad39be6a8d97557ed688ac00000000";
        let tx_hash = "15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521";

        let result = verify_bitcoin_tx_hash(tx_hash, tx_hex);
        assert!(result, "Should verify the first Bitcoin transaction");
    }

    #[test]
    fn test_verify_merkle_proof() {
        // txid from explorer → convert to internal big-endian
        let tx_hash = hex_rev32("15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521");

        // merkle siblings from explorer → convert each to internal big-endian
        let merkle_raw = vec![
            "acf931fe8980c6165b32fe7a8d25f779af7870a638599db1977d5309e24d2478",
            "ee25997c2520236892c6a67402650e6b721899869dcf6715294e98c0b45623f9",
            "790889ac7c0f7727715a7c1f1e8b05b407c4be3bd304f88c8b5b05ed4c0c24b7",
            "facfd99cc4cfe45e66601b37a9637e17fb2a69947b1f8dc3118ed7a50ba7c901",
            "8c871dd0b7915a114f274c354d8b6c12c689b99851edc55d29811449a6792ab7",
            "eb4d9605966b26cfa3bf69b1afebe375d3d6aadaa7f2899d48899b6bd2fd6a43",
            "daa1dc59f22a8601b489fc8a89da78bc35415291c62c185e711b8eef341e6e70",
            "102907c1b95874e2893c6f7f06b45a3d52455d3bb17796e761df75aeda6aa065",
            "baeede9b8e022bb98b63cb765ba5ca3e66e414bfd37702b349a04113bcfcaba6",
            "b6f07be94b55144588b33ff39fb8a08004baa03eb7ff121e1847d715d0da6590",
            "7d02c62697d783d85a51cd4f37a87987b8b3077df4ddd1227b254f59175ed1e4",
        ];
        let merkle_arr: Vec<[u8; 32]> = merkle_raw.into_iter().map(hex_rev32).collect();

        let pos = 1465;

        // block header merkle root (explorer hex) → convert
        let merkle_root =
            hex_rev32("d02f9ae95b1ed06a126ff60e667db491a8eba70d024a0942b7147451a82f0cef");

        let result = verify_merkle_proof(tx_hash, &merkle_arr, pos, merkle_root);
        assert!(result, "Should verify the Merkle proof");
    }
}
