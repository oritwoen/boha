use boha::{b1000, gsmg, hash_collision, Chain, PubkeyFormat, Puzzle, Stats, Status};
use clap::{Parser, Subcommand, ValueEnum};

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

        #[arg(long, value_parser = parse_chain)]
        chain: Option<Chain>,
    },

    /// Show puzzle details
    Show { id: String },

    /// Show statistics
    Stats,

    /// Show key range for puzzle
    Range { puzzle_number: u32 },

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
}

impl PuzzleTableRow {
    fn from_puzzle(p: &Puzzle) -> Self {
        let status = match p.status {
            Status::Solved => "solved".green().to_string(),
            Status::Unsolved => "unsolved".yellow().to_string(),
            Status::Claimed => "claimed".cyan().to_string(),
            Status::Swept => "swept".red().to_string(),
        };
        let prize = p.prize.map_or("-".dimmed().to_string(), |v| {
            format!("{:.4} {}", v, p.chain.symbol())
        });

        Self {
            id: p.id.to_string(),
            chain: p.chain.symbol().to_string(),
            address: p.address.to_string(),
            status,
            prize,
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
    confirmed: u64,
    confirmed_btc: f64,
    unconfirmed: i64,
    total_btc: f64,
}

fn output_puzzles(puzzles: &[&Puzzle], format: OutputFormat) {
    match format {
        OutputFormat::Table => {
            let rows: Vec<PuzzleTableRow> = puzzles
                .iter()
                .map(|p| PuzzleTableRow::from_puzzle(p))
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
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

fn output_puzzle(puzzle: &Puzzle, format: OutputFormat) {
    match format {
        OutputFormat::Table => print_puzzle_detail_table(puzzle),
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
            wtr.serialize(stats).unwrap();
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

fn print_puzzle_detail_table(p: &Puzzle) {
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
            field: "Address".to_string(),
            value: p.address.to_string(),
        },
        KeyValueRow {
            field: "Status".to_string(),
            value: status_colored,
        },
    ];

    if let Some(bits) = p.bits {
        rows.push(KeyValueRow {
            field: "Bits".to_string(),
            value: bits.to_string(),
        });
        let (start, end) = b1000::key_range_big(bits as u32);
        rows.push(KeyValueRow {
            field: "Range".to_string(),
            value: format!("0x{:x} - 0x{:x}", start, end),
        });
    }

    if let Some(pubkey) = &p.pubkey {
        rows.push(KeyValueRow {
            field: "Pubkey".to_string(),
            value: pubkey.key.to_string(),
        });
        rows.push(KeyValueRow {
            field: "Pubkey Format".to_string(),
            value: match pubkey.format {
                PubkeyFormat::Compressed => "compressed",
                PubkeyFormat::Uncompressed => "uncompressed",
            }
            .to_string(),
        });
    }

    if let Some(key) = p.private_key {
        rows.push(KeyValueRow {
            field: "Private Key".to_string(),
            value: key.to_string().bright_red().to_string(),
        });
    }

    if let Some(script) = p.redeem_script {
        rows.push(KeyValueRow {
            field: "Redeem Script".to_string(),
            value: script.to_string(),
        });
    }

    if let Some(prize) = p.prize {
        rows.push(KeyValueRow {
            field: "Prize".to_string(),
            value: format!("{} {}", prize, p.chain.symbol())
                .bright_green()
                .to_string(),
        });
    }

    if let Some(date) = p.start_date {
        rows.push(KeyValueRow {
            field: "Funded".to_string(),
            value: date.to_string(),
        });
    }

    if let Some(date) = p.solve_date {
        rows.push(KeyValueRow {
            field: "Solved".to_string(),
            value: date.to_string(),
        });
    }

    if let Some(url) = p.source_url {
        rows.push(KeyValueRow {
            field: "Source".to_string(),
            value: url.to_string(),
        });
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
    let rows = vec![
        KeyValueRow {
            field: "Address".to_string(),
            value: balance.address.clone(),
        },
        KeyValueRow {
            field: "Confirmed".to_string(),
            value: format!(
                "{} sats ({:.8} BTC)",
                balance.confirmed.to_string().green(),
                balance.confirmed_btc
            ),
        },
        KeyValueRow {
            field: "Unconfirmed".to_string(),
            value: if balance.unconfirmed != 0 {
                format!("{} sats", balance.unconfirmed)
            } else {
                "-".dimmed().to_string()
            },
        },
        KeyValueRow {
            field: "Total".to_string(),
            value: format!("{:.8} BTC", balance.total_btc)
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
    chain_filter: Option<Chain>,
    format: OutputFormat,
) {
    let puzzles: Vec<&Puzzle> = match collection {
        "b1000" => b1000::all().collect(),
        "gsmg" => gsmg::all().collect(),
        "hash_collision" | "peter_todd" => hash_collision::all().collect(),
        _ => boha::all().collect(),
    };

    let filtered: Vec<_> = puzzles
        .into_iter()
        .filter(|p| !unsolved || p.status == Status::Unsolved)
        .filter(|p| !solved || p.status == Status::Solved)
        .filter(|p| !with_pubkey || p.pubkey.is_some())
        .filter(|p| chain_filter.is_none_or(|c| p.chain == c))
        .collect();

    output_puzzles(&filtered, format);
}

fn cmd_show(id: &str, format: OutputFormat) {
    match boha::get(id) {
        Ok(puzzle) => output_puzzle(puzzle, format),
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
    let (start, end) = b1000::key_range_big(puzzle_number);

    let (address, pubkey) = if let Ok(p) = b1000::get(puzzle_number) {
        (
            Some(p.address.to_string()),
            p.pubkey.map(|pk| pk.key.to_string()),
        )
    } else {
        (None, None)
    };

    let range = RangeOutput {
        puzzle: puzzle_number,
        start: format!("0x{:x}", start),
        end: format!("0x{:x}", end),
        address,
        pubkey,
    };

    output_range(&range, format);
}

#[cfg(feature = "balance")]
async fn cmd_balance(id: &str, format: OutputFormat) {
    match boha::get(id) {
        Ok(puzzle) => match boha::balance::fetch(puzzle.address).await {
            Ok(bal) => {
                let output = BalanceOutput {
                    address: puzzle.address.to_string(),
                    confirmed: bal.confirmed,
                    confirmed_btc: bal.confirmed_btc(),
                    unconfirmed: bal.unconfirmed,
                    total_btc: bal.total_btc(),
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
    let cli = Cli::parse();
    run(cli).await;
}

#[cfg(not(feature = "balance"))]
fn main() {
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
            chain,
        } => cmd_list(
            &collection,
            unsolved,
            solved,
            with_pubkey,
            chain,
            cli.output,
        ),
        Commands::Show { id } => cmd_show(&id, cli.output),
        Commands::Stats => cmd_stats(cli.output),
        Commands::Range { puzzle_number } => cmd_range(puzzle_number, cli.output),
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
            chain,
        } => cmd_list(
            &collection,
            unsolved,
            solved,
            with_pubkey,
            chain,
            cli.output,
        ),
        Commands::Show { id } => cmd_show(&id, cli.output),
        Commands::Stats => cmd_stats(cli.output),
        Commands::Range { puzzle_number } => cmd_range(puzzle_number, cli.output),
    }
}
