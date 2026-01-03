use boha::{
    b1000, bitaps, bitimage, gsmg, hash_collision, zden, Author, Chain, PubkeyFormat, Puzzle,
    Stats, Status, TransactionType,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashMap;

fn parse_chain(s: &str) -> Result<Chain, String> {
    match s.to_lowercase().as_str() {
        "bitcoin" | "btc" => Ok(Chain::Bitcoin),
        "ethereum" | "eth" => Ok(Chain::Ethereum),
        "litecoin" | "ltc" => Ok(Chain::Litecoin),
        "monero" | "xmr" => Ok(Chain::Monero),
        "decred" | "dcr" => Ok(Chain::Decred),
        _ => Err(format!(
            "Unknown chain: {}. Use: bitcoin, ethereum, litecoin, monero, decred",
            s
        )),
    }
}
use owo_colors::OwoColorize;
use serde::Serialize;
use tabled::{settings::Style, Table, Tabled};

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Table,
    Json,
    Jsonl,
    Yaml,
    Csv,
}

#[derive(Parser)]
#[command(name = "boha")]
#[command(about = "Crypto bounties, puzzles and challenges data")]
#[command(version)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "table", global = true)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List puzzles from a collection
    List {
        #[arg(default_value = "all")]
        collection: String,

        #[arg(long)]
        unsolved: bool,

        #[arg(long)]
        solved: bool,

        #[arg(long, name = "with-pubkey")]
        with_pubkey: bool,

        #[arg(long, name = "with-transactions")]
        with_transactions: bool,

        #[arg(long, value_parser = parse_chain)]
        chain: Option<Chain>,
    },

    /// Show puzzle details
    Show {
        id: String,

        #[arg(long)]
        transactions: bool,
    },

    /// Show statistics
    Stats,

    /// Show key range for puzzle
    Range { puzzle_number: u32 },

    /// Show collection author
    Author { collection: String },

    /// Check balance (requires balance feature)
    #[cfg(feature = "balance")]
    Balance { id: String },
}

#[derive(Tabled)]
struct PuzzleTableRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Prize")]
    prize: String,
    #[tabled(rename = "Solve Time")]
    solve_time: String,
}

impl PuzzleTableRow {
    fn from_puzzle(p: &Puzzle, show_solve_time: bool) -> Self {
        let status = match p.status {
            Status::Solved => "solved".green().to_string(),
            Status::Unsolved => "unsolved".yellow().to_string(),
            Status::Claimed => "claimed".cyan().to_string(),
            Status::Swept => "swept".red().to_string(),
        };
        let prize = p.prize.map_or("-".dimmed().to_string(), |v| {
            format!("{:.4} {}", v, p.chain.symbol())
        });
        let solve_time = if show_solve_time {
            p.solve_time_formatted()
                .unwrap_or_else(|| "-".dimmed().to_string())
        } else {
            String::new()
        };

        Self {
            id: p.id.to_string(),
            chain: p.chain.symbol().to_string(),
            address: p.address.value.to_string(),
            status,
            prize,
            solve_time,
        }
    }
}

#[derive(Tabled)]
struct KeyValueRow {
    #[tabled(rename = "Field")]
    field: String,
    #[tabled(rename = "Value")]
    value: String,
}

#[derive(Serialize)]
struct RangeOutput {
    puzzle: u32,
    start: String,
    end: String,
    address: Option<String>,
    pubkey: Option<String>,
}

#[derive(Serialize)]
struct BalanceOutput {
    address: String,
    chain: String,
    confirmed: u128,
    confirmed_display: f64,
    unconfirmed: i128,
    total_display: f64,
    symbol: String,
}

#[derive(Serialize)]
struct StatsCsvRow {
    total: usize,
    solved: usize,
    unsolved: usize,
    claimed: usize,
    swept: usize,
    with_pubkey: usize,
    total_prize_btc: f64,
    total_prize_eth: f64,
    total_prize_ltc: f64,
    total_prize_xmr: f64,
    total_prize_dcr: f64,
    unsolved_prize_btc: f64,
    unsolved_prize_eth: f64,
    unsolved_prize_ltc: f64,
    unsolved_prize_xmr: f64,
    unsolved_prize_dcr: f64,
}

impl StatsCsvRow {
    fn from_stats(stats: &Stats) -> Self {
        fn get_prize(map: &HashMap<Chain, f64>, chain: Chain) -> f64 {
            *map.get(&chain).unwrap_or(&0.0)
        }

        Self {
            total: stats.total,
            solved: stats.solved,
            unsolved: stats.unsolved,
            claimed: stats.claimed,
            swept: stats.swept,
            with_pubkey: stats.with_pubkey,
            total_prize_btc: get_prize(&stats.total_prize, Chain::Bitcoin),
            total_prize_eth: get_prize(&stats.total_prize, Chain::Ethereum),
            total_prize_ltc: get_prize(&stats.total_prize, Chain::Litecoin),
            total_prize_xmr: get_prize(&stats.total_prize, Chain::Monero),
            total_prize_dcr: get_prize(&stats.total_prize, Chain::Decred),
            unsolved_prize_btc: get_prize(&stats.unsolved_prize, Chain::Bitcoin),
            unsolved_prize_eth: get_prize(&stats.unsolved_prize, Chain::Ethereum),
            unsolved_prize_ltc: get_prize(&stats.unsolved_prize, Chain::Litecoin),
            unsolved_prize_xmr: get_prize(&stats.unsolved_prize, Chain::Monero),
            unsolved_prize_dcr: get_prize(&stats.unsolved_prize, Chain::Decred),
        }
    }
}

fn output_puzzles(puzzles: &[&Puzzle], format: OutputFormat, show_solve_time: bool) {
    match format {
        OutputFormat::Table => {
            let rows: Vec<PuzzleTableRow> = puzzles
                .iter()
                .map(|p| PuzzleTableRow::from_puzzle(p, show_solve_time))
                .collect();
            let mut table = Table::new(rows);
            table.with(Style::rounded());
            if !show_solve_time {
                table.with(tabled::settings::Remove::column(
                    tabled::settings::location::ByColumnName::new("Solve Time"),
                ));
            }
            println!("{}", table);
            println!(
                "\n{} {} puzzles",
                "Total:".dimmed(),
                puzzles.len().to_string().bright_white()
            );
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(puzzles).unwrap());
        }
        OutputFormat::Jsonl => {
            for p in puzzles {
                println!("{}", serde_json::to_string(p).unwrap());
            }
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(puzzles).unwrap());
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            for p in puzzles {
                wtr.serialize(p).unwrap();
            }
            wtr.flush().unwrap();
        }
    }
}

fn output_puzzle(puzzle: &Puzzle, show_transactions: bool, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_puzzle_detail_table(puzzle, show_transactions),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(puzzle).unwrap());
        }
        OutputFormat::Jsonl => {
            println!("{}", serde_json::to_string(puzzle).unwrap());
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(puzzle).unwrap());
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.serialize(puzzle).unwrap();
            wtr.flush().unwrap();
        }
    }
}

fn output_stats(stats: &Stats, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_stats_table(stats),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(stats).unwrap());
        }
        OutputFormat::Jsonl => {
            println!("{}", serde_json::to_string(stats).unwrap());
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(stats).unwrap());
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.serialize(StatsCsvRow::from_stats(stats)).unwrap();
            wtr.flush().unwrap();
        }
    }
}

fn output_range(range: &RangeOutput, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_range_table(range),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(range).unwrap());
        }
        OutputFormat::Jsonl => {
            println!("{}", serde_json::to_string(range).unwrap());
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(range).unwrap());
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.serialize(range).unwrap();
            wtr.flush().unwrap();
        }
    }
}

#[cfg(feature = "balance")]
fn output_balance(balance: &BalanceOutput, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_balance_table(balance),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(balance).unwrap());
        }
        OutputFormat::Jsonl => {
            println!("{}", serde_json::to_string(balance).unwrap());
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(balance).unwrap());
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.serialize(balance).unwrap();
            wtr.flush().unwrap();
        }
    }
}

fn format_transaction_type(tx_type: TransactionType) -> String {
    match tx_type {
        TransactionType::Funding => "Funding".blue().to_string(),
        TransactionType::Increase => "Increase".green().to_string(),
        TransactionType::Decrease => "Decrease".yellow().to_string(),
        TransactionType::Sweep => "Sweep".red().to_string(),
        TransactionType::Claim => "Claim".cyan().to_string(),
        TransactionType::PubkeyReveal => "PubkeyReveal".magenta().to_string(),
    }
}

fn truncate_txid(txid: &str) -> String {
    if txid.len() > 16 {
        format!("{}...{}", &txid[..8], &txid[txid.len() - 8..])
    } else {
        txid.to_string()
    }
}

fn section(title: &str) -> KeyValueRow {
    KeyValueRow {
        field: format!("▸ {}", title).cyan().bold().to_string(),
        value: "".to_string(),
    }
}

fn print_puzzle_detail_table(p: &Puzzle, show_transactions: bool) {
    let status_colored = match p.status {
        Status::Solved => "Solved".green().to_string(),
        Status::Unsolved => "Unsolved".yellow().to_string(),
        Status::Claimed => "Claimed".cyan().to_string(),
        Status::Swept => "Swept".red().to_string(),
    };

    let mut rows = vec![
        KeyValueRow {
            field: "ID".to_string(),
            value: p.id.to_string().bright_white().to_string(),
        },
        KeyValueRow {
            field: "Chain".to_string(),
            value: p.chain.name().to_string(),
        },
        KeyValueRow {
            field: "Status".to_string(),
            value: status_colored,
        },
    ];

    if let Some(prize) = p.prize {
        rows.push(KeyValueRow {
            field: "Prize".to_string(),
            value: format!("{} {}", prize, p.chain.symbol())
                .bright_green()
                .to_string(),
        });
    }

    if let Some(url) = p.source_url {
        rows.push(KeyValueRow {
            field: "Source".to_string(),
            value: url.to_string(),
        });
    }

    rows.push(section("Address"));
    rows.push(KeyValueRow {
        field: "  Value".to_string(),
        value: p.address.value.to_string(),
    });
    rows.push(KeyValueRow {
        field: "  Type".to_string(),
        value: p.address.kind.to_uppercase(),
    });
    if let Some(hash160) = p.address.hash160 {
        rows.push(KeyValueRow {
            field: "  HASH160".to_string(),
            value: hash160.to_string(),
        });
    }
    if let Some(rs) = &p.address.redeem_script {
        rows.push(KeyValueRow {
            field: "  Redeem Script".to_string(),
            value: rs.script.to_string(),
        });
        rows.push(KeyValueRow {
            field: "  Script Hash".to_string(),
            value: rs.hash.to_string(),
        });
    }

    if let Some(pubkey) = &p.pubkey {
        rows.push(section("Public Key"));
        rows.push(KeyValueRow {
            field: "  Key".to_string(),
            value: pubkey.key.to_string(),
        });
        rows.push(KeyValueRow {
            field: "  Format".to_string(),
            value: match pubkey.format {
                PubkeyFormat::Compressed => "compressed",
                PubkeyFormat::Uncompressed => "uncompressed",
            }
            .to_string(),
        });
    }

    if let Some(key) = &p.key {
        if key.is_known() {
            rows.push(section("Private Key"));
            if let Some(hex) = key.hex {
                rows.push(KeyValueRow {
                    field: "  Hex".to_string(),
                    value: hex.to_string().bright_red().to_string(),
                });
            }
            if let Some(wif) = key.wif {
                rows.push(KeyValueRow {
                    field: "  WIF".to_string(),
                    value: wif.to_string().bright_red().to_string(),
                });
            }
            if let Some(seed) = &key.seed {
                if let Some(phrase) = seed.phrase {
                    rows.push(KeyValueRow {
                        field: "  Seed".to_string(),
                        value: phrase.to_string().bright_red().to_string(),
                    });
                }
                if let Some(path) = seed.path {
                    rows.push(KeyValueRow {
                        field: "  Seed Path".to_string(),
                        value: path.to_string(),
                    });
                }
                if let Some(xpub) = seed.xpub {
                    rows.push(KeyValueRow {
                        field: "  Xpub".to_string(),
                        value: xpub.to_string(),
                    });
                }
            }
            if let Some(mini) = key.mini {
                rows.push(KeyValueRow {
                    field: "  Mini".to_string(),
                    value: mini.to_string().bright_red().to_string(),
                });
            }
            if let Some(passphrase) = key.passphrase {
                rows.push(KeyValueRow {
                    field: "  Passphrase".to_string(),
                    value: passphrase.to_string().bright_red().to_string(),
                });
            }
        }

        if let Some(bits) = key.bits {
            rows.push(section("Key Range"));
            rows.push(KeyValueRow {
                field: "  Bits".to_string(),
                value: bits.to_string(),
            });
            if let Some((start, end)) = p.key_range_big() {
                rows.push(KeyValueRow {
                    field: "  Min".to_string(),
                    value: format!("0x{:x}", start),
                });
                rows.push(KeyValueRow {
                    field: "  Max".to_string(),
                    value: format!("0x{:x}", end),
                });
            }
        }
    }

    if p.start_date.is_some() || p.solve_date.is_some() || p.solve_time.is_some() {
        rows.push(section("Timeline"));
        if let Some(date) = p.start_date {
            rows.push(KeyValueRow {
                field: "  Funded".to_string(),
                value: date.to_string(),
            });
        }
        if let Some(date) = p.solve_date {
            rows.push(KeyValueRow {
                field: "  Solved".to_string(),
                value: date.to_string(),
            });
        }
        if let Some(formatted) = p.solve_time_formatted() {
            rows.push(KeyValueRow {
                field: "  Duration".to_string(),
                value: formatted,
            });
        }
    }

    if let Some(txid) = p.claim_txid() {
        rows.push(section("Claim"));
        rows.push(KeyValueRow {
            field: "  TX".to_string(),
            value: txid.to_string(),
        });
        rows.push(KeyValueRow {
            field: "  Explorer".to_string(),
            value: p.chain.tx_explorer_url(txid),
        });
    }

    if let Some(solver) = &p.solver {
        if solver.name.is_some() || solver.address.is_some() {
            rows.push(section("Solver"));
            if let Some(name) = solver.name {
                let verified_badge = if solver.verified {
                    " ✓".green().to_string()
                } else {
                    "".to_string()
                };
                rows.push(KeyValueRow {
                    field: "  Name".to_string(),
                    value: format!("{}{}", name.bright_white(), verified_badge),
                });
            }
            if let Some(addr) = solver.address {
                rows.push(KeyValueRow {
                    field: "  Address".to_string(),
                    value: addr.to_string(),
                });
            }
            if let Some(source) = solver.source {
                rows.push(KeyValueRow {
                    field: "  Source".to_string(),
                    value: source.to_string(),
                });
            }
        }
    }

    if show_transactions && !p.transactions.is_empty() {
        rows.push(section("Transactions"));
        for tx in p.transactions {
            let amount_str = tx
                .amount
                .map(|a| format!(" ({:.8} {})", a, p.chain.symbol()))
                .unwrap_or_default();
            let date_str = tx.date.unwrap_or("-");
            let txid_str = tx
                .txid
                .map(truncate_txid)
                .unwrap_or_else(|| "-".to_string());
            rows.push(KeyValueRow {
                field: format!("  {}", format_transaction_type(tx.tx_type)),
                value: format!("{} {}{}", txid_str, date_str, amount_str),
            });
        }
    }

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{}", table);
}

fn print_stats_table(stats: &Stats) {
    let mut rows = vec![
        KeyValueRow {
            field: "Total puzzles".to_string(),
            value: stats.total.to_string().bright_white().to_string(),
        },
        KeyValueRow {
            field: "Solved".to_string(),
            value: stats.solved.to_string().green().to_string(),
        },
        KeyValueRow {
            field: "Unsolved".to_string(),
            value: stats.unsolved.to_string().yellow().to_string(),
        },
        KeyValueRow {
            field: "Claimed".to_string(),
            value: stats.claimed.to_string().cyan().to_string(),
        },
        KeyValueRow {
            field: "Swept".to_string(),
            value: stats.swept.to_string().red().to_string(),
        },
        KeyValueRow {
            field: "With public key".to_string(),
            value: stats.with_pubkey.to_string(),
        },
    ];

    let mut total_prizes: Vec<_> = stats.total_prize.iter().collect();
    total_prizes.sort_by_key(|(chain, _)| chain.symbol());
    for (chain, amount) in total_prizes {
        rows.push(KeyValueRow {
            field: format!("Total {}", chain.symbol()),
            value: format!("{:.2}", amount),
        });
    }

    let mut unsolved_prizes: Vec<_> = stats.unsolved_prize.iter().collect();
    unsolved_prizes.sort_by_key(|(chain, _)| chain.symbol());
    for (chain, amount) in unsolved_prizes {
        rows.push(KeyValueRow {
            field: format!("Unsolved {}", chain.symbol()),
            value: format!("{:.2}", amount).bright_yellow().to_string(),
        });
    }

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{}", table);
}

fn print_range_table(range: &RangeOutput) {
    let mut rows = vec![
        KeyValueRow {
            field: "Puzzle".to_string(),
            value: range.puzzle.to_string().bright_white().to_string(),
        },
        KeyValueRow {
            field: "Start".to_string(),
            value: range.start.clone(),
        },
        KeyValueRow {
            field: "End".to_string(),
            value: range.end.clone(),
        },
    ];

    if let Some(addr) = &range.address {
        rows.push(KeyValueRow {
            field: "Address".to_string(),
            value: addr.clone(),
        });
    }

    if let Some(pk) = &range.pubkey {
        rows.push(KeyValueRow {
            field: "Pubkey".to_string(),
            value: pk.clone(),
        });
    }

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{}", table);
}

#[cfg(feature = "balance")]
fn print_balance_table(balance: &BalanceOutput) {
    let unit = if balance.symbol == "ETH" {
        "wei"
    } else {
        "sats"
    };
    let rows = vec![
        KeyValueRow {
            field: "Address".to_string(),
            value: balance.address.clone(),
        },
        KeyValueRow {
            field: "Chain".to_string(),
            value: balance.chain.clone(),
        },
        KeyValueRow {
            field: "Confirmed".to_string(),
            value: format!(
                "{} {} ({:.8} {})",
                balance.confirmed.to_string().green(),
                unit,
                balance.confirmed_display,
                balance.symbol
            ),
        },
        KeyValueRow {
            field: "Unconfirmed".to_string(),
            value: if balance.unconfirmed != 0 {
                format!("{} {}", balance.unconfirmed, unit)
            } else {
                "-".dimmed().to_string()
            },
        },
        KeyValueRow {
            field: "Total".to_string(),
            value: format!("{:.8} {}", balance.total_display, balance.symbol)
                .bright_green()
                .to_string(),
        },
    ];

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{}", table);
}

fn cmd_list(
    collection: &str,
    unsolved: bool,
    solved: bool,
    with_pubkey: bool,
    with_transactions: bool,
    chain_filter: Option<Chain>,
    format: OutputFormat,
) {
    let puzzles: Vec<&Puzzle> = match collection {
        "b1000" => b1000::all().collect(),
        "bitaps" => bitaps::all().collect(),
        "bitimage" => bitimage::all().collect(),
        "gsmg" => gsmg::all().collect(),
        "hash_collision" | "peter_todd" => hash_collision::all().collect(),
        "zden" => zden::all().collect(),
        _ => boha::all().collect(),
    };

    let filtered: Vec<_> = puzzles
        .into_iter()
        .filter(|p| !unsolved || p.status == Status::Unsolved)
        .filter(|p| !solved || p.status == Status::Solved)
        .filter(|p| !with_pubkey || p.pubkey.is_some())
        .filter(|p| !with_transactions || p.has_transactions())
        .filter(|p| chain_filter.is_none_or(|c| p.chain == c))
        .collect();

    output_puzzles(&filtered, format, solved);
}

fn cmd_show(id: &str, show_transactions: bool, format: OutputFormat) {
    match boha::get(id) {
        Ok(puzzle) => output_puzzle(puzzle, show_transactions, format),
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn cmd_stats(format: OutputFormat) {
    let stats = boha::stats();
    output_stats(&stats, format);
}

fn cmd_range(puzzle_number: u32, format: OutputFormat) {
    match b1000::get(puzzle_number) {
        Ok(p) => {
            let (start, end) = p.key_range_big().expect("b1000 puzzles always have bits");
            let range = RangeOutput {
                puzzle: puzzle_number,
                start: format!("0x{:x}", start),
                end: format!("0x{:x}", end),
                address: Some(p.address.value.to_string()),
                pubkey: p.pubkey.map(|pk| pk.key.to_string()),
            };
            output_range(&range, format);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn cmd_author(collection: &str, format: OutputFormat) {
    let author = match collection {
        "b1000" => b1000::author(),
        "bitaps" => bitaps::author(),
        "bitimage" => bitimage::author(),
        "gsmg" => gsmg::author(),
        "hash_collision" | "peter_todd" => hash_collision::author(),
        "zden" => zden::author(),
        _ => {
            eprintln!(
                "{} Unknown collection: {}. Use: b1000, bitaps, bitimage, gsmg, hash_collision, zden",
                "Error:".red().bold(),
                collection
            );
            std::process::exit(1);
        }
    };
    output_author(author, format);
}

fn output_author(author: &Author, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_author_table(author),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(author).unwrap());
        }
        OutputFormat::Jsonl => {
            println!("{}", serde_json::to_string(author).unwrap());
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(author).unwrap());
        }
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.serialize(author).unwrap();
            wtr.flush().unwrap();
        }
    }
}

fn print_author_table(author: &Author) {
    let mut rows = vec![];

    rows.push(KeyValueRow {
        field: "Name".to_string(),
        value: author
            .name
            .map(|n| n.bright_white().to_string())
            .unwrap_or_else(|| "Anonymous".dimmed().to_string()),
    });

    if !author.addresses.is_empty() {
        rows.push(KeyValueRow {
            field: "Addresses".to_string(),
            value: author.addresses.join(", "),
        });
    }

    if let Some(profile) = author.profile {
        rows.push(KeyValueRow {
            field: "Profile".to_string(),
            value: profile.to_string(),
        });
    }

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{}", table);
}

#[cfg(feature = "balance")]
async fn cmd_balance(id: &str, format: OutputFormat) {
    match boha::get(id) {
        Ok(puzzle) => match boha::balance::fetch(puzzle.address.value, puzzle.chain).await {
            Ok(bal) => {
                let (confirmed_display, total_display) = match puzzle.chain {
                    Chain::Ethereum => (bal.confirmed_eth(), bal.total_eth()),
                    _ => (bal.confirmed_btc(), bal.total_btc()),
                };
                let output = BalanceOutput {
                    address: puzzle.address.value.to_string(),
                    chain: puzzle.chain.name().to_string(),
                    confirmed: bal.confirmed,
                    confirmed_display,
                    unconfirmed: bal.unconfirmed,
                    total_display,
                    symbol: puzzle.chain.symbol().to_string(),
                };
                output_balance(&output, format);
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "balance")]
#[tokio::main]
async fn main() {
    human_panic::setup_panic!();
    let cli = Cli::parse();
    run(cli).await;
}

#[cfg(not(feature = "balance"))]
fn main() {
    human_panic::setup_panic!();
    let cli = Cli::parse();
    run(cli);
}

#[cfg(feature = "balance")]
async fn run(cli: Cli) {
    match cli.command {
        Commands::Balance { id } => cmd_balance(&id, cli.output).await,
        _ => run_sync(cli),
    }
}

#[cfg(feature = "balance")]
fn run_sync(cli: Cli) {
    match cli.command {
        Commands::List {
            collection,
            unsolved,
            solved,
            with_pubkey,
            with_transactions,
            chain,
        } => cmd_list(
            &collection,
            unsolved,
            solved,
            with_pubkey,
            with_transactions,
            chain,
            cli.output,
        ),
        Commands::Show { id, transactions } => cmd_show(&id, transactions, cli.output),
        Commands::Stats => cmd_stats(cli.output),
        Commands::Range { puzzle_number } => cmd_range(puzzle_number, cli.output),
        Commands::Author { collection } => cmd_author(&collection, cli.output),
        Commands::Balance { .. } => unreachable!(),
    }
}

#[cfg(not(feature = "balance"))]
fn run(cli: Cli) {
    match cli.command {
        Commands::List {
            collection,
            unsolved,
            solved,
            with_pubkey,
            with_transactions,
            chain,
        } => cmd_list(
            &collection,
            unsolved,
            solved,
            with_pubkey,
            with_transactions,
            chain,
            cli.output,
        ),
        Commands::Show { id, transactions } => cmd_show(&id, transactions, cli.output),
        Commands::Stats => cmd_stats(cli.output),
        Commands::Range { puzzle_number } => cmd_range(puzzle_number, cli.output),
        Commands::Author { collection } => cmd_author(&collection, cli.output),
    }
}
