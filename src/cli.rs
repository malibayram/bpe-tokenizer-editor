//! CLI argument definitions

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "BPE Tokenizer Editor - Add/Remove tokens with consistency guarantees"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Validate merges and optionally clean invalid ones
    Validate {
        /// Input tokenizer.json file
        #[arg(short, long)]
        input: PathBuf,

        /// Output tokenizer.json file (if not specified, will overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Dry run - only report, don't modify
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },

    /// Add tokens from a JSON file
    Add {
        /// Input tokenizer.json file
        #[arg(short, long)]
        input: PathBuf,

        /// JSON file with tokens to add (array of strings)
        #[arg(short, long)]
        tokens: PathBuf,

        /// Output tokenizer.json file
        #[arg(short, long)]
        output: PathBuf,

        /// Keep vocab size fixed by removing least important tokens
        #[arg(long, default_value = "true")]
        keep_size: bool,

        /// JSON file with tokens that should never be removed
        #[arg(long)]
        whitelist: Option<PathBuf>,
    },

    /// Remove tokens from a JSON file
    Remove {
        /// Input tokenizer.json file
        #[arg(short, long)]
        input: PathBuf,

        /// JSON file with tokens to remove (array of strings)
        #[arg(short, long)]
        tokens: PathBuf,

        /// Output tokenizer.json file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Show tokenizer stats
    Stats {
        /// Input tokenizer.json file
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Shrink vocab by removing N longest non-special tokens with highest IDs
    Shrink {
        /// Input tokenizer.json file
        #[arg(short, long)]
        input: PathBuf,

        /// Output tokenizer.json file
        #[arg(short, long)]
        output: PathBuf,

        /// Number of tokens to remove
        #[arg(short, long)]
        count: usize,

        /// Minimum token ID to consider for removal (default: 50000)
        #[arg(long, default_value = "50000")]
        min_id: u32,

        /// Dry run - only report tokens that would be removed
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Save removed tokens to a JSON file
        #[arg(long)]
        save_removed: Option<PathBuf>,
    },

    /// Sync single-letter tokens from source to target tokenizer
    SyncChars {
        /// Source tokenizer.json file (to copy single-letter tokens from)
        #[arg(short, long)]
        source: PathBuf,

        /// Target tokenizer.json file (to add tokens to)
        #[arg(short, long)]
        target: PathBuf,

        /// Output tokenizer.json file
        #[arg(short, long)]
        output: PathBuf,

        /// Minimum token ID to consider for removal (default: 50000)
        #[arg(long, default_value = "50000")]
        min_id: u32,

        /// Dry run - only report what would happen
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Save sync report to a JSON file
        #[arg(long)]
        save_report: Option<PathBuf>,
    },

    /// Sync 2 and 3 letter tokens from source to target tokenizer (with their merges)
    SyncShortTokens {
        /// Source tokenizer.json file (to copy tokens from)
        #[arg(short, long)]
        source: PathBuf,

        /// Target tokenizer.json file (to add tokens to)
        #[arg(short, long)]
        target: PathBuf,

        /// Output tokenizer.json file
        #[arg(short, long)]
        output: PathBuf,

        /// Minimum token length to sync (default: 2)
        #[arg(long, default_value = "2")]
        min_len: usize,

        /// Maximum token length to sync (default: 3)
        #[arg(long, default_value = "3")]
        max_len: usize,

        /// Minimum token ID to consider for removal (default: 50000)
        #[arg(long, default_value = "50000")]
        min_id: u32,

        /// Dry run - only report what would happen
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Save sync report to a JSON file
        #[arg(long)]
        save_report: Option<PathBuf>,
    },
}
