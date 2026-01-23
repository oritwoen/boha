use boha::{
    b1000, bitaps, bitimage, gsmg, hash_collision, zden, Chain, Passphrase, PubkeyFormat, Status,
    TransactionType,
};
use num_bigint::BigUint;

#[test]
fn b1000_has_256_puzzles() {
    assert_eq!(b1000::all().count(), 256);
}

#[test]
fn b1000_puzzles_have_sequential_bits() {
    let bits: Vec<u16> = b1000::all()
        .filter_map(|p| p.key.and_then(|k| k.bits))
        .collect();

    for i in 1u16..=256 {
        assert!(bits.contains(&i), "Missing puzzle with bits={}", i);
    }
}

#[test]
fn b1000_get_returns_correct_puzzle() {
    let p1 = b1000::get(1).unwrap();
    assert_eq!(p1.key.and_then(|k| k.bits), Some(1));
    assert_eq!(p1.address.value, "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH");
    assert_eq!(p1.status, Status::Solved);

    let p66 = b1000::get(66).unwrap();
    assert_eq!(p66.key.and_then(|k| k.bits), Some(66));
    assert_eq!(p66.address.value, "13zb1hQbWVsc2S7ZTZnP2G4undNNpdh5so");

    let p256 = b1000::get(256).unwrap();
    assert_eq!(p256.key.and_then(|k| k.bits), Some(256));
}

#[test]
fn b1000_get_accepts_multiple_types() {
    let by_u32 = b1000::get(66_u32).unwrap();
    let by_str = b1000::get("66").unwrap();
    let by_usize = b1000::get(66_usize).unwrap();

    assert_eq!(by_u32.id, by_str.id);
    assert_eq!(by_u32.id, by_usize.id);

    assert!(b1000::get("abc").is_err());
    assert!(b1000::get(-1_i32).is_err());
}

#[test]
fn puzzle_key_range_valid() {
    let p1 = b1000::get(1).unwrap();
    let range = p1.key_range().unwrap();
    assert_eq!(*range.start(), 1);
    assert_eq!(*range.end(), 1);

    let p8 = b1000::get(8).unwrap();
    let range = p8.key_range().unwrap();
    assert_eq!(*range.start(), 128);
    assert_eq!(*range.end(), 255);

    let p66 = b1000::get(66).unwrap();
    let range = p66.key_range().unwrap();
    assert_eq!(*range.start(), 1u128 << 65);
    assert_eq!(*range.end(), (1u128 << 66) - 1);

    // bits == 128 edge case (max value for u128)
    let p128 = b1000::get(128).unwrap();
    let range = p128.key_range().unwrap();
    assert_eq!(*range.start(), 1u128 << 127);
    assert_eq!(*range.end(), u128::MAX);

    let p129 = b1000::get(129).unwrap();
    assert!(p129.key_range().is_none());
}

#[test]
fn puzzle_key_range_big_valid() {
    let p1 = b1000::get(1).unwrap();
    let (start, end) = p1.key_range_big().unwrap();
    assert_eq!(start, BigUint::from(1u32));
    assert_eq!(end, BigUint::from(1u32));

    let p66 = b1000::get(66).unwrap();
    let (start, end) = p66.key_range_big().unwrap();
    assert_eq!(start, BigUint::from(1u128) << 65);
    assert_eq!(end, (BigUint::from(1u128) << 66) - 1u32);

    let p256 = b1000::get(256).unwrap();
    let (start, end) = p256.key_range_big().unwrap();
    assert!(start > BigUint::ZERO);
    assert!(end > start);
}

#[test]
fn puzzle_key_range_none_for_p2sh() {
    for puzzle in hash_collision::all() {
        assert!(
            puzzle.address.redeem_script.is_some(),
            "hash_collision should have redeem_script"
        );
        assert!(puzzle.key_range().is_none());
        assert!(puzzle.key_range_big().is_none());
    }
}

#[test]
fn solved_puzzles_private_key_in_range() {
    for puzzle in b1000::solved() {
        let pk_hex = puzzle.key.and_then(|k| k.hex).unwrap();
        let pk_bytes = hex::decode(pk_hex).unwrap();
        let key = BigUint::from_bytes_be(&pk_bytes);

        let (start, end) = puzzle.key_range_big().unwrap();
        assert!(
            key >= start && key <= end,
            "Puzzle {} private_key not in range: key={}, range=[{}, {}]",
            puzzle.id,
            key,
            start,
            end
        );
    }
}

#[test]
fn b1000_all_addresses_start_with_1() {
    for puzzle in b1000::all() {
        assert!(
            puzzle.address.value.starts_with('1'),
            "BTC1000 address should start with 1: {}",
            puzzle.address.value
        );
        assert!(
            bs58::decode(puzzle.address.value).into_vec().is_ok(),
            "Invalid base58: {}",
            puzzle.address.value
        );
    }
}

#[test]
fn b1000_solved_have_private_keys() {
    for puzzle in b1000::solved() {
        assert!(
            puzzle.key.and_then(|k| k.hex).is_some(),
            "Solved puzzle {} missing private key",
            puzzle.id
        );
    }
}

#[test]
fn hash_collision_count() {
    assert_eq!(hash_collision::all().count(), 6);
}

#[test]
fn hash_collision_get_by_name() {
    let sha1 = hash_collision::get("sha1").unwrap();
    assert_eq!(sha1.address.value, "37k7toV1Nv4DfmQbmZ8KuZDQCYK9x5KpzP");
    assert_eq!(sha1.status, Status::Claimed);

    let sha256 = hash_collision::get("sha256").unwrap();
    assert_eq!(sha256.status, Status::Unsolved);
}

#[test]
fn hash_collision_all_p2sh() {
    for puzzle in hash_collision::all() {
        assert_eq!(puzzle.address.kind, "p2sh");
        assert!(
            puzzle.address.redeem_script.is_some(),
            "hash_collision should have redeem_script"
        );
    }
}

#[test]
fn gsmg_count() {
    assert_eq!(gsmg::all().count(), 1);
}

#[test]
fn gsmg_get_returns_correct_puzzle() {
    let puzzle = gsmg::get();
    assert_eq!(puzzle.id, "gsmg");
    assert_eq!(puzzle.address.value, "1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe");
    assert_eq!(puzzle.status, Status::Unsolved);
    assert_eq!(puzzle.address.kind, "p2pkh");
    assert_eq!(puzzle.chain, Chain::Bitcoin);
}

#[test]
fn universal_get_works() {
    assert!(boha::get("b1000/66").is_ok());
    assert!(boha::get("gsmg").is_ok());
    assert!(boha::get("hash_collision/sha256").is_ok());
    assert!(boha::get("peter_todd/sha256").is_ok());
}

#[test]
fn stats_are_reasonable() {
    let stats = boha::stats();
    assert!(stats.total > 270);
    assert!(stats.solved > 60);
    assert!(stats.unsolved > 50);
    assert!(stats.swept > 90);
    let total_btc = stats
        .total_prize
        .get(&Chain::Bitcoin)
        .copied()
        .unwrap_or(0.0);
    assert!(total_btc > 100.0);
}

#[test]
fn all_puzzles_have_start_date() {
    for puzzle in boha::all() {
        assert!(
            puzzle.start_date.is_some(),
            "Puzzle {} missing start_date",
            puzzle.id
        );
    }
}

#[test]
fn all_dates_have_time() {
    let datetime_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
    for puzzle in boha::all() {
        if let Some(date) = puzzle.start_date {
            assert!(
                datetime_regex.is_match(date),
                "Puzzle {} start_date must include time: {}",
                puzzle.id,
                date
            );
        }
        if let Some(date) = puzzle.solve_date {
            assert!(
                datetime_regex.is_match(date),
                "Puzzle {} solve_date must include time: {}",
                puzzle.id,
                date
            );
        }
    }
}

#[test]
fn start_date_before_solve_date() {
    for puzzle in boha::all() {
        if let (Some(start), Some(solve)) = (puzzle.start_date, puzzle.solve_date) {
            assert!(
                start <= solve,
                "Puzzle {} has start_date {} after solve_date {}",
                puzzle.id,
                start,
                solve
            );
        }
    }
}

#[test]
fn solve_time_matches_dates() {
    fn parse_datetime(s: &str) -> Option<i64> {
        let parts: Vec<&str> = s.split(&['-', ' ', ':'][..]).collect();
        if parts.len() != 6 {
            return None;
        }
        let year: i64 = parts[0].parse().ok()?;
        let month: i64 = parts[1].parse().ok()?;
        let day: i64 = parts[2].parse().ok()?;
        let hour: i64 = parts[3].parse().ok()?;
        let min: i64 = parts[4].parse().ok()?;
        let sec: i64 = parts[5].parse().ok()?;

        fn days_in_month(year: i64, month: i64) -> i64 {
            match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                        29
                    } else {
                        28
                    }
                }
                _ => 0,
            }
        }

        let mut days: i64 = 0;
        for y in 1970..year {
            days += if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                366
            } else {
                365
            };
        }
        for m in 1..month {
            days += days_in_month(year, m);
        }
        days += day - 1;

        Some(days * 86400 + hour * 3600 + min * 60 + sec)
    }

    for puzzle in boha::all() {
        if let (Some(start), Some(solve), Some(solve_time)) =
            (puzzle.start_date, puzzle.solve_date, puzzle.solve_time)
        {
            if let (Some(start_ts), Some(solve_ts)) = (parse_datetime(start), parse_datetime(solve))
            {
                let calculated = (solve_ts - start_ts) as u64;
                let diff = calculated.abs_diff(solve_time);
                assert!(
                    diff < 2,
                    "Puzzle {} solve_time mismatch: declared {} but calculated {} (diff: {}s)\n  start: {}\n  solve: {}",
                    puzzle.id,
                    solve_time,
                    calculated,
                    diff,
                    start,
                    solve
                );
            }
        }
    }
}

#[test]
fn source_url_format_valid() {
    for puzzle in boha::all() {
        if let Some(url) = puzzle.source_url {
            assert!(
                url.starts_with("http://") || url.starts_with("https://"),
                "Invalid source_url format for {}: {}",
                puzzle.id,
                url
            );
        }
    }
}

#[test]
fn all_puzzles_have_valid_chain() {
    for puzzle in boha::all() {
        let valid_chains = [
            Chain::Bitcoin,
            Chain::Ethereum,
            Chain::Litecoin,
            Chain::Monero,
            Chain::Decred,
        ];
        assert!(
            valid_chains.contains(&puzzle.chain),
            "Puzzle {} has invalid chain",
            puzzle.id
        );
    }
}

#[test]
fn b1000_is_bitcoin() {
    for puzzle in b1000::all() {
        assert_eq!(puzzle.chain, Chain::Bitcoin);
    }
}

#[test]
fn hash_collision_is_bitcoin() {
    for puzzle in hash_collision::all() {
        assert_eq!(puzzle.chain, Chain::Bitcoin);
    }
}

#[test]
fn chain_symbol_correct() {
    assert_eq!(Chain::Bitcoin.symbol(), "BTC");
    assert_eq!(Chain::Ethereum.symbol(), "ETH");
    assert_eq!(Chain::Litecoin.symbol(), "LTC");
    assert_eq!(Chain::Monero.symbol(), "XMR");
    assert_eq!(Chain::Decred.symbol(), "DCR");
}

#[test]
fn chain_name_correct() {
    assert_eq!(Chain::Bitcoin.name(), "Bitcoin");
    assert_eq!(Chain::Ethereum.name(), "Ethereum");
    assert_eq!(Chain::Litecoin.name(), "Litecoin");
    assert_eq!(Chain::Monero.name(), "Monero");
    assert_eq!(Chain::Decred.name(), "Decred");
}

#[test]
fn gsmg_has_uncompressed_pubkey() {
    let puzzle = gsmg::get();
    let pubkey = puzzle.pubkey.expect("GSMG should have pubkey");
    assert_eq!(pubkey.format, PubkeyFormat::Uncompressed);
    assert_eq!(
        pubkey.value,
        "04f4d1bbd91e65e2a019566a17574e97dae908b784b388891848007e4f55d5a4649c73d25fc5ed8fd7227cab0be4e576c0c6404db5aa546286563e4be12bf33559"
    );
}

#[test]
fn b1000_pubkeys_are_compressed() {
    for puzzle in b1000::all() {
        if let Some(pubkey) = &puzzle.pubkey {
            assert_eq!(
                pubkey.format,
                PubkeyFormat::Compressed,
                "b1000 puzzle {} should have compressed pubkey",
                puzzle.id
            );
        }
    }
}

#[test]
fn pubkey_format_matches_key_length() {
    for puzzle in boha::all() {
        if let Some(pubkey) = &puzzle.pubkey {
            match pubkey.format {
                PubkeyFormat::Compressed => {
                    assert_eq!(
                        pubkey.value.len(),
                        66,
                        "Compressed pubkey should be 66 hex chars: {}",
                        puzzle.id
                    );
                }
                PubkeyFormat::Uncompressed => {
                    assert_eq!(
                        pubkey.value.len(),
                        130,
                        "Uncompressed pubkey should be 130 hex chars: {}",
                        puzzle.id
                    );
                }
            }
        }
    }
}

#[test]
fn pubkey_has_non_empty_key() {
    for puzzle in boha::all() {
        if let Some(pubkey) = &puzzle.pubkey {
            assert!(
                !pubkey.value.is_empty(),
                "Puzzle {} has empty pubkey",
                puzzle.id
            );
        }
    }
}

#[test]
fn b1000_p2pkh_has_hash160() {
    for puzzle in b1000::all() {
        assert!(
            puzzle.address.hash160.is_some(),
            "b1000 puzzle {} missing hash160",
            puzzle.id
        );
    }
}

#[test]
fn gsmg_has_hash160() {
    let puzzle = gsmg::get();
    assert!(
        puzzle.address.hash160.is_some(),
        "gsmg puzzle missing hash160"
    );
}

#[test]
fn hash_collision_has_hash160() {
    for puzzle in hash_collision::all() {
        assert!(
            puzzle.address.hash160.is_some(),
            "hash_collision puzzle {} should have hash160",
            puzzle.id
        );
    }
}

#[test]
fn hash160_format_valid() {
    let hex_regex = regex::Regex::new(r"^[0-9a-f]{40}$").unwrap();
    for puzzle in boha::all() {
        if let Some(hash160) = puzzle.address.hash160 {
            assert!(
                hex_regex.is_match(hash160),
                "Invalid hash160 format for {}: {} (expected 40 lowercase hex chars)",
                puzzle.id,
                hash160
            );
        }
    }
}

fn address_to_hash160(address: &str) -> Option<String> {
    let decoded = bs58::decode(address).into_vec().ok()?;
    if decoded.len() != 25 {
        return None;
    }
    let hash160 = &decoded[1..21];
    Some(hex::encode(hash160))
}

#[test]
fn hash160_matches_address() {
    for puzzle in boha::all() {
        if matches!(puzzle.address.kind, "p2wpkh" | "p2wsh") {
            continue;
        }
        if let Some(hash160) = puzzle.address.hash160 {
            let computed = address_to_hash160(puzzle.address.value)
                .unwrap_or_else(|| panic!("Failed to compute hash160 for {}", puzzle.id));
            assert_eq!(
                hash160, computed,
                "hash160 mismatch for {}: stored {} != computed {}",
                puzzle.id, hash160, computed
            );
        }
    }
}

#[test]
fn hash_collision_p2sh_has_redeem_script() {
    for puzzle in hash_collision::all() {
        assert!(
            puzzle.address.redeem_script.is_some(),
            "P2SH puzzle {} missing redeem_script",
            puzzle.id
        );
    }
}

#[test]
fn b1000_has_key_with_bits() {
    for puzzle in b1000::all() {
        assert!(
            puzzle.key.and_then(|k| k.bits).is_some(),
            "b1000 puzzle {} should have key with bits",
            puzzle.id
        );
    }
}

#[test]
fn gsmg_has_no_key() {
    let puzzle = gsmg::get();
    assert!(puzzle.key.is_none(), "gsmg puzzle should have no key");
}

#[test]
fn redeem_script_hash_format_valid() {
    let hex_regex = regex::Regex::new(r"^[0-9a-f]{40}$").unwrap();
    for puzzle in boha::all() {
        if let Some(rs) = &puzzle.address.redeem_script {
            assert!(
                hex_regex.is_match(rs.hash),
                "Invalid redeem_script.hash format for {}: {} (expected 40 lowercase hex chars)",
                puzzle.id,
                rs.hash
            );
        }
    }
}

/// Compute HASH160 (SHA256 → RIPEMD160) of hex-encoded data.
fn hash160(hex_data: &str) -> Option<String> {
    use ripemd::Ripemd160;
    use sha2::{Digest, Sha256};

    let bytes = hex::decode(hex_data).ok()?;
    let sha256_hash = Sha256::digest(&bytes);
    let hash160 = Ripemd160::digest(sha256_hash);
    Some(hex::encode(hash160))
}

#[test]
fn redeem_script_hash_matches_script() {
    for puzzle in hash_collision::all() {
        if let Some(rs) = &puzzle.address.redeem_script {
            let computed = hash160(rs.script)
                .unwrap_or_else(|| panic!("Failed to compute script_hash for {}", puzzle.id));
            assert_eq!(
                rs.hash, computed,
                "redeem_script.hash mismatch for {}: stored {} != computed {}",
                puzzle.id, rs.hash, computed
            );
        }
    }
}

#[test]
fn pubkey_matches_hash160() {
    for puzzle in boha::all() {
        if let (Some(pubkey), Some(expected)) = (&puzzle.pubkey, puzzle.address.hash160) {
            let computed = hash160(pubkey.value).unwrap_or_else(|| {
                panic!("Failed to compute hash160 from pubkey for {}", puzzle.id)
            });
            assert_eq!(
                expected, computed,
                "pubkey doesn't match hash160 for {}: stored {} != computed {}",
                puzzle.id, expected, computed
            );
        }
    }
}

#[test]
fn private_key_derives_correct_address() {
    use k256::{elliptic_curve::sec1::ToEncodedPoint, SecretKey};
    use ripemd::Ripemd160;
    use sha2::{Digest, Sha256};

    for puzzle in boha::all() {
        let Some(pk_hex) = puzzle.key.and_then(|k| k.hex) else {
            continue;
        };
        let Some(expected_hash160) = puzzle.address.hash160 else {
            continue;
        };

        let pk_bytes = hex::decode(pk_hex).unwrap();
        let mut padded = [0u8; 32];
        padded[32 - pk_bytes.len()..].copy_from_slice(&pk_bytes);

        let secret_key = SecretKey::from_bytes((&padded).into())
            .unwrap_or_else(|_| panic!("Invalid private key for {}", puzzle.id));
        let public_key = secret_key.public_key();

        let compress = match &puzzle.pubkey {
            Some(pk) => pk.format == PubkeyFormat::Compressed,
            None => true,
        };

        let pubkey_point = public_key.to_encoded_point(compress);
        let sha256_hash = Sha256::digest(pubkey_point.as_bytes());
        let computed_hash160 = hex::encode(Ripemd160::digest(sha256_hash));

        assert_eq!(
            expected_hash160, computed_hash160,
            "Private key doesn't derive correct address for {}: expected {} != computed {}",
            puzzle.id, expected_hash160, computed_hash160
        );
    }
}

#[test]
fn solved_puzzles_with_dates_have_solve_time() {
    for puzzle in boha::all() {
        if matches!(puzzle.status, Status::Solved | Status::Claimed)
            && puzzle.start_date.is_some()
            && puzzle.solve_date.is_some()
        {
            assert!(
                puzzle.solve_time.is_some(),
                "Solved puzzle {} with both start_date and solve_date should have solve_time",
                puzzle.id
            );
        }
    }
}

#[test]
fn unsolved_puzzles_no_solve_time() {
    for puzzle in boha::all() {
        if matches!(puzzle.status, Status::Unsolved | Status::Swept) {
            assert!(
                puzzle.solve_time.is_none(),
                "Unsolved/swept puzzle {} should not have solve_time",
                puzzle.id
            );
        }
    }
}

#[test]
fn solve_time_is_reasonable() {
    const SECONDS_PER_DAY: u64 = 86400;
    const MAX_YEARS: u64 = 15;

    for puzzle in boha::all() {
        if let Some(solve_time) = puzzle.solve_time {
            let max_seconds = MAX_YEARS * 365 * SECONDS_PER_DAY;
            assert!(
                solve_time <= max_seconds,
                "Puzzle {} solve_time {} seconds seems too large (>{} years)",
                puzzle.id,
                solve_time,
                MAX_YEARS
            );
        }
    }
}

#[test]
fn b1000_66_solve_time_correct() {
    let p66 = b1000::get(66).unwrap();
    assert_eq!(p66.start_date, Some("2015-01-15 18:07:14"));
    assert_eq!(p66.solve_date, Some("2024-09-12 22:59:39"));
    assert_eq!(p66.solve_time, Some(304836745));
    let formatted = p66.solve_time_formatted().unwrap();
    assert!(
        formatted.contains('y'),
        "Should contain years: {}",
        formatted
    );
}

#[test]
fn solve_time_formatted_works() {
    let p22 = b1000::get(22).unwrap();
    assert_eq!(p22.solve_time, Some(14891));
    let formatted = p22.solve_time_formatted().unwrap();
    assert!(
        formatted.contains('h'),
        "Should contain hours: {}",
        formatted
    );

    let p66 = b1000::get(66).unwrap();
    assert!(p66.solve_time_formatted().is_some());
    let formatted = p66.solve_time_formatted().unwrap();
    assert!(
        formatted.contains('y'),
        "Should contain years: {}",
        formatted
    );
}

#[test]
fn b1000_has_author() {
    let author = b1000::author();
    assert_eq!(author.name, Some("saatoshi_rising"));
    assert!(!author.addresses.is_empty());
    assert!(!author.profiles.is_empty());
}

#[test]
fn gsmg_has_author() {
    let author = gsmg::author();
    assert_eq!(author.name, Some("GSMG.io"));
    assert!(!author.profiles.is_empty());
}

#[test]
fn hash_collision_has_author() {
    let author = hash_collision::author();
    assert_eq!(author.name, Some("Peter Todd"));
    assert!(!author.addresses.is_empty());
    assert!(!author.profiles.is_empty());
}

#[test]
fn author_addresses_valid_format() {
    fn is_valid_address(addr: &str) -> bool {
        // Base58 (P2PKH: 1..., P2SH: 3...)
        if addr.starts_with('1') || addr.starts_with('3') {
            return bs58::decode(addr).into_vec().is_ok();
        }
        // Bech32 (P2WPKH/P2WSH: bc1...)
        if addr.starts_with("bc1") {
            const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
            let data_part = &addr[3..];
            return addr.len() >= 42 && data_part.chars().all(|c| BECH32_CHARSET.contains(c));
        }
        false
    }

    for addr in b1000::author().addresses {
        assert!(
            is_valid_address(addr),
            "Invalid address in b1000 author: {}",
            addr
        );
    }

    for addr in gsmg::author().addresses {
        assert!(
            is_valid_address(addr),
            "Invalid address in gsmg author: {}",
            addr
        );
    }

    for addr in hash_collision::author().addresses {
        assert!(
            is_valid_address(addr),
            "Invalid address in hash_collision author: {}",
            addr
        );
    }
}

#[test]
fn transaction_txid_format_valid() {
    let btc_regex = regex::Regex::new(r"^[0-9a-f]{64}$").unwrap();
    let eth_regex = regex::Regex::new(r"^0x[0-9a-f]{64}$").unwrap();
    for puzzle in boha::all() {
        for tx in puzzle.transactions {
            if let Some(txid) = tx.txid {
                let valid = match puzzle.chain {
                    Chain::Ethereum => eth_regex.is_match(txid),
                    _ => btc_regex.is_match(txid),
                };
                assert!(
                    valid,
                    "Invalid txid format for {:?} transaction in {}: {}",
                    tx.tx_type, puzzle.id, txid
                );
            }
        }
    }
}

#[test]
fn transaction_date_format_valid() {
    let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
    for puzzle in boha::all() {
        for tx in puzzle.transactions {
            if let Some(date) = tx.date {
                assert!(
                    date_regex.is_match(date),
                    "Invalid date format in {:?} transaction for {}: {}",
                    tx.tx_type,
                    puzzle.id,
                    date
                );
            }
        }
    }
}

#[test]
fn transaction_amount_positive() {
    for puzzle in boha::all() {
        for tx in puzzle.transactions {
            if let Some(amount) = tx.amount {
                assert!(
                    amount > 0.0,
                    "Transaction amount should be positive for {:?} in {}: {}",
                    tx.tx_type,
                    puzzle.id,
                    amount
                );
            }
        }
    }
}

#[test]
fn all_puzzles_have_funding_transaction() {
    for puzzle in boha::all() {
        // Pre-genesis puzzles (like b1000/1) were claimed before the puzzle officially existed
        // and may not have a funding transaction
        if puzzle.pre_genesis {
            continue;
        }
        let has_funding = puzzle
            .transactions
            .iter()
            .any(|t| t.tx_type == TransactionType::Funding);
        assert!(
            has_funding,
            "Puzzle {} missing funding transaction",
            puzzle.id
        );
    }
}

#[test]
fn solved_puzzles_have_claim_transaction() {
    for puzzle in boha::all() {
        if puzzle.status == Status::Solved {
            let has_claim = puzzle
                .transactions
                .iter()
                .any(|t| t.tx_type == TransactionType::Claim);
            assert!(
                has_claim,
                "Solved puzzle {} missing claim transaction",
                puzzle.id
            );
        }
    }
}

#[test]
fn claimed_puzzles_have_claim_transaction() {
    for puzzle in boha::all() {
        if puzzle.status == Status::Claimed {
            let has_claim = puzzle
                .transactions
                .iter()
                .any(|t| t.tx_type == TransactionType::Claim);
            assert!(
                has_claim,
                "Claimed puzzle {} missing claim transaction",
                puzzle.id
            );
        }
    }
}

#[test]
fn swept_puzzles_have_sweep_transaction() {
    for puzzle in boha::all() {
        if puzzle.status == Status::Swept {
            let has_sweep = puzzle
                .transactions
                .iter()
                .any(|t| t.tx_type == TransactionType::Sweep);
            assert!(
                has_sweep,
                "Swept puzzle {} missing sweep transaction",
                puzzle.id
            );
        }
    }
}

#[test]
fn transactions_chronologically_ordered() {
    for puzzle in boha::all() {
        let dates: Vec<&str> = puzzle.transactions.iter().filter_map(|t| t.date).collect();

        for window in dates.windows(2) {
            assert!(
                window[0] <= window[1],
                "Transactions not chronologically ordered in {}: {} > {}",
                puzzle.id,
                window[0],
                window[1]
            );
        }
    }
}

#[test]
fn claim_txid_matches_claim_tx() {
    for puzzle in b1000::solved() {
        let txid = puzzle.claim_txid();
        let from_tx = puzzle.claim_tx().and_then(|t| t.txid);
        assert_eq!(txid, from_tx, "claim_txid() mismatch for {}", puzzle.id);
    }
}

#[test]
fn funding_txid_matches_funding_tx() {
    for puzzle in boha::all() {
        let txid = puzzle.funding_txid();
        let from_tx = puzzle.funding_tx().and_then(|t| t.txid);
        assert_eq!(txid, from_tx, "funding_txid() mismatch for {}", puzzle.id);
    }
}

#[test]
fn start_date_matches_first_funding() {
    for puzzle in boha::all() {
        if puzzle.pre_genesis {
            continue;
        }
        let funding_date = puzzle.funding_tx().and_then(|t| t.date);
        if let (Some(start_date), Some(funding_date)) = (puzzle.start_date, funding_date) {
            assert_eq!(
                start_date, funding_date,
                "Puzzle {} start_date ({}) does not match first funding transaction date ({})",
                puzzle.id, start_date, funding_date
            );
        }
    }
}

#[test]
fn solve_date_matches_claim_transaction() {
    for puzzle in boha::all() {
        if puzzle.pre_genesis {
            continue;
        }
        if !matches!(
            puzzle.status,
            Status::Solved | Status::Claimed | Status::Swept
        ) {
            continue;
        }
        let terminal_tx = puzzle
            .transactions
            .iter()
            .find(|t| matches!(t.tx_type, TransactionType::Claim | TransactionType::Sweep));
        if let (Some(solve_date), Some(tx)) = (puzzle.solve_date, terminal_tx) {
            if let Some(tx_date) = tx.date {
                assert_eq!(
                    solve_date, tx_date,
                    "Puzzle {} solve_date ({}) does not match claim/sweep transaction date ({})",
                    puzzle.id, solve_date, tx_date
                );
            }
        }
    }
}

#[test]
fn unsolved_puzzles_no_solver() {
    for puzzle in boha::all() {
        if matches!(puzzle.status, Status::Unsolved | Status::Swept) {
            assert!(
                puzzle.solver.is_none(),
                "Unsolved/swept puzzle {} should not have solver",
                puzzle.id
            );
        }
    }
}

#[test]
fn solver_name_non_empty() {
    for puzzle in boha::all() {
        if let Some(solver) = &puzzle.solver {
            if let Some(name) = solver.name {
                assert!(
                    !name.is_empty(),
                    "Solver name for {} should not be empty string",
                    puzzle.id
                );
            }
        }
    }
}

#[test]
fn solver_addresses_format_valid() {
    fn is_valid_btc_address(addr: &str) -> bool {
        if addr.starts_with('1') || addr.starts_with('3') {
            return bs58::decode(addr).into_vec().is_ok();
        }
        if addr.starts_with("bc1") {
            const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
            let data_part = &addr[3..];
            return addr.len() >= 42 && data_part.chars().all(|c| BECH32_CHARSET.contains(c));
        }
        false
    }

    for puzzle in boha::all() {
        if let Some(solver) = &puzzle.solver {
            for addr in solver.addresses {
                assert!(
                    is_valid_btc_address(addr),
                    "Invalid solver address for {}: {}",
                    puzzle.id,
                    addr
                );
            }
        }
    }
}

#[test]
fn solver_has_at_least_one_address() {
    for puzzle in boha::all() {
        if let Some(solver) = &puzzle.solver {
            assert!(
                !solver.addresses.is_empty(),
                "Solver for {} should have at least one address",
                puzzle.id
            );
        }
    }
}

#[test]
fn solver_profiles_valid_url() {
    for puzzle in boha::all() {
        if let Some(solver) = &puzzle.solver {
            for profile in solver.profiles {
                assert!(
                    profile.url.starts_with("http://") || profile.url.starts_with("https://"),
                    "Invalid solver profile URL for {}: {}",
                    puzzle.id,
                    profile.url
                );
            }
        }
    }
}

#[test]
fn author_profiles_valid_url() {
    let authors = [
        boha::b1000::author(),
        boha::zden::author(),
        boha::hash_collision::author(),
        boha::gsmg::author(),
        boha::bitaps::author(),
        boha::bitimage::author(),
    ];

    for author in authors {
        for profile in author.profiles {
            assert!(
                profile.url.starts_with("http://") || profile.url.starts_with("https://"),
                "Invalid author profile URL: {}",
                profile.url
            );
        }
    }
}

#[test]
fn unsolved_puzzles_no_claim_txid() {
    for puzzle in b1000::unsolved() {
        assert!(
            puzzle.claim_txid().is_none(),
            "Unsolved puzzle {} should not have claim_txid",
            puzzle.id
        );
    }
}

#[test]
fn tx_explorer_url_format() {
    assert_eq!(
        Chain::Bitcoin.tx_explorer_url("abc123"),
        "https://mempool.space/tx/abc123"
    );
    assert_eq!(
        Chain::Ethereum.tx_explorer_url("0xdef456"),
        "https://etherscan.io/tx/0xdef456"
    );
    assert_eq!(
        Chain::Litecoin.tx_explorer_url("abc"),
        "https://blockchair.com/litecoin/transaction/abc"
    );
    assert_eq!(
        Chain::Monero.tx_explorer_url("xyz"),
        "https://xmrchain.net/tx/xyz"
    );
    assert_eq!(
        Chain::Decred.tx_explorer_url("dcr123"),
        "https://dcrdata.decred.org/tx/dcr123"
    );
}

#[test]
fn zden_count() {
    assert_eq!(zden::all().count(), 15);
}

#[test]
fn zden_get_by_name() {
    let level1 = zden::get("Level 1").unwrap();
    assert_eq!(level1.address.value, "1cryptommoqPHVNHuxVQG3bzujnRJYB1D");
    assert_eq!(level1.status, Status::Solved);
    assert_eq!(level1.chain, Chain::Bitcoin);

    let xixoio = zden::get("XIXOIO").unwrap();
    assert_eq!(xixoio.chain, Chain::Ethereum);

    let ltc = zden::get("Litecoin SegWit").unwrap();
    assert_eq!(ltc.chain, Chain::Litecoin);
}

#[test]
fn zden_has_author() {
    let author = zden::author();
    assert_eq!(author.name, Some("Zden"));
    assert!(!author.addresses.is_empty());
    assert!(!author.profiles.is_empty());
}

#[test]
fn zden_multi_chain() {
    let chains: Vec<Chain> = zden::all().map(|p| p.chain).collect();
    assert!(chains.contains(&Chain::Bitcoin));
    assert!(chains.contains(&Chain::Ethereum));
    assert!(chains.contains(&Chain::Litecoin));
    assert!(chains.contains(&Chain::Decred));
}

#[test]
fn zden_btc_ltc_have_hash160() {
    for puzzle in zden::all() {
        if puzzle.chain == Chain::Bitcoin || puzzle.chain == Chain::Litecoin {
            assert!(
                puzzle.address.hash160.is_some(),
                "Zden BTC/LTC puzzle {} should have hash160",
                puzzle.id
            );
        }
    }
}

#[test]
fn zden_eth_dcr_no_hash160() {
    for puzzle in zden::all() {
        if puzzle.chain == Chain::Ethereum || puzzle.chain == Chain::Decred {
            assert!(
                puzzle.address.hash160.is_none(),
                "Zden ETH/DCR puzzle {} should not have hash160",
                puzzle.id
            );
        }
    }
}

#[test]
fn universal_get_works_with_zden() {
    assert!(boha::get("zden/Level 1").is_ok());
    assert!(boha::get("zden/XIXOIO").is_ok());
    assert!(boha::get("zden/Litecoin SegWit").is_ok());
}

#[test]
fn bech32_address_has_segwit_kind() {
    for puzzle in boha::all() {
        if puzzle.address.value.starts_with("bc1q") || puzzle.address.value.starts_with("ltc1q") {
            assert!(
                matches!(puzzle.address.kind, "p2wpkh" | "p2wsh"),
                "bc1q/ltc1q address {} should have kind p2wpkh or p2wsh, got {}",
                puzzle.id,
                puzzle.address.kind
            );
        }
    }
}

#[test]
fn p2wpkh_has_hash160() {
    for puzzle in boha::all() {
        if puzzle.address.kind == "p2wpkh" {
            assert!(
                puzzle.address.hash160.is_some(),
                "P2WPKH puzzle {} missing hash160",
                puzzle.id
            );
        }
    }
}

fn decode_bech32_witness_program(address: &str) -> Option<(u8, Vec<u8>)> {
    let (_hrp, version, program) = bech32::segwit::decode(address).ok()?;
    Some((version.to_u8(), program))
}

#[test]
fn p2wpkh_hash160_matches_witness_program() {
    for puzzle in boha::all() {
        if puzzle.address.kind == "p2wpkh" {
            let Some(hash160) = puzzle.address.hash160 else {
                continue;
            };
            let Some((version, witness_program)) =
                decode_bech32_witness_program(puzzle.address.value)
            else {
                panic!("Failed to decode bech32 address for {}", puzzle.id);
            };
            assert_eq!(
                version, 0,
                "P2WPKH {} should have witness version 0, got {}",
                puzzle.id, version
            );
            assert_eq!(
                witness_program.len(),
                20,
                "P2WPKH {} witness program should be 20 bytes, got {}",
                puzzle.id,
                witness_program.len()
            );
            let computed_hash160 = hex::encode(&witness_program);
            assert_eq!(
                hash160, computed_hash160,
                "P2WPKH {} hash160 mismatch: stored {} != computed {}",
                puzzle.id, hash160, computed_hash160
            );
        }
    }
}

#[test]
fn p2wsh_has_witness_program() {
    for puzzle in boha::all() {
        if puzzle.address.kind == "p2wsh" {
            assert!(
                puzzle.address.witness_program.is_some(),
                "P2WSH puzzle {} missing witness_program",
                puzzle.id
            );
        }
    }
}

#[test]
fn witness_program_format_valid() {
    let hex_regex = regex::Regex::new(r"^[0-9a-f]{64}$").unwrap();
    for puzzle in boha::all() {
        if let Some(wp) = puzzle.address.witness_program {
            assert!(
                hex_regex.is_match(wp),
                "Invalid witness_program format for {}: {} (expected 64 lowercase hex chars)",
                puzzle.id,
                wp
            );
        }
    }
}

#[test]
fn p2wsh_witness_program_matches_address() {
    for puzzle in boha::all() {
        if puzzle.address.kind == "p2wsh" {
            let Some(witness_program) = puzzle.address.witness_program else {
                continue;
            };
            let Some((version, decoded_wp)) = decode_bech32_witness_program(puzzle.address.value)
            else {
                panic!("Failed to decode bech32 address for {}", puzzle.id);
            };
            assert_eq!(
                version, 0,
                "P2WSH {} should have witness version 0, got {}",
                puzzle.id, version
            );
            assert_eq!(
                decoded_wp.len(),
                32,
                "P2WSH {} witness program should be 32 bytes, got {}",
                puzzle.id,
                decoded_wp.len()
            );
            let computed_wp = hex::encode(&decoded_wp);
            assert_eq!(
                witness_program, computed_wp,
                "P2WSH {} witness_program mismatch: stored {} != computed {}",
                puzzle.id, witness_program, computed_wp
            );
        }
    }
}

#[test]
fn bech32m_address_has_taproot_kind() {
    for puzzle in boha::all() {
        if puzzle.address.value.starts_with("bc1p") {
            assert_eq!(
                puzzle.address.kind, "p2tr",
                "bc1p address {} should have kind p2tr, got {}",
                puzzle.id, puzzle.address.kind
            );
        }
    }
}

#[test]
fn p2tr_has_witness_program() {
    for puzzle in boha::all() {
        if puzzle.address.kind == "p2tr" {
            assert!(
                puzzle.address.witness_program.is_some(),
                "P2TR puzzle {} missing witness_program (x-only pubkey)",
                puzzle.id
            );
        }
    }
}

#[test]
fn p2tr_no_hash160() {
    for puzzle in boha::all() {
        if puzzle.address.kind == "p2tr" {
            assert!(
                puzzle.address.hash160.is_none(),
                "P2TR puzzle {} should not have hash160 (uses x-only pubkey instead)",
                puzzle.id
            );
        }
    }
}

#[test]
fn p2tr_witness_program_matches_address() {
    for puzzle in boha::all() {
        if puzzle.address.kind == "p2tr" {
            let Some(witness_program) = puzzle.address.witness_program else {
                continue;
            };
            let Some((version, decoded_wp)) = decode_bech32_witness_program(puzzle.address.value)
            else {
                panic!("Failed to decode bech32m address for {}", puzzle.id);
            };
            assert_eq!(
                version, 1,
                "P2TR {} should have witness version 1, got {}",
                puzzle.id, version
            );
            assert_eq!(
                decoded_wp.len(),
                32,
                "P2TR {} witness program (x-only pubkey) should be 32 bytes, got {}",
                puzzle.id,
                decoded_wp.len()
            );
            let computed_wp = hex::encode(&decoded_wp);
            assert_eq!(
                witness_program, computed_wp,
                "P2TR {} witness_program mismatch: stored {} != computed {}",
                puzzle.id, witness_program, computed_wp
            );
        }
    }
}

#[test]
fn decode_bech32m_taproot_address() {
    let taproot_address = "bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxg3297";
    let (version, witness_program) = decode_bech32_witness_program(taproot_address)
        .expect("Should decode valid Taproot address");

    assert_eq!(version, 1, "Taproot should have witness version 1");
    assert_eq!(
        witness_program.len(),
        32,
        "Taproot witness program should be 32 bytes"
    );
    assert_eq!(
        hex::encode(&witness_program),
        "a37c3903c8d0db6512e2b40b0dffa05e5a3ab73603ce8c9c4b7771e5412328f9"
    );
}

#[test]
fn bitaps_count() {
    assert_eq!(bitaps::all().count(), 1);
}

#[test]
fn bitaps_get_returns_correct_puzzle() {
    let puzzle = bitaps::get();
    assert_eq!(puzzle.id, "bitaps");
    assert_eq!(
        puzzle.address.value,
        "bc1qyjwa0tf0en4x09magpuwmt2smpsrlaxwn85lh6"
    );
    assert_eq!(puzzle.status, Status::Unsolved);
    assert_eq!(puzzle.address.kind, "p2wpkh");
    assert_eq!(puzzle.chain, Chain::Bitcoin);
}

#[test]
fn bitaps_has_author() {
    let author = bitaps::author();
    assert_eq!(author.name, Some("Bitaps"));
    assert!(!author.profiles.is_empty());
}

#[test]
fn bitaps_has_shares() {
    let puzzle = bitaps::get();
    let key = puzzle.key.expect("bitaps should have key");
    let shares = key.shares.expect("bitaps key should have shares");
    assert_eq!(shares.threshold, 3);
    assert_eq!(shares.total, 5);
    assert_eq!(shares.shares.len(), 2);
}

#[test]
fn bitaps_has_seed_path() {
    let puzzle = bitaps::get();
    let key = puzzle.key.expect("bitaps should have key");
    let seed = key.seed.expect("bitaps key should have seed");
    assert_eq!(seed.phrase, None);
    assert_eq!(seed.path, Some("m/84'/0'/0'/0/0"));
    assert!(seed.xpub.unwrap().starts_with("zpub"));
}

#[test]
fn bitaps_shares_valid() {
    let puzzle = bitaps::get();
    let shares = puzzle.key.unwrap().shares.unwrap();
    for share in shares.shares {
        assert!(share.index > 0, "Share index should be 1-based");
        assert!(!share.data.is_empty(), "Share data should not be empty");
        let word_count = share.data.split_whitespace().count();
        assert_eq!(word_count, 12, "Each share should have 12 words");
    }
}

#[test]
fn universal_get_works_with_bitaps() {
    assert!(boha::get("bitaps").is_ok());
}

#[test]
fn bitimage_count() {
    assert_eq!(bitimage::all().count(), 2);
}

#[test]
fn bitimage_get_by_name() {
    let kitten = bitimage::get("kitten").unwrap();
    assert_eq!(
        kitten.address.value,
        "bc1q57euh23y3qs2f9d5mtwpax5lqecfvrdkqce82a"
    );
    assert_eq!(kitten.status, Status::Solved);
    assert_eq!(kitten.chain, Chain::Bitcoin);

    let passphrase = bitimage::get("kitten_passphrase").unwrap();
    assert_eq!(
        passphrase.address.value,
        "bc1qcyrndzgy036f6ax370g8zyvlw86ulawgt0246r"
    );
    assert_eq!(passphrase.status, Status::Unsolved);
}

#[test]
fn bitimage_has_author() {
    let author = bitimage::author();
    assert_eq!(author.name, Some("Corey Phillips"));
    assert!(!author.profiles.is_empty());
}

#[test]
fn bitimage_has_entropy() {
    for puzzle in bitimage::all() {
        let key = puzzle.key.expect("bitimage should have key");
        let seed = key.seed.expect("bitimage key should have seed");
        let entropy = seed.entropy.expect("bitimage seed should have entropy");
        assert_eq!(
            entropy.hash,
            "1808d35318ac7cb98b69ff9779b699d6a631f15e0b353ac89b7c4020774832ed"
        );
        assert!(seed.path.is_some());
    }
}

#[test]
fn bitimage_passphrase_puzzle_has_required_flag() {
    let puzzle = bitimage::get("kitten_passphrase").unwrap();
    let seed = puzzle.key.unwrap().seed.unwrap();
    let entropy = seed.entropy.unwrap();
    assert!(matches!(entropy.passphrase, Some(Passphrase::Required)));
}

#[test]
fn bitimage_no_passphrase_puzzle_has_none() {
    let puzzle = bitimage::get("kitten").unwrap();
    let seed = puzzle.key.unwrap().seed.unwrap();
    let entropy = seed.entropy.unwrap();
    assert!(entropy.passphrase.is_none());
}

#[test]
fn bitimage_entropy_has_source() {
    for puzzle in bitimage::all() {
        let entropy = puzzle.key.unwrap().seed.unwrap().entropy.unwrap();
        let source = entropy.source.expect("bitimage entropy should have source");
        assert!(source.url.is_some());
        assert!(source.description.is_some());
    }
}

#[test]
fn universal_get_works_with_bitimage() {
    assert!(boha::get("bitimage/kitten").is_ok());
    assert!(boha::get("bitimage/kitten_passphrase").is_ok());
}

// ============================================================================
// Assets validation tests
// ============================================================================

#[test]
fn zden_puzzles_have_assets() {
    for puzzle in zden::all() {
        assert!(
            puzzle.assets.is_some(),
            "Zden puzzle {} should have assets",
            puzzle.id
        );
    }
}

#[test]
fn gsmg_has_assets() {
    let puzzle = gsmg::get();
    assert!(puzzle.assets.is_some(), "GSMG puzzle should have assets");
    let assets = puzzle.assets.unwrap();
    assert_eq!(assets.puzzle, Some("puzzle.png"));
    assert!(!assets.hints.is_empty());
    assert!(assets.source_url.is_some());
}

#[test]
fn bitimage_puzzles_have_assets() {
    for puzzle in bitimage::all() {
        assert!(
            puzzle.assets.is_some(),
            "Bitimage puzzle {} should have assets",
            puzzle.id
        );
    }
}

#[test]
fn b1000_puzzles_have_no_assets() {
    for puzzle in b1000::all() {
        assert!(
            puzzle.assets.is_none(),
            "b1000 puzzle {} should not have assets",
            puzzle.id
        );
    }
}

#[test]
fn hash_collision_puzzles_have_no_assets() {
    for puzzle in hash_collision::all() {
        assert!(
            puzzle.assets.is_none(),
            "hash_collision puzzle {} should not have assets",
            puzzle.id
        );
    }
}

#[test]
fn bitaps_has_no_assets() {
    let puzzle = bitaps::get();
    assert!(
        puzzle.assets.is_none(),
        "bitaps puzzle should not have assets"
    );
}

#[test]
fn assets_puzzle_path_exists() {
    for puzzle in boha::all() {
        if let Some(path) = puzzle.asset_path() {
            let full_path = std::path::Path::new(&path);
            assert!(
                full_path.exists(),
                "Asset file {} does not exist for puzzle {}",
                full_path.display(),
                puzzle.id
            );
        }
    }
}

#[test]
fn assets_hints_paths_exist() {
    for puzzle in boha::all() {
        if let Some(assets) = &puzzle.assets {
            for hint in assets.hints {
                let path = format!("assets/{}/{}", puzzle.collection(), hint);
                let full_path = std::path::Path::new(&path);
                assert!(
                    full_path.exists(),
                    "Hint asset {} does not exist for puzzle {}",
                    full_path.display(),
                    puzzle.id
                );
            }
        }
    }
}

#[test]
fn assets_source_url_valid() {
    for puzzle in boha::all() {
        if let Some(assets) = &puzzle.assets {
            if let Some(url) = assets.source_url {
                assert!(
                    url.starts_with("http://") || url.starts_with("https://"),
                    "Invalid asset source_url for {}: {}",
                    puzzle.id,
                    url
                );
            }
        }
    }
}

#[test]
fn asset_url_format() {
    let puzzle = gsmg::get();
    let url = puzzle.asset_url().expect("GSMG should have asset_url");
    assert!(url.contains("raw.githubusercontent.com"));
    assert!(url.contains("oritwoen/boha"));
    assert!(url.ends_with("puzzle.png"));
}

#[test]
fn asset_path_format() {
    let puzzle = gsmg::get();
    let path = puzzle.asset_path().expect("GSMG should have asset_path");
    assert!(path.contains("gsmg"));
    assert!(path.ends_with("puzzle.png"));

    let kitten = bitimage::get("kitten").unwrap();
    let path = kitten.asset_path().expect("kitten should have asset_path");
    assert!(path.contains("bitimage"));
}

#[test]
fn claimed_puzzles_have_pubkey() {
    for puzzle in boha::all() {
        // Skip unsolved puzzles - they don't need pubkey
        if puzzle.status == Status::Unsolved {
            continue;
        }

        // Skip P2SH addresses (hash_collision puzzles use redeem_script instead)
        if puzzle.address.kind == "p2sh" {
            continue;
        }

        // Check if puzzle has a Claim or Sweep transaction
        let has_claim_or_sweep = puzzle
            .transactions
            .iter()
            .any(|tx| matches!(tx.tx_type, TransactionType::Claim | TransactionType::Sweep));

        // If status is Solved/Claimed/Swept and has claim/sweep tx, pubkey MUST exist
        if has_claim_or_sweep {
            assert!(
                puzzle.pubkey.is_some(),
                "Puzzle {} has status {:?} with claim/sweep tx but no pubkey",
                puzzle.id,
                puzzle.status
            );
        }
    }
}
