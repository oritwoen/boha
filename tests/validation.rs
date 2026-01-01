use boha::{b1000, gsmg, hash_collision, AddressType, Chain, KeySource, PubkeyFormat, Status};
use num_bigint::BigUint;

#[test]
fn b1000_has_256_puzzles() {
    assert_eq!(b1000::all().count(), 256);
}

#[test]
fn b1000_puzzles_have_sequential_bits() {
    let bits: Vec<u16> = b1000::all()
        .filter_map(|p| match p.key_source {
            KeySource::Direct { bits } => Some(bits),
            _ => None,
        })
        .collect();

    for i in 1u16..=256 {
        assert!(bits.contains(&i), "Missing puzzle with bits={}", i);
    }
}

#[test]
fn b1000_get_returns_correct_puzzle() {
    let p1 = b1000::get(1).unwrap();
    assert_eq!(p1.key_source, KeySource::Direct { bits: 1 });
    assert_eq!(p1.address, "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH");
    assert_eq!(p1.status, Status::Solved);

    let p66 = b1000::get(66).unwrap();
    assert_eq!(p66.key_source, KeySource::Direct { bits: 66 });
    assert_eq!(p66.address, "13zb1hQbWVsc2S7ZTZnP2G4undNNpdh5so");

    let p256 = b1000::get(256).unwrap();
    assert_eq!(p256.key_source, KeySource::Direct { bits: 256 });
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
fn puzzle_key_range_none_for_script_source() {
    for puzzle in hash_collision::all() {
        assert!(
            matches!(puzzle.key_source, KeySource::Script { .. }),
            "hash_collision should have Script key_source"
        );
        assert!(puzzle.key_range().is_none());
        assert!(puzzle.key_range_big().is_none());
    }
}

#[test]
fn solved_puzzles_private_key_in_range() {
    for puzzle in b1000::solved() {
        let pk_hex = puzzle.private_key.unwrap();
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
            puzzle.address.starts_with('1'),
            "BTC1000 address should start with 1: {}",
            puzzle.address
        );
        assert!(
            bs58::decode(puzzle.address).into_vec().is_ok(),
            "Invalid base58: {}",
            puzzle.address
        );
    }
}

#[test]
fn b1000_solved_have_private_keys() {
    for puzzle in b1000::solved() {
        assert!(
            puzzle.private_key.is_some(),
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
    assert_eq!(sha1.address, "37k7toV1Nv4DfmQbmZ8KuZDQCYK9x5KpzP");
    assert_eq!(sha1.status, Status::Claimed);

    let sha256 = hash_collision::get("sha256").unwrap();
    assert_eq!(sha256.status, Status::Unsolved);
}

#[test]
fn hash_collision_all_p2sh() {
    for puzzle in hash_collision::all() {
        assert_eq!(puzzle.address_type, Some(AddressType::P2SH));
        assert!(
            matches!(puzzle.key_source, KeySource::Script { .. }),
            "hash_collision should have Script key_source"
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
    assert_eq!(puzzle.address, "1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe");
    assert_eq!(puzzle.status, Status::Unsolved);
    assert_eq!(puzzle.address_type, Some(AddressType::P2PKH));
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
    assert!(stats.total > 250);
    assert!(stats.solved > 50);
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
fn start_date_format_valid() {
    let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}( \d{2}:\d{2}:\d{2})?$").unwrap();
    for puzzle in boha::all() {
        if let Some(date) = puzzle.start_date {
            assert!(
                date_regex.is_match(date),
                "Invalid start_date format for {}: {}",
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
fn all_current_puzzles_are_bitcoin() {
    for puzzle in boha::all() {
        assert_eq!(
            puzzle.chain,
            Chain::Bitcoin,
            "Puzzle {} should be Bitcoin",
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
        pubkey.key,
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
                        pubkey.key.len(),
                        66,
                        "Compressed pubkey should be 66 hex chars: {}",
                        puzzle.id
                    );
                }
                PubkeyFormat::Uncompressed => {
                    assert_eq!(
                        pubkey.key.len(),
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
                !pubkey.key.is_empty(),
                "Puzzle {} has empty pubkey",
                puzzle.id
            );
        }
    }
}

#[test]
fn b1000_p2pkh_has_h160() {
    for puzzle in b1000::all() {
        assert!(
            puzzle.h160.is_some(),
            "b1000 puzzle {} missing h160",
            puzzle.id
        );
    }
}

#[test]
fn gsmg_has_h160() {
    let puzzle = gsmg::get();
    assert!(puzzle.h160.is_some(), "gsmg puzzle missing h160");
}

#[test]
fn hash_collision_no_h160() {
    for puzzle in hash_collision::all() {
        assert!(
            puzzle.h160.is_none(),
            "hash_collision puzzle {} should not have h160 (P2SH)",
            puzzle.id
        );
    }
}

#[test]
fn h160_format_valid() {
    let hex_regex = regex::Regex::new(r"^[0-9a-f]{40}$").unwrap();
    for puzzle in boha::all() {
        if let Some(h160) = puzzle.h160 {
            assert!(
                hex_regex.is_match(h160),
                "Invalid h160 format for {}: {} (expected 40 lowercase hex chars)",
                puzzle.id,
                h160
            );
        }
    }
}

fn address_to_h160(address: &str) -> Option<String> {
    let decoded = bs58::decode(address).into_vec().ok()?;
    if decoded.len() != 25 {
        return None;
    }
    let h160 = &decoded[1..21];
    Some(hex::encode(h160))
}

#[test]
fn h160_matches_address() {
    for puzzle in boha::all() {
        if let Some(h160) = puzzle.h160 {
            let computed = address_to_h160(puzzle.address)
                .unwrap_or_else(|| panic!("Failed to compute h160 for {}", puzzle.id));
            assert_eq!(
                h160, computed,
                "h160 mismatch for {}: stored {} != computed {}",
                puzzle.id, h160, computed
            );
        }
    }
}

#[test]
fn hash_collision_p2sh_has_script_hash() {
    for puzzle in hash_collision::all() {
        match &puzzle.key_source {
            KeySource::Script { script_hash, .. } => {
                assert!(
                    script_hash.is_some(),
                    "P2SH puzzle {} missing script_hash",
                    puzzle.id
                );
            }
            _ => panic!(
                "hash_collision puzzle {} should have Script key_source",
                puzzle.id
            ),
        }
    }
}

#[test]
fn b1000_has_direct_key_source() {
    for puzzle in b1000::all() {
        assert!(
            matches!(puzzle.key_source, KeySource::Direct { .. }),
            "b1000 puzzle {} should have Direct key_source",
            puzzle.id
        );
    }
}

#[test]
fn gsmg_has_unknown_key_source() {
    let puzzle = gsmg::get();
    assert_eq!(
        puzzle.key_source,
        KeySource::Unknown,
        "gsmg puzzle should have Unknown key_source"
    );
}

#[test]
fn script_hash_format_valid() {
    let hex_regex = regex::Regex::new(r"^[0-9a-f]{40}$").unwrap();
    for puzzle in boha::all() {
        if let KeySource::Script {
            script_hash: Some(script_hash),
            ..
        } = &puzzle.key_source
        {
            assert!(
                hex_regex.is_match(script_hash),
                "Invalid script_hash format for {}: {} (expected 40 lowercase hex chars)",
                puzzle.id,
                script_hash
            );
        }
    }
}

fn redeem_script_to_script_hash(redeem_script_hex: &str) -> Option<String> {
    use ripemd::Ripemd160;
    use sha2::{Digest, Sha256};

    let script_bytes = hex::decode(redeem_script_hex).ok()?;
    let sha256_hash = Sha256::digest(&script_bytes);
    let hash160 = Ripemd160::digest(sha256_hash);
    Some(hex::encode(hash160))
}

#[test]
fn script_hash_matches_redeem_script() {
    for puzzle in hash_collision::all() {
        if let KeySource::Script {
            redeem_script,
            script_hash: Some(script_hash),
        } = &puzzle.key_source
        {
            let computed = redeem_script_to_script_hash(redeem_script)
                .unwrap_or_else(|| panic!("Failed to compute script_hash for {}", puzzle.id));
            assert_eq!(
                *script_hash, computed,
                "script_hash mismatch for {}: stored {} != computed {}",
                puzzle.id, script_hash, computed
            );
        }
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
    assert_eq!(p66.solve_date, Some("2025-02-19 04:19:52"));
    assert_eq!(p66.solve_time, Some(318593558));
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
    assert!(author.profile.is_some());
}

#[test]
fn gsmg_has_author() {
    let author = gsmg::author();
    assert_eq!(author.name, Some("GSMG.io"));
    assert!(author.profile.is_some());
}

#[test]
fn hash_collision_has_author() {
    let author = hash_collision::author();
    assert_eq!(author.name, Some("Peter Todd"));
    assert!(!author.addresses.is_empty());
    assert!(author.profile.is_some());
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
            return addr.len() >= 42 && addr.chars().all(|c| c.is_ascii_alphanumeric());
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
fn author_profile_valid_url() {
    let authors = [b1000::author(), gsmg::author(), hash_collision::author()];

    for author in authors {
        if let Some(url) = author.profile {
            assert!(
                url.starts_with("http://") || url.starts_with("https://"),
                "Invalid profile URL format: {}",
                url
            );
        }
    }
}
