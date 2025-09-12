use bech32::{convert_bits, decode, u5, Variant};
use sha2::{Digest, Sha256};

/// Double SHA-256
fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    second.into()
}

/// Compute raw internal tx hash (big-endian) by double-sha256 over tx bytes
fn compute_raw_tx_hash_from_txhex(tx_hex: &str) -> Result<[u8; 32], String> {
    let tx_bytes = hex::decode(tx_hex).map_err(|e| format!("tx hex decode: {}", e))?;
    Ok(sha256d(&tx_bytes))
}

/// Verify expected explorer txid (little-endian hex) matches computed tx hash
fn verify_txid(expected_txid_hex: &str, tx_hex: &str) -> Result<bool, String> {
    let expected_bytes =
        hex::decode(expected_txid_hex).map_err(|e| format!("expected txid hex decode: {}", e))?;
    if expected_bytes.len() != 32 {
        return Err("expected txid len != 32".to_string());
    }
    let mut expected_arr: [u8; 32] = expected_bytes.as_slice().try_into().unwrap();
    // explorer txid is little-endian display, convert to internal (big-endian)
    expected_arr.reverse();

    let computed = compute_raw_tx_hash_from_txhex(tx_hex)?;
    Ok(computed == expected_arr)
}

/// Convert a hex sibling (explorer display) -> internal big-endian [u8;32]
fn hex_sibling_to_internal(s: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(s).map_err(|e| format!("hex decode sibling: {}", e))?;
    if bytes.len() != 32 {
        return Err("sibling len != 32".into());
    }
    let mut arr: [u8; 32] = bytes.as_slice().try_into().unwrap();
    // explorer gives little-endian display; convert to internal big-endian
    arr.reverse();
    Ok(arr)
}

/// Verify merkle inclusion
/// - `leaf_internal` : internal big-endian [u8;32] (computed tx hash)
/// - `merkle_siblings_internal` : vector of internal big-endian [u8;32]
/// - `pos` : index in block
/// - `merkle_root_internal` : internal big-endian [u8;32]
fn verify_merkle_inclusion(
    mut leaf_internal: [u8; 32],
    merkle_siblings_internal: Vec<[u8; 32]>,
    mut pos: usize,
    merkle_root_internal: [u8; 32],
) -> bool {
    for sibling in merkle_siblings_internal.iter() {
        let mut buf = [0u8; 64];
        if pos % 2 == 0 {
            buf[0..32].copy_from_slice(&leaf_internal);
            buf[32..64].copy_from_slice(sibling);
        } else {
            buf[0..32].copy_from_slice(sibling);
            buf[32..64].copy_from_slice(&leaf_internal);
        }
        leaf_internal = sha256d(&buf);
        pos >>= 1;
    }
    leaf_internal == merkle_root_internal
}

/// Verify merkle proof - wrapper around verify_merkle_inclusion
/// - `tx_hash` : internal big-endian [u8;32] (computed tx hash)
/// - `merkle_siblings` : vector of internal big-endian [u8;32]
/// - `pos` : index in block
/// - `merkle_root` : internal big-endian [u8;32]
pub fn verify_merkle_proof(
    tx_hash: [u8; 32],
    merkle_siblings: &[[u8; 32]],
    pos: usize,
    merkle_root: [u8; 32],
) -> bool {
    verify_merkle_inclusion(tx_hash, merkle_siblings.to_vec(), pos, merkle_root)
}

/// Decode bech32 P2WPKH (v0) -> 20-byte pubkey hash
fn decode_bech32_pubkey_hash(address: &str) -> Result<[u8; 20], String> {
    let (hrp, data, variant) = decode(address).map_err(|e| format!("bech32 decode: {}", e))?;
    if hrp != "bc" && hrp != "tb" {
        return Err(format!("unexpected hrp: {}", hrp));
    }
    if variant != Variant::Bech32 {
        return Err("expected Bech32 variant".into());
    }
    if data.is_empty() {
        return Err("bech32 data empty".into());
    }
    // first u5 is witness version (we expect 0)
    if data[0].to_u8() != 0 {
        return Err("non-zero witness version".into());
    }
    let converted =
        convert_bits(&data[1..], 5, 8, false).map_err(|_| "convert_bits failed".to_string())?;
    if converted.len() != 20 {
        return Err(format!("expected 20 bytes, got {}", converted.len()));
    }
    let mut out = [0u8; 20];
    out.copy_from_slice(&converted);
    Ok(out)
}

/// Sum outputs to the target address given parsed outputs (address,value)
fn sum_outputs_to_target(
    parsed_outputs: Vec<(String, u64)>,
    target_address: &str,
) -> Result<u64, String> {
    // Try to decode as bech32 first, then fall back to legacy address matching
    let target_hash = if target_address.starts_with("bc1") || target_address.starts_with("tb1") {
        decode_bech32_pubkey_hash(target_address)?
    } else {
        // For legacy addresses, we'll match by address string directly
        return sum_outputs_to_target_legacy(parsed_outputs, target_address);
    };

    let mut total: u64 = 0;
    let mut matched = false;
    for (addr, val) in parsed_outputs.iter() {
        if let Ok(h) = decode_bech32_pubkey_hash(addr) {
            if h == target_hash {
                total = total.checked_add(*val).ok_or("overflow adding outputs")?;
                matched = true;
            }
        }
    }
    if !matched {
        return Err("no outputs to target".into());
    }
    Ok(total)
}

/// Sum outputs to legacy target address by string matching
fn sum_outputs_to_target_legacy(
    parsed_outputs: Vec<(String, u64)>,
    target_address: &str,
) -> Result<u64, String> {
    let mut total: u64 = 0;
    let mut matched = false;
    for (addr, val) in parsed_outputs.iter() {
        if addr == target_address {
            total = total.checked_add(*val).ok_or("overflow adding outputs")?;
            matched = true;
        }
    }
    if !matched {
        return Err("no outputs to target".into());
    }
    Ok(total)
}

/// Extract merkle_root (internal big-endian) and compute block hash (display little-endian) from header hex
fn block_header_merkle_root_and_block_hash(header_hex: &str) -> Result<([u8; 32], String), String> {
    let header_bytes = hex::decode(header_hex).map_err(|e| format!("header hex decode: {}", e))?;
    if header_bytes.len() != 80 {
        return Err("block header must be 80 bytes".into());
    }
    // header layout: version(4) prev(32) merkle(32) time(4) bits(4) nonce(4)
    let merkle_root_internal: [u8; 32] = header_bytes[36..68].try_into().unwrap();
    // compute block hash (sha256d) and show as explorer display (little-endian hex)
    let block_hash_internal = sha256d(&header_bytes);
    let mut block_hash_disp = block_hash_internal;
    block_hash_disp.reverse();
    Ok((merkle_root_internal, hex::encode(block_hash_disp)))
}

/// Parse transaction outputs from transaction hex
/// Returns vector of (address, value) tuples
fn parse_tx_outputs(tx_hex: &str) -> Result<Vec<(String, u64)>, String> {
    let tx_bytes = hex::decode(tx_hex).map_err(|e| format!("tx hex decode: {}", e))?;
    let mut cursor = 0;

    // Skip version (4 bytes)
    if tx_bytes.len() < 4 {
        return Err("tx too short for version".into());
    }
    cursor += 4;

    // Check if this is a SegWit transaction (has witness marker)
    let is_segwit =
        tx_bytes.len() > 4 && tx_bytes[4] == 0x00 && tx_bytes.len() > 5 && tx_bytes[5] == 0x01;

    if is_segwit {
        // Skip witness marker (0x00) and flag (0x01)
        cursor += 2;
    }

    // Parse input count (varint)
    let (input_count, input_count_len) = parse_varint(&tx_bytes[cursor..])?;
    cursor += input_count_len;

    // Skip all inputs
    for _ in 0..input_count {
        // Skip previous txid (32 bytes) + vout (4 bytes)
        if cursor + 36 > tx_bytes.len() {
            return Err("tx too short for input".into());
        }
        cursor += 36;

        // Parse script length (varint)
        let (script_len, script_len_len) = parse_varint(&tx_bytes[cursor..])?;
        cursor += script_len_len;

        // Skip script + sequence (4 bytes)
        if cursor + script_len as usize + 4 > tx_bytes.len() {
            return Err("tx too short for input script".into());
        }
        cursor += script_len as usize + 4;
    }

    // Parse output count (varint)
    let (output_count, output_count_len) = parse_varint(&tx_bytes[cursor..])?;
    cursor += output_count_len;

    let mut outputs = Vec::new();

    // Parse each output
    for _ in 0..output_count {
        // Parse value (8 bytes, little-endian)
        if cursor + 8 > tx_bytes.len() {
            return Err("tx too short for output value".into());
        }
        let value_bytes = &tx_bytes[cursor..cursor + 8];
        let value = u64::from_le_bytes(value_bytes.try_into().unwrap());
        cursor += 8;

        // Parse script length (varint)
        let (script_len, script_len_len) = parse_varint(&tx_bytes[cursor..])?;
        cursor += script_len_len;

        // Parse script
        if cursor + script_len as usize > tx_bytes.len() {
            return Err("tx too short for output script".into());
        }
        let script = &tx_bytes[cursor..cursor + script_len as usize];
        cursor += script_len as usize;

        // Extract address from script (handles P2PKH and P2WPKH)
        if let Ok(address) = extract_p2pkh_address(script) {
            outputs.push((address, value));
        } else if let Ok(address) = extract_p2wpkh_address(script) {
            outputs.push((address, value));
        }
    }

    Ok(outputs)
}

/// Parse variable-length integer (varint)
fn parse_varint(data: &[u8]) -> Result<(u64, usize), String> {
    if data.is_empty() {
        return Err("empty varint".into());
    }

    match data[0] {
        0xfd => {
            if data.len() < 3 {
                return Err("varint too short for 0xfd".into());
            }
            let value = u16::from_le_bytes([data[1], data[2]]);
            Ok((value as u64, 3))
        }
        0xfe => {
            if data.len() < 5 {
                return Err("varint too short for 0xfe".into());
            }
            let value = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
            Ok((value as u64, 5))
        }
        0xff => {
            if data.len() < 9 {
                return Err("varint too short for 0xff".into());
            }
            let value = u64::from_le_bytes([
                data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            ]);
            Ok((value, 9))
        }
        n => Ok((n as u64, 1)),
    }
}

/// Extract P2PKH address from script (simplified)
fn extract_p2pkh_address(script: &[u8]) -> Result<String, String> {
    // P2PKH script: OP_DUP OP_HASH160 OP_PUSHBYTES_20 <20-byte-hash> OP_EQUALVERIFY OP_CHECKSIG
    // Pattern: 76a914<20 bytes>88ac
    if script.len() != 25
        || script[0] != 0x76
        || script[1] != 0xa9
        || script[2] != 0x14
        || script[23] != 0x88
        || script[24] != 0xac
    {
        return Err("not a P2PKH script".into());
    }

    let pubkey_hash = &script[3..23];

    // Create legacy P2PKH address: version_byte(1) + pubkey_hash(20) + checksum(4)
    let mut address_bytes = Vec::new();
    address_bytes.push(0x00); // Mainnet version byte
    address_bytes.extend_from_slice(pubkey_hash);

    // Calculate checksum (first 4 bytes of double SHA256)
    let checksum = sha256d(&address_bytes);
    address_bytes.extend_from_slice(&checksum[..4]);

    // Encode to base58
    Ok(bs58::encode(&address_bytes).into_string())
}

/// Extract P2WPKH address from script
fn extract_p2wpkh_address(script: &[u8]) -> Result<String, String> {
    // P2WPKH script: OP_0 OP_PUSHBYTES_20 <20-byte-hash>
    // Pattern: 0014<20 bytes>
    if script.len() != 22 || script[0] != 0x00 || script[1] != 0x14 {
        return Err("not a P2WPKH script".into());
    }

    let pubkey_hash = &script[2..22];

    // Convert 8-bit bytes to 5-bit groups
    let converted = convert_bits(pubkey_hash, 8, 5, true)
        .map_err(|_| "convert_bits failed for P2WPKH".to_string())?;

    // Convert Vec<u8> to Vec<u5> for bech32 encoding
    let mut data_u5: Vec<u5> = Vec::new();
    data_u5.push(u5::try_from_u8(0).unwrap()); // witness version 0
    for byte in converted {
        data_u5.push(u5::try_from_u8(byte).unwrap());
    }

    // Encode as bech32
    Ok(bech32::encode("bc", data_u5, Variant::Bech32)
        .map_err(|e| format!("bech32 encode failed: {}", e))
        .unwrap())
}

/// Combined verification function
/// Returns (block_hash_display_hex, total_amount) on success
pub fn verify_tx_in_block_and_outputs(
    tx_hex: &str,
    expected_txid_hex: &str,
    merkle_hex_siblings: Vec<String>,
    pos: usize,
    block_header_hex: &str,
    target_address: &str,
) -> Result<(String, u64), String> {
    // 1) txid correctness
    if !verify_txid(expected_txid_hex, tx_hex)? {
        return Err("txid mismatch".into());
    }

    // 2) leaf internal
    let leaf_internal = compute_raw_tx_hash_from_txhex(tx_hex)?;

    // 3) convert siblings to internal
    let mut siblings_internal = Vec::with_capacity(merkle_hex_siblings.len());
    for s in merkle_hex_siblings.iter() {
        siblings_internal.push(hex_sibling_to_internal(s)?);
    }

    // 4) extract merkle_root and block hash
    let (merkle_root_internal, block_hash_disp) =
        block_header_merkle_root_and_block_hash(block_header_hex)?;

    // 5) merkle inclusion
    let merkle_ok = verify_merkle_inclusion(
        leaf_internal,
        siblings_internal.clone(),
        pos,
        merkle_root_internal,
    );
    if !merkle_ok {
        return Err("merkle inclusion failed".into());
    }
    // 6) parse actual outputs from transaction
    let actual_outputs = parse_tx_outputs(tx_hex)?;

    // 7) sum outputs to target and ensure >0
    let total = sum_outputs_to_target(actual_outputs, target_address)?;

    // success
    Ok((block_hash_disp, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convert hex string (explorer display) -> internal big-endian [u8;32]
    fn hex_rev32(hex_str: &str) -> [u8; 32] {
        let bytes = hex::decode(hex_str).unwrap();
        let mut arr: [u8; 32] = bytes.as_slice().try_into().unwrap();
        // explorer gives little-endian display; convert to internal big-endian
        arr.reverse();
        arr
    }

    /// Reverse 32-byte array (internal <-> explorer display)
    fn rev32(mut a: [u8; 32]) -> [u8; 32] {
        a.reverse();
        a
    }

    #[test]
    fn test_parse_tx_outputs() {
        // Test with the actual transaction from our test case
        let tx_hex = "010000000536a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0c0000006b483045022100bcdf40fb3b5ebfa2c158ac8d1a41c03eb3dba4e180b00e81836bafd56d946efd022005cc40e35022b614275c1e485c409599667cbd41f6e5d78f421cb260a020a24f01210255ea3f53ce3ed1ad2c08dfc23b211b15b852afb819492a9a0f3f99e5747cb5f0ffffffffee08cb90c4e84dd7952b2cfad81ed3b088f5b32183da2894c969f6aa7ec98405020000006a47304402206332beadf5302281f88502a53cc4dd492689057f2f2f0f82476c1b5cd107c14a02207f49abc24fc9d94270f53a4fb8a8fbebf872f85fff330b72ca91e06d160dcda50121027943329cc801a8924789dc3c561d89cf234082685cbda90f398efa94f94340f2ffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f060000006b4830450221009c97a25ae70e208b25306cc870686c1f0c238100e9100aa2599b3cd1c010d8ff0220545b34c80ed60efcfbd18a7a22f00b5f0f04cfe58ca30f21023b873a959f1bd3012102e54cd4a05fe29be75ad539a80e7a5608a15dffbfca41bec13f6bf4a32d92e2f4ffffffff73cabea6245426bf263e7ec469a868e2e12a83345e8d2a5b0822bc7f43853956050000006b483045022100b934aa0f5cf67f284eebdf4faa2072345c2e448b758184cee38b7f3430129df302200dffac9863e03e08665f3fcf9683db0000b44bf1e308721eb40d76b180a457ce012103634b52718e4ddf125f3e66e5a3cd083765820769fd7824fd6aa38eded48cd77fffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0b0000006a47304402206348e277f65b0d23d8598944cc203a477ba1131185187493d164698a2b13098a02200caaeb6d3847b32568fd58149529ef63f0902e7d9c9b4cc5f9422319a8beecd50121025af6ba0ccd2b7ac96af36272ae33fa6c793aa69959c97989f5fa397eb8d13e69ffffffff0400e6e849000000001976a91472d52e2f5b88174c35ee29844cce0d6d24b921ef88ac20aaa72e000000001976a914c15b731d0116ef8192f240d4397a8cdbce5fe8bc88acf02cfa51000000001976a914c7ee32e6945d7de5a4541dd2580927128c11517488acf012e39b000000001976a9140a59837ccd4df25adc31cdad39be6a8d97557ed688ac00000000";

        let result = parse_tx_outputs(tx_hex);
        assert!(result.is_ok());
        let outputs = result.unwrap();
        dbg!(&outputs);

        // Should have 4 outputs
        assert_eq!(outputs.len(), 4);

        // Check the values match what we expect from the API
        let expected_values = vec![1240000000, 782740000, 1375350000, 2615350000];
        let actual_values: Vec<u64> = outputs.iter().map(|(_, v)| *v).collect();

        // Sort both to compare (order might be different)
        let mut expected_sorted = expected_values;
        let mut actual_sorted = actual_values;
        expected_sorted.sort();
        actual_sorted.sort();

        assert_eq!(expected_sorted, actual_sorted);

        // Check that addresses are correctly generated
        let expected_addresses = vec![
            "1BUBQuPV3gEV7P2XLNuAJQjf5t265Yyj9t",
            "1JdNy4KCNVQ6ay8qsc52DW1TtS7ZCnvJ5W",
            "1KE8pX7V7D8b4Cd5DL1jZwjy2vS5NtZpBT",
            "1wizSAYSbuyXbt9d8JV8ytm5acqq2TorC",
        ];
        let actual_addresses: Vec<&str> = outputs.iter().map(|(addr, _)| addr.as_str()).collect();

        // Sort both to compare (order might be different)
        let mut expected_addr_sorted = expected_addresses;
        let mut actual_addr_sorted = actual_addresses;
        expected_addr_sorted.sort();
        actual_addr_sorted.sort();

        assert_eq!(expected_addr_sorted, actual_addr_sorted);
    }

    #[test]
    fn test_parse_tx_outputs_new_transaction() {
        // Test with the new transaction: cce9ac461e348a6863a5ab91a7f23261b6b395337fe59787a7674b996496311d
        let tx_hex = "02000000000105fcb90a06d2390c467c1189a456ded18ada3aaa44319d9ace0b2e7feaf4bf599a0000000017160014e6b4c5ff28851b556728a07ac6f39c30e8d5338cffffffff9665ad7b601c071dd10d4e5f16eecda6b1a8923572c66c9eac6ea99d03112722000000001716001424e200da3ebf9364302da53a9ea34426ef99e2d5ffffffffcff9b155c625f48d028d81c123411ec30524ad8124b2979f6791db242019ab2e000000001716001418a080e34d1654114c16f69a0fe198b7303b0339ffffffff852a1fd197008c669cc29cbe007e585facf45a7eaa724a3c298737942e6b90850100000000ffffffff66f159174c8d670ec596819c7aba0e68c15701c9924527b44343a35a8235274a0100000000ffffffff024ae98100000000001600145b983b1242987fab8dedad0358e2d294534ab95b081400000000000016001480b6e1230a6b2ffe47a2a54cb43054dbf113c95902473044022057a2196d29b66b790c013baa60eb0de5d2239ef74e3d0823c2d833aed2dc0af602204af18daff3f5b1c9c8404586964deded9484ca3e904f7ddc17b8795c0b6a884801210200746b4cccbff680f23f86fbd69cbe1a5140cea10744aea67991f4e3f0009164024730440220361e863eb5b1579ec8f732d5af99db0d5f182f9f12e53777452825d8a2e9050202202bc738c13b1a6a4382f8b5779e0b86862684704a02f70dfe7b0edfef26439a9a01210227d231e32ddaaa3c276e98bf4a50197d753f1a30505d829e9a0453945d94970102473044022028dbeb2d9e5d758676b10d168a947d87789a0e79a4a05b4eb51fb8a5dd5f08f9022030c760ea64f609d21027f3b552cb04cc4fff1ad1e21e7b9a0194930c5590b04601210226e68b416d21c0fbb393312b0ba25ce16ec57529ccc72452af5e5ece52d19e8202473044022069a29449588622ef7284e0eef08e1f0b814390e05cd746cf1e5f195b6f20796102204f74e333bd66c12dfd57c53ae4af4d911463fccf80982f25cc8c7bffb8b8bb1a012102aadde2bccb94dac97bd6904d33053d8ed9f514425b2cc277184f4b9fb9c002cd0247304402205b9ec23e409392a95b7c752c2ffeb94b4530fbd679fe1cedc21725b7dc0bc2960220391e91692bee0c04fff1c008ee1020fde1a842551873a0a96423bd1904d0c0d601210265d2453707c07b2b10b0411473aba1f1b84aa3de6968f6cf893b8b63a2f36b3900000000";

        let result = parse_tx_outputs(tx_hex);
        println!("Parse result: {:?}", result);

        if let Ok(outputs) = result {
            println!("Number of outputs: {}", outputs.len());
            for (i, (addr, value)) in outputs.iter().enumerate() {
                println!("Output {}: address={}, value={}", i, addr, value);
            }

            // Expected from API: 2 outputs
            // 1. bc1qtwvrkyjznpl6hr0d45p43ckjj3f54w2m89j4n2 - 8513866 satoshis
            // 2. bc1qszmwzgc2dvhlu3az54xtgvz5m0c38j2eklg80q - 5128 satoshis
            assert_eq!(outputs.len(), 2);

            // Check values
            let values: Vec<u64> = outputs.iter().map(|(_, v)| *v).collect();
            assert!(values.contains(&8513866));
            assert!(values.contains(&5128));
        } else {
            panic!("Failed to parse transaction outputs: {:?}", result.err());
        }
    }

    #[test]
    fn test_sha256d() {
        let test_data = b"hello world";
        let hash = sha256d(test_data);
        let expected_hash = "bc62d4b80d9e36da29c16c5d4d9f11731f36052c72401a76c23c0fb5a9b74423";
        assert_eq!(hex::encode(hash), expected_hash);
    }

    #[test]
    fn test_compute_raw_tx_hash_from_txhex() {
        // Test with valid hex
        let tx_hex = "010000000536a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0c0000006b483045022100bcdf40fb3b5ebfa2c158ac8d1a41c03eb3dba4e180b00e81836bafd56d946efd022005cc40e35022b614275c1e485c409599667cbd41f6e5d78f421cb260a020a24f01210255ea3f53ce3ed1ad2c08dfc23b211b15b852afb819492a9a0f3f99e5747cb5f0ffffffffee08cb90c4e84dd7952b2cfad81ed3b088f5b32183da2894c969f6aa7ec98405020000006a47304402206332beadf5302281f88502a53cc4dd492689057f2f2f0f82476c1b5cd107c14a02207f49abc24fc9d94270f53a4fb8a8fbebf872f85fff330b72ca91e06d160dcda50121027943329cc801a8924789dc3c561d89cf234082685cbda90f398efa94f94340f2ffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f060000006b4830450221009c97a25ae70e208b25306cc870686c1f0c238100e9100aa2599b3cd1c010d8ff0220545b34c80ed60efcfbd18a7a22f00b5f0f04cfe58ca30f21023b873a959f1bd3012102e54cd4a05fe29be75ad539a80e7a5608a15dffbfca41bec13f6bf4a32d92e2f4ffffffff73cabea6245426bf263e7ec469a868e2e12a83345e8d2a5b0822bc7f43853956050000006b483045022100b934aa0f5cf67f284eebdf4faa2072345c2e448b758184cee38b7f3430129df302200dffac9863e03e08665f3fcf9683db0000b44bf1e308721eb40d76b180a457ce012103634b52718e4ddf125f3e66e5a3cd083765820769fd7824fd6aa38eded48cd77fffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0b0000006a47304402206348e277f65b0d23d8598944cc203a477ba1131185187493d164698a2b13098a02200caaeb6d3847b32568fd58149529ef63f0902e7d9c9b4cc5f9422319a8beecd50121025af6ba0ccd2b7ac96af36272ae33fa6c793aa69959c97989f5fa397eb8d13e69ffffffff0400e6e849000000001976a91472d52e2f5b88174c35ee29844cce0d6d24b921ef88ac20aaa72e000000001976a914c15b731d0116ef8192f240d4397a8cdbce5fe8bc88acf02cfa51000000001976a914c7ee32e6945d7de5a4541dd2580927128c11517488acf012e39b000000001976a9140a59837ccd4df25adc31cdad39be6a8d97557ed688ac00000000";

        let result = compute_raw_tx_hash_from_txhex(tx_hex);
        assert!(result.is_ok());
        let mut hash = result.unwrap();
        hash = rev32(hash);

        assert_eq!(hash.len(), 32);
        // Verify the hash is the expected txid (in internal big-endian format)
        let expected_hash = "15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521";
        assert_eq!(hex::encode(hash), expected_hash);

        // Test with invalid hex
        let invalid_hex = "invalid_hex";
        let result = compute_raw_tx_hash_from_txhex(invalid_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_txid() {
        // Test with valid txid and tx hex
        let tx_hex = "010000000536a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0c0000006b483045022100bcdf40fb3b5ebfa2c158ac8d1a41c03eb3dba4e180b00e81836bafd56d946efd022005cc40e35022b614275c1e485c409599667cbd41f6e5d78f421cb260a020a24f01210255ea3f53ce3ed1ad2c08dfc23b211b15b852afb819492a9a0f3f99e5747cb5f0ffffffffee08cb90c4e84dd7952b2cfad81ed3b088f5b32183da2894c969f6aa7ec98405020000006a47304402206332beadf5302281f88502a53cc4dd492689057f2f2f0f82476c1b5cd107c14a02207f49abc24fc9d94270f53a4fb8a8fbebf872f85fff330b72ca91e06d160dcda50121027943329cc801a8924789dc3c561d89cf234082685cbda90f398efa94f94340f2ffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f060000006b4830450221009c97a25ae70e208b25306cc870686c1f0c238100e9100aa2599b3cd1c010d8ff0220545b34c80ed60efcfbd18a7a22f00b5f0f04cfe58ca30f21023b873a959f1bd3012102e54cd4a05fe29be75ad539a80e7a5608a15dffbfca41bec13f6bf4a32d92e2f4ffffffff73cabea6245426bf263e7ec469a868e2e12a83345e8d2a5b0822bc7f43853956050000006b483045022100b934aa0f5cf67f284eebdf4faa2072345c2e448b758184cee38b7f3430129df302200dffac9863e03e08665f3fcf9683db0000b44bf1e308721eb40d76b180a457ce012103634b52718e4ddf125f3e66e5a3cd083765820769fd7824fd6aa38eded48cd77fffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0b0000006a47304402206348e277f65b0d23d8598944cc203a477ba1131185187493d164698a2b13098a02200caaeb6d3847b32568fd58149529ef63f0902e7d9c9b4cc5f9422319a8beecd50121025af6ba0ccd2b7ac96af36272ae33fa6c793aa69959c97989f5fa397eb8d13e69ffffffff0400e6e849000000001976a91472d52e2f5b88174c35ee29844cce0d6d24b921ef88ac20aaa72e000000001976a914c15b731d0116ef8192f240d4397a8cdbce5fe8bc88acf02cfa51000000001976a914c7ee32e6945d7de5a4541dd2580927128c11517488acf012e39b000000001976a9140a59837ccd4df25adc31cdad39be6a8d97557ed688ac00000000";
        let txid_hex = "15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521";

        let result = verify_txid(txid_hex, tx_hex);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test with mismatched txid
        let wrong_txid = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = verify_txid(wrong_txid, tx_hex);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Test with invalid hex
        let result = verify_txid("invalid", tx_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_sibling_to_internal() {
        // Test with valid hex sibling (little-endian display -> big-endian internal)
        let hex_sibling = "15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521";
        let result = hex_sibling_to_internal(hex_sibling);
        assert!(result.is_ok());
        let internal = result.unwrap();
        assert_eq!(internal.len(), 32);
        // Verify the conversion: little-endian -> big-endian
        let expected_internal = "2185dcea0f9d66cb7b6b69c42c41127c3ddd1b1991f3ce99a89355f14507e115";
        assert_eq!(hex::encode(internal), expected_internal);

        // Test with invalid hex
        let result = hex_sibling_to_internal("invalid");
        assert!(result.is_err());

        // Test with wrong length
        let result = hex_sibling_to_internal("1234");
        assert!(result.is_err());
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

    #[test]
    fn test_decode_bech32_pubkey_hash() {
        // Test with valid mainnet address
        let address = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let result = decode_bech32_pubkey_hash(address);
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash.len(), 20);
        // Verify the decoded pubkey hash
        let expected_hash = "751e76e8199196d454941c45d1b3a323f1433bd6";
        assert_eq!(hex::encode(hash), expected_hash);

        // Test with valid testnet address
        let testnet_address = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
        let result = decode_bech32_pubkey_hash(testnet_address);
        assert!(result.is_ok());
        let testnet_hash = result.unwrap();
        assert_eq!(testnet_hash.len(), 20);

        // Test with invalid address
        let invalid_address = "invalid_address";
        let result = decode_bech32_pubkey_hash(invalid_address);
        assert!(result.is_err());

        // Test with wrong HRP
        let wrong_hrp = "ltc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let result = decode_bech32_pubkey_hash(wrong_hrp);
        assert!(result.is_err());
    }

    #[test]
    fn test_sum_outputs_to_target() {
        let target_address = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let outputs = vec![
            (target_address.to_string(), 1000),
            (
                "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3".to_string(),
                2000,
            ),
            (target_address.to_string(), 500),
        ];

        let result = sum_outputs_to_target(outputs.clone(), target_address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1500);

        // Test with no outputs to target
        let outputs_no_match = vec![(
            "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3".to_string(),
            2000,
        )];
        let result = sum_outputs_to_target(outputs_no_match, target_address);
        assert!(result.is_err());

        // Test with invalid target address
        let result = sum_outputs_to_target(outputs, "invalid_address");
        assert!(result.is_err());
    }

    #[test]
    fn test_block_header_merkle_root_and_block_hash() {
        // Test with valid 80-byte header
        let header_hex = "0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c";
        let result = block_header_merkle_root_and_block_hash(header_hex);
        assert!(result.is_ok());
        let (merkle_root, block_hash) = result.unwrap();
        assert_eq!(merkle_root.len(), 32);
        assert_eq!(block_hash.len(), 64); // hex string length

        // Test with invalid length
        let invalid_header = "01000000";
        let result = block_header_merkle_root_and_block_hash(invalid_header);
        assert!(result.is_err());

        // Test with invalid hex
        let result = block_header_merkle_root_and_block_hash("invalid_hex");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_tx_in_block_and_outputs() {
        // Real mainnet transaction: 15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521
        let tx_hex = "010000000536a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0c0000006b483045022100bcdf40fb3b5ebfa2c158ac8d1a41c03eb3dba4e180b00e81836bafd56d946efd022005cc40e35022b614275c1e485c409599667cbd41f6e5d78f421cb260a020a24f01210255ea3f53ce3ed1ad2c08dfc23b211b15b852afb819492a9a0f3f99e5747cb5f0ffffffffee08cb90c4e84dd7952b2cfad81ed3b088f5b32183da2894c969f6aa7ec98405020000006a47304402206332beadf5302281f88502a53cc4dd492689057f2f2f0f82476c1b5cd107c14a02207f49abc24fc9d94270f53a4fb8a8fbebf872f85fff330b72ca91e06d160dcda50121027943329cc801a8924789dc3c561d89cf234082685cbda90f398efa94f94340f2ffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f060000006b4830450221009c97a25ae70e208b25306cc870686c1f0c238100e9100aa2599b3cd1c010d8ff0220545b34c80ed60efcfbd18a7a22f00b5f0f04cfe58ca30f21023b873a959f1bd3012102e54cd4a05fe29be75ad539a80e7a5608a15dffbfca41bec13f6bf4a32d92e2f4ffffffff73cabea6245426bf263e7ec469a868e2e12a83345e8d2a5b0822bc7f43853956050000006b483045022100b934aa0f5cf67f284eebdf4faa2072345c2e448b758184cee38b7f3430129df302200dffac9863e03e08665f3fcf9683db0000b44bf1e308721eb40d76b180a457ce012103634b52718e4ddf125f3e66e5a3cd083765820769fd7824fd6aa38eded48cd77fffffffff36a007284bd52ee826680a7f43536472f1bcce1e76cd76b826b88c5884eddf1f0b0000006a47304402206348e277f65b0d23d8598944cc203a477ba1131185187493d164698a2b13098a02200caaeb6d3847b32568fd58149529ef63f0902e7d9c9b4cc5f9422319a8beecd50121025af6ba0ccd2b7ac96af36272ae33fa6c793aa69959c97989f5fa397eb8d13e69ffffffff0400e6e849000000001976a91472d52e2f5b88174c35ee29844cce0d6d24b921ef88ac20aaa72e000000001976a914c15b731d0116ef8192f240d4397a8cdbce5fe8bc88acf02cfa51000000001976a914c7ee32e6945d7de5a4541dd2580927128c11517488acf012e39b000000001976a9140a59837ccd4df25adc31cdad39be6a8d97557ed688ac00000000";
        let expected_txid = "15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521";

        // Merkle siblings from the actual transaction (would need to get from block explorer)
        let merkle_siblings = vec![
            "acf931fe8980c6165b32fe7a8d25f779af7870a638599db1977d5309e24d2478".to_string(),
            "ee25997c2520236892c6a67402650e6b721899869dcf6715294e98c0b45623f9".to_string(),
            "790889ac7c0f7727715a7c1f1e8b05b407c4be3bd304f88c8b5b05ed4c0c24b7".to_string(),
            "facfd99cc4cfe45e66601b37a9637e17fb2a69947b1f8dc3118ed7a50ba7c901".to_string(),
            "8c871dd0b7915a114f274c354d8b6c12c689b99851edc55d29811449a6792ab7".to_string(),
            "eb4d9605966b26cfa3bf69b1afebe375d3d6aadaa7f2899d48899b6bd2fd6a43".to_string(),
            "daa1dc59f22a8601b489fc8a89da78bc35415291c62c185e711b8eef341e6e70".to_string(),
            "102907c1b95874e2893c6f7f06b45a3d52455d3bb17796e761df75aeda6aa065".to_string(),
            "baeede9b8e022bb98b63cb765ba5ca3e66e414bfd37702b349a04113bcfcaba6".to_string(),
            "b6f07be94b55144588b33ff39fb8a08004baa03eb7ff121e1847d715d0da6590".to_string(),
            "7d02c62697d783d85a51cd4f37a87987b8b3077df4ddd1227b254f59175ed1e4".to_string(),
        ];

        let pos = 1465; // Transaction position in block
                        // Real block header from mainnet block 363348
        let block_header = "0300000058f6dd09ac5aea942c01d12e75b351e73f4304cc442741000000000000000000ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd09ae093558e411618c14240df";

        let target_address = "1BUBQuPV3gEV7P2XLNuAJQjf5t265Yyj9t";

        let result = verify_tx_in_block_and_outputs(
            tx_hex,
            expected_txid,
            merkle_siblings.clone(),
            pos,
            block_header,
            target_address,
        );
        if let Err(e) = &result {
            println!("Error: {}", e);
            println!("Block header length: {}", block_header.len());
            println!("Block header: {}", block_header);
        }
        assert!(result.is_ok());
        let (block_hash, total) = result.unwrap();
        assert_eq!(total, 1240000000);
        assert_eq!(block_hash.len(), 64);

        // Test with wrong txid
        let wrong_txid = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = verify_tx_in_block_and_outputs(
            tx_hex,
            wrong_txid,
            merkle_siblings.clone(),
            pos,
            block_header,
            target_address,
        );
        assert!(result.is_err());

        // Test with no outputs to target (use a different target address)
        let result = verify_tx_in_block_and_outputs(
            tx_hex,
            expected_txid,
            merkle_siblings,
            pos,
            block_header,
            "1InvalidAddressThatDoesNotExist123456789",
        );
        assert!(result.is_err());
    }
}
