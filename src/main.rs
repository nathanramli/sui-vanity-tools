use clap::Parser;
use crossbeam_channel::{bounded, select};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sui_keys::key_derive::generate_new_key;
use sui_types::crypto::SignatureScheme;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target prefix (without 0x)
    #[arg(short, long)]
    prefix: Option<String>,

    /// Target suffix (last characters of address)
    #[arg(short, long)]
    suffix: Option<String>,

    /// Word size for mnemonic (12, 15, 18, 21, or 24)
    #[arg(short, long, default_value = "24")]
    word_size: u8,

    /// Number of worker threads (default: number of CPU cores)
    #[arg(short, long)]
    threads: Option<usize>,

    /// Batch size: number of keys to generate before checking counter
    #[arg(short, long, default_value = "1000")]
    batch_size: u32,
}

struct Result {
    address: String,
    mnemonic: String,
}

#[derive(Clone)]
enum MatchMode {
    Prefix(String),
    Suffix(String),
    Both { prefix: String, suffix: String },
}

impl MatchMode {
    fn matches(&self, address: &str) -> bool {
        let addr_lower = address.to_lowercase();
        match self {
            MatchMode::Prefix(prefix) => addr_lower.starts_with(&prefix.to_lowercase()),
            MatchMode::Suffix(suffix) => addr_lower.ends_with(&suffix.to_lowercase()),
            MatchMode::Both { prefix, suffix } => {
                addr_lower.starts_with(&prefix.to_lowercase())
                    && addr_lower.ends_with(&suffix.to_lowercase())
            }
        }
    }

    fn difficulty(&self) -> u64 {
        match self {
            MatchMode::Prefix(p) => {
                let len = p.chars().skip(2).count(); // Skip "0x"
                16_u64.saturating_pow(len as u32)
            }
            MatchMode::Suffix(s) => 16_u64.saturating_pow(s.chars().count() as u32),
            MatchMode::Both { prefix, suffix } => {
                let prefix_len = prefix.chars().skip(2).count();
                let suffix_len = suffix.chars().count();
                16_u64
                    .saturating_pow(prefix_len as u32)
                    .saturating_mul(16_u64.saturating_pow(suffix_len as u32))
            }
        }
    }

    fn description(&self) -> String {
        match self {
            MatchMode::Prefix(p) => format!("Prefix: {}", p),
            MatchMode::Suffix(s) => format!("Suffix: {}", s),
            MatchMode::Both { prefix, suffix } => format!("Prefix: {} / Suffix: {}", prefix, suffix),
        }
    }
}

fn main() {
    let args = Args::parse();

    // Validate that at least one of prefix or suffix is provided
    if args.prefix.is_none() && args.suffix.is_none() {
        eprintln!("Error: Must specify at least one of --prefix or --suffix");
        std::process::exit(1);
    }

    // Build match mode
    let match_mode = match (args.prefix, args.suffix) {
        (Some(p), None) => {
            let prefix = format!("0x{}", p.to_lowercase());
            if !prefix.chars().skip(2).all(|c| c.is_ascii_hexdigit()) {
                eprintln!("Error: Prefix must contain only hexadecimal characters (0-9, a-f)");
                std::process::exit(1);
            }
            MatchMode::Prefix(prefix)
        }
        (None, Some(s)) => {
            if !s.chars().all(|c| c.is_ascii_hexdigit()) {
                eprintln!("Error: Suffix must contain only hexadecimal characters (0-9, a-f)");
                std::process::exit(1);
            }
            MatchMode::Suffix(s.to_lowercase())
        }
        (Some(p), Some(s)) => {
            let prefix = format!("0x{}", p.to_lowercase());
            if !prefix.chars().skip(2).all(|c| c.is_ascii_hexdigit()) {
                eprintln!("Error: Prefix must contain only hexadecimal characters (0-9, a-f)");
                std::process::exit(1);
            }
            if !s.chars().all(|c| c.is_ascii_hexdigit()) {
                eprintln!("Error: Suffix must contain only hexadecimal characters (0-9, a-f)");
                std::process::exit(1);
            }
            MatchMode::Both {
                prefix,
                suffix: s.to_lowercase(),
            }
        }
        (None, None) => unreachable!(),
    };

    // Validate word size
    if ![12, 15, 18, 21, 24].contains(&args.word_size) {
        eprintln!("Error: Word size must be 12, 15, 18, 21, or 24");
        std::process::exit(1);
    }

    // Determine thread count
    let num_threads = args.threads.unwrap_or_else(num_cpus::get);

    println!("🔍 Sui Vanity Address Generator (Optimized)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Target:            {}", match_mode.description());
    println!("Word size:         {}", args.word_size);
    println!("Worker threads:    {}", num_threads);
    println!("Batch size:        {}", args.batch_size);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Calculate estimated difficulty
    let difficulty = match_mode.difficulty();
    println!(
        "Estimated attempts needed: ~{} (1 in {})",
        format_number(difficulty),
        format_number(difficulty)
    );
    println!("Starting search...\n");

    // Shared state
    let found = Arc::new(AtomicBool::new(false));
    let total_attempts = Arc::new(AtomicU64::new(0));
    let (result_tx, result_rx) = bounded::<Result>(1);

    // Spawn worker threads
    let mut handles = vec![];
    for _thread_id in 0..num_threads {
        let match_mode = match_mode.clone();
        let found = Arc::clone(&found);
        let total_attempts = Arc::clone(&total_attempts);
        let result_tx = result_tx.clone();
        let word_size = args.word_size;
        let batch_size = args.batch_size;

        let handle = thread::spawn(move || {
            let mut local_attempts = 0u64;

            loop {
                // Check if another thread found a match
                if found.load(Ordering::Relaxed) {
                    break;
                }

                // Generate keys in batches for better performance
                for _ in 0..batch_size {
                    let (sui_address, _, _, mnemonic) = generate_new_key(
                        SignatureScheme::ED25519,
                        None,
                        Some(format!("word{}", word_size)),
                    )
                    .unwrap();

                    local_attempts += 1;

                    let address_str = sui_address.to_string();

                    if match_mode.matches(&address_str) {
                        // Found a match!
                        found.store(true, Ordering::Relaxed);
                        total_attempts.fetch_add(local_attempts, Ordering::Relaxed);

                        let _ = result_tx.send(Result {
                            address: address_str,
                            mnemonic,
                        });
                        return;
                    }
                }

                // Update global counter periodically (after each batch)
                total_attempts.fetch_add(batch_size as u64, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    // Drop the original sender so the channel closes when all threads are done
    drop(result_tx);

    // Progress reporting thread
    let progress_found = Arc::clone(&found);
    let progress_attempts = Arc::clone(&total_attempts);
    let progress_difficulty = difficulty;
    let progress_handle = thread::spawn(move || {
        let start_time = Instant::now();
        let mut last_attempts = 0u64;
        let mut last_time = Instant::now();

        loop {
            thread::sleep(Duration::from_secs(2));

            if progress_found.load(Ordering::Relaxed) {
                break;
            }

            let current_attempts = progress_attempts.load(Ordering::Relaxed);
            let current_time = Instant::now();

            let elapsed = current_time.duration_since(last_time).as_secs_f64();
            let attempts_delta = current_attempts - last_attempts;
            let rate = attempts_delta as f64 / elapsed;

            let total_elapsed = current_time.duration_since(start_time);

            // Calculate estimated time remaining
            let est_time_str = if current_attempts > 0 && rate > 0.0 {
                let remaining_attempts = progress_difficulty.saturating_sub(current_attempts);
                let est_seconds = remaining_attempts as f64 / rate;
                format!(" | ETA: {}", format_duration(Duration::from_secs(est_seconds as u64)))
            } else {
                String::from(" | ETA: calculating...")
            };

            println!(
                "⚡ {} attempts | {}/sec | elapsed: {}{}",
                format_number(current_attempts),
                format_number(rate as u64),
                format_duration(total_elapsed),
                est_time_str
            );

            last_attempts = current_attempts;
            last_time = current_time;
        }
    });

    // Wait for result or completion
    select! {
        recv(result_rx) -> msg => {
            if let Ok(result) = msg {
                let final_attempts = total_attempts.load(Ordering::Relaxed);

                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("✅ FOUND MATCHING ADDRESS!");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("Address:  {}", result.address);
                println!("Mnemonic: {}", result.mnemonic);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("Total attempts: {}", format_number(final_attempts));
            }
        }
    }

    // Wait for all threads to finish
    for handle in handles {
        let _ = handle.join();
    }
    let _ = progress_handle.join();
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
