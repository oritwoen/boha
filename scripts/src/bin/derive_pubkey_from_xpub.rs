use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::path::Path;

fn zpub_to_xpub(zpub: &str) -> Result<String, Box<dyn std::error::Error>> {
    let decoded = bs58::decode(zpub).with_check(None).into_vec()?;
    if decoded.len() != 78 {
        return Err(format!("Invalid zpub length: {} (expected 78)", decoded.len()).into());
    }

    // zpub version: 0x04B24746, xpub version: 0x0488B21E
    let mut xpub_bytes = decoded.clone();
    xpub_bytes[0] = 0x04;
    xpub_bytes[1] = 0x88;
    xpub_bytes[2] = 0xB2;
    xpub_bytes[3] = 0x1E;

    Ok(bs58::encode(&xpub_bytes).with_check().into_string())
}

fn derive_child_pubkey(
    parent_pubkey: &[u8],
    parent_chaincode: &[u8],
    index: u32,
) -> Result<([u8; 33], [u8; 32]), Box<dyn std::error::Error>> {
    use hmac::{Hmac, Mac};
    use sha2::Sha512;

    if index >= 0x80000000 {
        return Err("Cannot derive hardened child from public key".into());
    }

    let mut data = Vec::with_capacity(37);
    data.extend_from_slice(parent_pubkey);
    data.extend_from_slice(&index.to_be_bytes());

    // BIP32 uses HMAC-SHA512 (64 bytes output)
    let mut mac = Hmac::<Sha512>::new_from_slice(parent_chaincode)?;
    mac.update(&data);
    let result = mac.finalize().into_bytes();

    let (il, ir) = result.split_at(32);

    // Add il to parent public key point (secp256k1)
    let child_pubkey = add_scalar_to_pubkey(parent_pubkey, il)?;

    let mut child_chaincode = [0u8; 32];
    child_chaincode.copy_from_slice(ir);

    Ok((child_pubkey, child_chaincode))
}

fn add_scalar_to_pubkey(
    pubkey: &[u8],
    scalar: &[u8],
) -> Result<[u8; 33], Box<dyn std::error::Error>> {
    use secp256k1::{PublicKey, Secp256k1, SecretKey};

    let secp = Secp256k1::new();
    let parent = PublicKey::from_slice(pubkey)?;
    let tweak = SecretKey::from_slice(scalar)?;

    let child = parent.add_exp_tweak(&secp, &tweak.into())?;
    Ok(child.serialize())
}

fn hash160(data: &[u8]) -> [u8; 20] {
    let sha256 = Sha256::digest(data);
    let ripemd = Ripemd160::digest(&sha256);
    let mut result = [0u8; 20];
    result.copy_from_slice(&ripemd);
    result
}

fn parse_xpub(xpub_str: &str) -> Result<([u8; 33], [u8; 32]), Box<dyn std::error::Error>> {
    let decoded = bs58::decode(xpub_str).with_check(None).into_vec()?;
    if decoded.len() != 78 {
        return Err(format!("Invalid xpub length: {}", decoded.len()).into());
    }

    // Bytes 45-78 contain the 33-byte public key
    let mut pubkey = [0u8; 33];
    pubkey.copy_from_slice(&decoded[45..78]);

    // Bytes 13-45 contain the 32-byte chain code
    let mut chaincode = [0u8; 32];
    chaincode.copy_from_slice(&decoded[13..45]);

    Ok((pubkey, chaincode))
}

fn derive_pubkey_from_zpub(
    zpub: &str,
    path: &[u32],
) -> Result<[u8; 33], Box<dyn std::error::Error>> {
    let xpub = zpub_to_xpub(zpub)?;
    let (mut pubkey, mut chaincode) = parse_xpub(&xpub)?;

    for &index in path {
        let (child_pubkey, child_chaincode) = derive_child_pubkey(&pubkey, &chaincode, index)?;
        pubkey = child_pubkey;
        chaincode = child_chaincode;
    }

    Ok(pubkey)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zpub = "zpub6qdEDkv51FpxX6g1rpFGckmiL46vV8ccmtEgPAkj3qj8N4ZZHyXDRA9RwpTiFK2Kb8vRaDmSmwgX6rfB4t2K8Ktdq8ExQ6fumKpn2ndJCqL";
    let expected_hash160 = "249dd7ad2fccea67977d4078edad50d8603ff4ce";

    // zpub is at m/84'/0'/0', we need to derive /0/0 to get m/84'/0'/0'/0/0
    let path = [0u32, 0u32];

    println!("Deriving public key from zpub...");
    println!("zpub: {}", zpub);
    println!("Path: /0/0 (relative to zpub at m/84'/0'/0')");

    let pubkey = derive_pubkey_from_zpub(zpub, &path)?;
    let pubkey_hex = hex::encode(&pubkey);

    println!("\nDerived public key: {}", pubkey_hex);

    let computed_hash160 = hash160(&pubkey);
    let computed_hash160_hex = hex::encode(&computed_hash160);

    println!("Computed HASH160:   {}", computed_hash160_hex);
    println!("Expected HASH160:   {}", expected_hash160);

    if computed_hash160_hex == expected_hash160 {
        println!("\n✓ HASH160 matches! Public key is correct.");

        // Update bitaps.jsonc
        let jsonc_path = Path::new("../data/bitaps.jsonc");
        if jsonc_path.exists() {
            let content = std::fs::read_to_string(jsonc_path)?;
            let mut value: serde_json::Value = serde_json::from_str(&content)?;

            if let Some(puzzle) = value.get_mut("puzzle") {
                puzzle["pubkey"] = serde_json::json!(&pubkey_hex);
                std::fs::write(jsonc_path, value.to_string())?;
                println!("\n✓ Updated bitaps.jsonc with pubkey");
            }
        }
    } else {
        println!("\n✗ HASH160 mismatch! Something is wrong.");
        return Err("Hash verification failed".into());
    }

    Ok(())
}
