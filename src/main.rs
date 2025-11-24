//! Module: Main
//! But: Entry point for the Kaspa vanity address generator.
//!
//! *Signed: kryon.kas*

mod vanity;

use clap::Parser;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::vanity::generate_random_mnemonic;

// --- Configuration ---

/// Kaspa Vanity Address Generator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Prefix to search for (e.g., "test")
    /// NB: Matching starts from the 3rd character of the Bech32 payload.
    #[arg(short, long)]
    prefix: Option<String>,

    /// Suffix to search for (e.g., "2025")
    /// NB: Long suffixes are computationally expensive due to the checksum.
    #[arg(short, long)]
    suffix: Option<String>,

    /// Number of threads to use.
    /// Défaut: All logical cores.
    #[arg(short, long)]
    threads: Option<usize>,

    /// Case sensitive matching.
    #[arg(long, default_value_t = false)]
    case_sensitive: bool,

    /// Mnemonic word count: 12 or 24.
    #[arg(short, long, default_value_t = 24)]
    words: usize,

    /// Address scan limit per mnemonic.
    /// Increases speed by checking multiple indices (0..N) per seed.
    #[arg(long, default_value_t = 1)]
    scan_limit: u32,
}

// --- Utilitaires ---

/// Validates a search pattern (prefix or suffix) for invalid Bech32 characters.
fn validate_pattern(pattern: &str, name: &str, invalid_chars: &[char]) {
    if let Some(invalid_char) = pattern.chars().find(|c| invalid_chars.contains(c)) {
        eprintln!("Error: Invalid character '{invalid_char}' in {name} '{pattern}'");
        eprintln!();
        eprintln!("Bech32 encoding excludes the following characters to avoid confusion:");
        eprintln!("  - '1' : separator");
        eprintln!("  - 'b' : confused with '6'");
        eprintln!("  - 'i' : confused with '1' and 'l'");
        eprintln!("  - 'o' : confused with '0' (zero)");
        eprintln!();
        eprintln!("Valid Bech32 characters: qpzry9x8gf2tvdw0s3jn54khce6mua7l");
        std::process::exit(1);
    }
}

// --- Exécution ---

fn main() {
    let args = Args::parse();

    // Vérification des arguments
    if args.prefix.is_none() && args.suffix.is_none() {
        eprintln!("Error: You must specify at least one of --prefix or --suffix");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  kas-vanity --prefix test");
        eprintln!("  kas-vanity --suffix 2025");
        eprintln!("  kas-vanity --prefix test --suffix 2025");
        std::process::exit(1);
    }

    // Configuration du ThreadPool
    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("Failed to build thread pool");
    }

    // --- Validation ---

    // Bech32 charset excludes '1', 'b', 'i', 'o'.
    // Attention: These characters are strictly forbidden.
    const INVALID_CHARS: &[char] = &['1', 'b', 'i', 'o'];
    
    let prefix = args.prefix.map(|p| {
        let normalized = if args.case_sensitive { p } else { p.to_lowercase() };
        validate_pattern(&normalized, "prefix", INVALID_CHARS);
        normalized
    });

    let suffix = args.suffix.map(|s| {
        let normalized = if args.case_sensitive { s } else { s.to_lowercase() };
        validate_pattern(&normalized, "suffix", INVALID_CHARS);
        normalized
    });

    // --- Initialisation ---

    // Calculate target probability
    let prefix_len = prefix.as_ref().map_or(0, |p| p.len());
    let suffix_len = suffix.as_ref().map_or(0, |s| s.len());
    let total_len = prefix_len + suffix_len;
    let prob_single = 1.0 / 32.0f64.powi(total_len as i32);

    println!("Searching for prefix: {:?}", prefix.as_deref().unwrap_or(""));
    if let Some(s) = &suffix {
        println!("Searching for suffix: {s}");
    }
    println!("Difficulty: 1 in {:.0} (approx)", 1.0 / prob_single);
    println!("Scan limit: {} addresses per mnemonic", args.scan_limit);
    println!("Using {} threads...", rayon::current_num_threads());

    let start_time = std::time::Instant::now();
    let found = Arc::new(AtomicBool::new(false));
    let counter = Arc::new(AtomicU64::new(0));

    // --- Boucle Principale ---
    // Parallel infinite loop searching for matching addresses.
    // La chasse commence.
    rayon::iter::repeat(()).for_each(|_| {
        if found.load(Ordering::Relaxed) {
            return;
        }

        let mnemonic = generate_random_mnemonic(args.words);
        
        // Derive batch of addresses (0..scan_limit)
        let addresses = crate::vanity::derive_batch(&mnemonic, args.scan_limit);
        
        if addresses.is_empty() {
             eprintln!("Warning: Failed to derive addresses (should be rare)");
             return;
        }

        let count = counter.fetch_add(addresses.len() as u64, Ordering::Relaxed);

        // Progress indicator every 1000 addresses
        if count.is_multiple_of(1000) {
            let prob = 1.0 - (1.0 - prob_single).powf(count as f64);
            println!("Checked {count} addresses... ({:.2}% chance)", prob * 100.0);
        }

        for (index, address) in addresses {
            let addr_str = address.to_string();
            let payload = addr_str.split(':').nth(1).unwrap_or("");
            
            // Kaspa address format:
            // 1. Version prefix ('q')
            // 2. Limited char (p, q, r, z)
            // 3. Payload (Full Bech32)
            //
            // NB: We skip the first 2 characters for prefix matching.
            let searchable = if payload.len() > 2 { &payload[2..] } else { "" };

            // Vérification du préfixe
            let prefix_match = prefix.as_ref().is_none_or(|p| {
                if args.case_sensitive {
                    searchable.starts_with(p)
                } else {
                    searchable.to_lowercase().starts_with(p)
                }
            });

            // Vérification du suffixe
            let suffix_match = suffix.as_ref().is_none_or(|s| {
                if args.case_sensitive {
                    searchable.ends_with(s)
                } else {
                    searchable.to_lowercase().ends_with(s)
                }
            });

            if prefix_match && suffix_match {
                let elapsed = start_time.elapsed();
                println!("\n[MATCH FOUND]");
                println!("Address: {addr_str}");
                println!("Mnemonic: {mnemonic}");
                if args.scan_limit > 1 {
                    println!("Path Index: {index} (m/44'/111111'/0'/0/{index})");
                }
                println!("Time taken: {elapsed:.2?}");
                
                // Signal victory to all threads
                found.store(true, Ordering::Relaxed);
                std::process::exit(0);
            }
        }
    });
}
