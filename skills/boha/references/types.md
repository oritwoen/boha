# Boha Types Reference

## Puzzle

```rust
pub struct Puzzle {
    pub id: &'static str,              // "b1000/66", "gsmg"
    pub chain: Chain,
    pub address: Address,
    pub status: Status,
    pub pubkey: Option<Pubkey>,
    pub key: Option<Key>,              // private key (if solved)
    pub prize: Option<f64>,
    pub start_date: Option<&'static str>,
    pub solve_date: Option<&'static str>,
    pub solve_time: Option<u64>,       // seconds
    pub pre_genesis: bool,
    pub source_url: Option<&'static str>,
    pub transactions: &'static [Transaction],
    pub solver: Option<Solver>,
    pub assets: Option<Assets>,
    pub entropy: Option<Entropy>,
}
```

### Methods

```rust
// Identity
puzzle.collection() -> &str      // "b1000"
puzzle.name() -> &str            // "66"

// Checks
puzzle.has_pubkey() -> bool
puzzle.has_private_key() -> bool
puzzle.has_transactions() -> bool

// Transactions
puzzle.funding_tx() -> Option<&Transaction>
puzzle.claim_tx() -> Option<&Transaction>
puzzle.funding_txid() -> Option<&str>
puzzle.claim_txid() -> Option<&str>
puzzle.transaction_count() -> usize

// Key range (b1000 only)
puzzle.key_range() -> Option<RangeInclusive<u128>>      // bits <= 128
puzzle.key_range_big() -> Option<(BigUint, BigUint)>    // bits <= 256

// Assets & URLs
puzzle.asset_path() -> Option<String>
puzzle.asset_url() -> Option<String>
puzzle.explorer_url() -> String
puzzle.solve_time_formatted() -> Option<String>
puzzle.solver_name() -> Option<&str>
```

## Address

```rust
pub struct Address {
    pub value: &'static str,
    pub chain: Chain,
    pub kind: &'static str,            // "p2pkh", "p2sh", "p2wpkh", "p2wsh", "p2tr", "standard"
    pub hash160: Option<&'static str>,
    pub witness_program: Option<&'static str>,
    pub redeem_script: Option<RedeemScript>,
}
```

## Key

```rust
pub struct Key {
    pub hex: Option<&'static str>,     // 64-char hex
    pub wif: Option<Wif>,
    pub seed: Option<Seed>,            // BIP39 mnemonic
    pub mini: Option<&'static str>,    // Mini private key (starts with 'S')
    pub bits: Option<u16>,             // bit range constraint
    pub shares: Option<Shares>,        // Shamir secret sharing
}
```

## Chain

```rust
pub enum Chain {
    Bitcoin, Ethereum, Litecoin, Monero, Decred, Arweave,
}

chain.symbol() -> &str           // "BTC", "ETH", "LTC", "XMR", "DCR", "AR"
chain.name() -> &str             // "Bitcoin", "Ethereum", ...
chain.tx_explorer_url(txid) -> String
chain.address_explorer_url(addr) -> String
chain.is_valid_txid(txid) -> bool
```

## Status

```rust
pub enum Status {
    Solved,    // private key known, funds claimed
    Unsolved,  // active bounty
    Claimed,   // funds claimed but unclear solver
    Swept,     // abandoned/inactivity timeout
}
```

Both `Chain` and `Status` implement `Display` and `FromStr`.

## Transaction

```rust
pub struct Transaction {
    pub r#type: TransactionType,       // Funding, Increase, Decrease, Sweep, Claim, PubkeyReveal
    pub txid: &'static str,
    pub date: Option<&'static str>,
    pub amount: Option<f64>,
}
```

## Collection API

Each collection module exports:

```rust
pub fn author() -> &'static Author
pub fn get(key) -> Result<&'static Puzzle>
pub fn all() -> impl Iterator<Item = &'static Puzzle>
pub fn solved() -> impl Iterator<Item = &'static Puzzle>
pub fn unsolved() -> impl Iterator<Item = &'static Puzzle>
pub fn count() -> usize
```

Some collections also have `with_pubkey()`, `solved_count()`, `unsolved_count()`.
