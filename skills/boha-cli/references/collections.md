# Boha Collections

## b1000

[Bitcoin Puzzle Transaction](https://privatekeys.pw/puzzles/bitcoin-puzzle-tx) - 256 puzzles. Puzzle N has private key in `[2^(N-1), 2^N - 1]`. 82 solved (1-70, 75, 80, 85, 90, 95, 100, 105, 110, 115, 120, 125, 130). 6 unsolved with known pubkey (135, 140, 145, 150, 155, 160). Created by unknown author in 2015.

```rust
let p = b1000::get(66).unwrap();  // accepts u32, usize, or &str
let range = p.key_range().unwrap();
```

## arweave

Tiamat's Arweave bounties (chronobot.io). 11 puzzles on Arweave blockchain. 7 claimed, 4 unsolved.

```rust
let p = arweave::get("weave1").unwrap();
```

## ballet

Bobby Lee's Ballet Crypto Puzzles. Physical Bitcoin notes with BIP38-encrypted private keys. 3 puzzles. 1 solved, 2 unsolved.

```rust
let p = ballet::get("AA007448").unwrap();
```

## bitaps

Shamir Secret Sharing puzzle. Single puzzle. 3-of-5 threshold, 2 shares published.

```rust
let p = bitaps::get();  // no argument, single puzzle
```

## bitimage

Keys derived from files using SHA256(Base64(file)) as BIP39 entropy. 2 puzzles (kitten, kitten_passphrase).

```rust
let p = bitimage::get("kitten").unwrap();
```

## gsmg

GSMG.IO 5 BTC puzzle. Single multi-phase cryptographic challenge. Prize halves with each Bitcoin halving.

```rust
let p = gsmg::get();  // no argument, single puzzle
```

## hash_collision

Peter Todd's P2SH hash collision bounties. 6 puzzles (sha1, sha256, ripemd160, hash160, hash256, op_abs). SHA-1 claimed in 2017. Also accessible via `peter_todd/` prefix.

```rust
let p = hash_collision::get("sha256").unwrap();
let p = boha::get("peter_todd/sha256").unwrap();  // alias works
```

## zden

Visual crypto puzzles by Zden. Private keys encoded in images, animations, visual patterns. 15 puzzles across Bitcoin, Ethereum, Litecoin, Decred. 2 unsolved (level_5, level_halv).

```rust
let p = zden::get("level_4").unwrap();
let img = p.asset_url().unwrap();  // GitHub raw URL to puzzle image
```
