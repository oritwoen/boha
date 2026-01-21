//! Cryptographic verification of puzzle private keys.
//!
//! This module provides functions to verify that a puzzle's private key
//! correctly derives its stored address across multiple blockchains.

use crate::{PubkeyFormat, Puzzle};
use k256::ecdsa::SigningKey;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::PublicKey;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Result of a verification operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyResult {
    /// Puzzle ID that was verified
    pub id: String,
    /// Whether verification succeeded
    pub verified: bool,
    /// Private key hex (if available)
    pub private_key: Option<String>,
    /// Expected address from puzzle data
    pub expected_address: String,
    /// Derived address from private key
    pub derived_address: Option<String>,
    /// Error message if verification failed
    pub error: Option<String>,
}

/// Errors that can occur during verification.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VerifyError {
    #[error("Puzzle has no private key")]
    NoPrivateKey,

    #[error("Invalid private key format: {0}")]
    InvalidKey(String),

    #[error("Address derivation failed: {0}")]
    DerivationFailed(String),

    #[error("Verification failed: expected {expected}, got {derived}")]
    Mismatch { expected: String, derived: String },

    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
}

impl VerifyResult {
    /// Create a successful verification result.
    pub fn success(id: String, private_key: String, address: String) -> Self {
        Self {
            id,
            verified: true,
            private_key: Some(private_key),
            expected_address: address.clone(),
            derived_address: Some(address),
            error: None,
        }
    }

    /// Create a failed verification result.
    pub fn failure(id: String, address: String, error: VerifyError) -> Self {
        Self {
            id,
            verified: false,
            private_key: None,
            expected_address: address,
            derived_address: None,
            error: Some(error.to_string()),
        }
    }
}

/// Verify a puzzle's private key derives its address.
///
/// This is a placeholder that will be implemented in subsequent tasks.
pub fn verify_puzzle(_puzzle: &Puzzle) -> Result<VerifyResult, VerifyError> {
    // Placeholder - will be implemented in Task 2-5
    Err(VerifyError::DerivationFailed(
        "Not yet implemented".to_string(),
    ))
}

fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn hash160(data: &[u8]) -> [u8; 20] {
    let sha = sha256(data);
    Ripemd160::digest(sha).into()
}

/// HASH160 using BLAKE-256 instead of SHA-256 (for Decred).
/// Formula: RIPEMD160(BLAKE256(data))
#[cfg(feature = "cli")]
fn hash160_blake256(data: &[u8]) -> [u8; 20] {
    use blake_hash::{Blake256, Digest};
    let blake = Blake256::digest(data);
    Ripemd160::digest(blake).into()
}

pub fn verify_bitcoin_address(
    hex_key: &str,
    expected_address: &str,
    pubkey_format: PubkeyFormat,
) -> Result<String, VerifyError> {
    let key_bytes =
        hex::decode(hex_key).map_err(|e| VerifyError::InvalidKey(format!("Invalid hex: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(VerifyError::InvalidKey(format!(
            "Key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into())
        .map_err(|e| VerifyError::InvalidKey(format!("Invalid secp256k1 key: {}", e)))?;

    let public_key = PublicKey::from(signing_key.verifying_key());

    let pubkey_bytes = match pubkey_format {
        PubkeyFormat::Compressed => public_key.to_sec1_bytes().to_vec(),
        PubkeyFormat::Uncompressed => public_key.to_encoded_point(false).as_bytes().to_vec(),
    };

    let hash = hash160(&pubkey_bytes);

    if expected_address.starts_with("bc1q") {
        verify_p2wpkh(&hash, expected_address)
    } else if expected_address.starts_with('1') || expected_address.starts_with('3') {
        verify_p2pkh(&hash, expected_address)
    } else {
        Err(VerifyError::UnsupportedChain(format!(
            "Unsupported address format: {}",
            expected_address
        )))
    }
}

fn verify_p2pkh(hash160: &[u8; 20], expected_address: &str) -> Result<String, VerifyError> {
    let mut data = vec![0x00];
    data.extend_from_slice(hash160);
    let checksum = &sha256(&sha256(&data))[..4];
    data.extend_from_slice(checksum);

    let derived = bs58::encode(data).into_string();

    if derived == expected_address {
        Ok(derived)
    } else {
        Err(VerifyError::Mismatch {
            expected: expected_address.to_string(),
            derived,
        })
    }
}

fn verify_p2wpkh(hash160: &[u8; 20], expected_address: &str) -> Result<String, VerifyError> {
    use bech32::{segwit, Hrp};

    let hrp = Hrp::parse("bc")
        .map_err(|e| VerifyError::DerivationFailed(format!("Invalid HRP: {}", e)))?;

    let witness_version = bech32::Fe32::Q;

    let derived = segwit::encode(hrp, witness_version, hash160)
        .map_err(|e| VerifyError::DerivationFailed(format!("Bech32 encoding failed: {}", e)))?;

    if derived == expected_address {
        Ok(derived)
    } else {
        Err(VerifyError::Mismatch {
            expected: expected_address.to_string(),
            derived,
        })
    }
}

/// Verify Ethereum address derivation from private key.
///
/// Ethereum addresses are derived as:
/// 1. Get uncompressed public key (65 bytes: 0x04 + x + y)
/// 2. Keccak256 hash of public key bytes (excluding 0x04 prefix)
/// 3. Take last 20 bytes
/// 4. Prefix with "0x" and lowercase hex
pub fn verify_ethereum_address(
    hex_key: &str,
    expected_address: &str,
) -> Result<String, VerifyError> {
    use tiny_keccak::{Hasher, Keccak};

    let key_bytes =
        hex::decode(hex_key).map_err(|e| VerifyError::InvalidKey(format!("Invalid hex: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(VerifyError::InvalidKey(format!(
            "Key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into())
        .map_err(|e| VerifyError::InvalidKey(format!("Invalid secp256k1 key: {}", e)))?;

    let public_key = PublicKey::from(signing_key.verifying_key());
    let pubkey_bytes = public_key.to_encoded_point(false);
    let pubkey_slice = pubkey_bytes.as_bytes();

    if pubkey_slice.len() != 65 || pubkey_slice[0] != 0x04 {
        return Err(VerifyError::DerivationFailed(
            "Invalid uncompressed public key".to_string(),
        ));
    }

    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(&pubkey_slice[1..]);
    keccak.finalize(&mut hash);

    let address_bytes = &hash[12..];
    let derived = format!("0x{}", hex::encode(address_bytes));

    let expected_lower = expected_address.to_lowercase();
    if derived == expected_lower {
        Ok(derived)
    } else {
        Err(VerifyError::Mismatch {
            expected: expected_address.to_string(),
            derived,
        })
    }
}

/// Verify Litecoin address derivation from private key.
///
/// Litecoin uses same algorithm as Bitcoin but different network bytes:
/// - P2PKH: 0x30 (addresses start with 'L')
/// - P2WPKH: bech32 with hrp "ltc"
pub fn verify_litecoin_address(
    hex_key: &str,
    expected_address: &str,
    pubkey_format: PubkeyFormat,
) -> Result<String, VerifyError> {
    let key_bytes =
        hex::decode(hex_key).map_err(|e| VerifyError::InvalidKey(format!("Invalid hex: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(VerifyError::InvalidKey(format!(
            "Key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into())
        .map_err(|e| VerifyError::InvalidKey(format!("Invalid secp256k1 key: {}", e)))?;

    let public_key = PublicKey::from(signing_key.verifying_key());

    let pubkey_bytes = match pubkey_format {
        PubkeyFormat::Compressed => public_key.to_sec1_bytes().to_vec(),
        PubkeyFormat::Uncompressed => public_key.to_encoded_point(false).as_bytes().to_vec(),
    };

    let hash = hash160(&pubkey_bytes);

    if expected_address.starts_with("ltc1") {
        verify_ltc_p2wpkh(&hash, expected_address)
    } else if expected_address.starts_with('L') || expected_address.starts_with('M') {
        verify_ltc_p2pkh(&hash, expected_address)
    } else {
        Err(VerifyError::UnsupportedChain(format!(
            "Unsupported Litecoin address format: {}",
            expected_address
        )))
    }
}

fn verify_ltc_p2pkh(hash160: &[u8; 20], expected_address: &str) -> Result<String, VerifyError> {
    let mut data = vec![0x30];
    data.extend_from_slice(hash160);
    let checksum = &sha256(&sha256(&data))[..4];
    data.extend_from_slice(checksum);

    let derived = bs58::encode(data).into_string();

    if derived == expected_address {
        Ok(derived)
    } else {
        Err(VerifyError::Mismatch {
            expected: expected_address.to_string(),
            derived,
        })
    }
}

fn verify_ltc_p2wpkh(hash160: &[u8; 20], expected_address: &str) -> Result<String, VerifyError> {
    use bech32::{segwit, Hrp};

    let hrp = Hrp::parse("ltc")
        .map_err(|e| VerifyError::DerivationFailed(format!("Invalid HRP: {}", e)))?;

    let witness_version = bech32::Fe32::Q;

    let derived = segwit::encode(hrp, witness_version, hash160)
        .map_err(|e| VerifyError::DerivationFailed(format!("Bech32 encoding failed: {}", e)))?;

    if derived == expected_address {
        Ok(derived)
    } else {
        Err(VerifyError::Mismatch {
            expected: expected_address.to_string(),
            derived,
        })
    }
}

/// Verify Decred address derivation from private key.
///
/// Decred P2PKH addresses use:
/// - Network bytes: [0x07, 0x3f] for mainnet
/// - BLAKE-256 for HASH160: RIPEMD160(BLAKE256(pubkey))
/// - Double BLAKE-256 for checksum: BLAKE256(BLAKE256(data))[0:4]
pub fn verify_decred_address(
    hex_key: &str,
    expected_address: &str,
    pubkey_format: PubkeyFormat,
) -> Result<String, VerifyError> {
    use blake_hash::{Blake256, Digest};

    let key_bytes =
        hex::decode(hex_key).map_err(|e| VerifyError::InvalidKey(format!("Invalid hex: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(VerifyError::InvalidKey(format!(
            "Key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into())
        .map_err(|e| VerifyError::InvalidKey(format!("Invalid secp256k1 key: {}", e)))?;

    let public_key = PublicKey::from(signing_key.verifying_key());

    let pubkey_bytes = match pubkey_format {
        PubkeyFormat::Compressed => public_key.to_sec1_bytes().to_vec(),
        PubkeyFormat::Uncompressed => public_key.to_encoded_point(false).as_bytes().to_vec(),
    };

    let hash = hash160_blake256(&pubkey_bytes);

    let mut data = vec![0x07, 0x3f];
    data.extend_from_slice(&hash);

    let first_hash = Blake256::digest(&data);
    let checksum_hash = Blake256::digest(&first_hash);
    let checksum = &checksum_hash[..4];
    data.extend_from_slice(checksum);

    let derived = bs58::encode(data).into_string();

    if derived == expected_address {
        Ok(derived)
    } else {
        Err(VerifyError::Mismatch {
            expected: expected_address.to_string(),
            derived,
        })
    }
}

/// Verify WIF (Wallet Import Format) private key.
///
/// WIF format:
/// - Version byte (0x80 for Bitcoin mainnet)
/// - 32-byte private key
/// - Optional compression flag (0x01)
/// - 4-byte checksum (double SHA256)
///
/// Supports:
/// - Compressed WIF (K/L prefix, 52 chars)
/// - Uncompressed WIF (5 prefix, 51 chars)
pub fn verify_wif(wif: &str, expected_address: &str) -> Result<String, VerifyError> {
    let decoded = bs58::decode(wif)
        .into_vec()
        .map_err(|e| VerifyError::InvalidKey(format!("Invalid base58: {}", e)))?;

    if decoded.len() != 37 && decoded.len() != 38 {
        return Err(VerifyError::InvalidKey(format!(
            "Invalid WIF length: {} bytes (expected 37 or 38)",
            decoded.len()
        )));
    }

    if decoded[0] != 0x80 {
        return Err(VerifyError::InvalidKey(format!(
            "Invalid network byte: 0x{:02x} (expected 0x80 for Bitcoin)",
            decoded[0]
        )));
    }

    let checksum_start = decoded.len() - 4;
    let payload = &decoded[..checksum_start];
    let checksum = &decoded[checksum_start..];

    let hash = sha256(&sha256(payload));
    if &hash[..4] != checksum {
        return Err(VerifyError::InvalidKey(
            "WIF checksum verification failed".to_string(),
        ));
    }

    let compressed = decoded.len() == 38;
    if compressed && decoded[33] != 0x01 {
        return Err(VerifyError::InvalidKey(format!(
            "Invalid compression flag: 0x{:02x} (expected 0x01)",
            decoded[33]
        )));
    }

    let key_bytes = &decoded[1..33];
    let hex_key = hex::encode(key_bytes);

    let pubkey_format = if compressed {
        PubkeyFormat::Compressed
    } else {
        PubkeyFormat::Uncompressed
    };

    verify_bitcoin_address(&hex_key, expected_address, pubkey_format)
}

/// Verify seed phrase derivation and address.
///
/// BIP39 + BIP32 workflow:
/// 1. Parse and validate BIP39 mnemonic phrase
/// 2. Generate seed from mnemonic (with optional passphrase)
/// 3. Derive private key using BIP32 path
/// 4. Verify derived address matches expected
pub fn verify_seed(
    phrase: &str,
    path: &str,
    expected_address: &str,
    pubkey_format: PubkeyFormat,
) -> Result<String, VerifyError> {
    use bip32::{DerivationPath, XPrv};
    use bip39::Mnemonic;
    use std::str::FromStr;

    let mnemonic = Mnemonic::parse_normalized(phrase)
        .map_err(|e| VerifyError::InvalidKey(format!("Invalid mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed("");

    let derivation_path = DerivationPath::from_str(path)
        .map_err(|e| VerifyError::InvalidKey(format!("Invalid derivation path: {}", e)))?;

    let xprv = XPrv::derive_from_path(seed, &derivation_path)
        .map_err(|e| VerifyError::DerivationFailed(format!("Key derivation failed: {}", e)))?;

    let private_key_bytes = xprv.private_key().to_bytes();
    let hex_key = hex::encode(private_key_bytes);

    verify_bitcoin_address(&hex_key, expected_address, pubkey_format)
}
