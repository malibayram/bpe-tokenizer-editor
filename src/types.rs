//! Result types for tokenizer operations

use serde::Serialize;

/// Information about a token removal
#[derive(Debug, Clone, Serialize)]
pub struct TokenRemovalInfo {
    pub token: String,
    pub id: u32,
    pub length: usize,
    pub cascade_tokens_removed: usize,
    pub merges_removed: usize,
}

/// Result of a token removal operation
#[derive(Debug)]
#[allow(dead_code)]
pub struct RemovalResult {
    pub root_token: String,
    pub removed_tokens: Vec<String>,
    pub removed_merges: Vec<(String, String)>,
}

/// Result of a token addition operation
#[derive(Debug)]
#[allow(dead_code)]
pub struct AdditionResult {
    pub token: String,
    pub added: bool,
    pub method: String,
    pub added_merges: Vec<(String, String)>,
}

/// Result of batch token addition
#[derive(Debug)]
pub struct BatchAddResult {
    pub initial_vocab_size: usize,
    pub final_vocab_size: usize,
    pub tokens_requested: usize,
    pub tokens_added: usize,
    pub tokens_already_present: usize,
    pub merges_added: usize,
    pub tokens_removed: usize,
    pub additions: Vec<AdditionResult>,
    pub removals: Vec<(RemovalResult, String)>,
}

/// Result of vocab shrinking
#[derive(Debug)]
pub struct ShrinkResult {
    pub initial_vocab_size: usize,
    pub initial_merges_count: usize,
    pub final_vocab_size: usize,
    pub final_merges_count: usize,
    pub tokens_requested: usize,
    pub tokens_found: usize,
    pub tokens_removed: Vec<TokenRemovalInfo>,
    pub total_tokens_removed: usize,
    pub total_merges_removed: usize,
}

/// Information about a character token addition
#[derive(Debug, Clone, Serialize)]
pub struct CharAddInfo {
    pub char_token: String,
    pub source_id: u32,
}

/// Result of syncing single-char tokens
#[derive(Debug, Serialize)]
pub struct SyncCharsResult {
    pub initial_vocab_size: usize,
    pub initial_merges_count: usize,
    pub final_vocab_size: usize,
    pub final_merges_count: usize,
    pub chars_in_source: usize,
    pub chars_already_present: usize,
    pub chars_added: Vec<CharAddInfo>,
    pub tokens_removed: Vec<TokenRemovalInfo>,
    pub total_tokens_removed: usize,
    pub total_merges_removed: usize,
}

/// Information about a short token addition
#[derive(Debug, Clone, Serialize)]
pub struct ShortTokenAddInfo {
    pub token: String,
    pub source_id: u32,
    pub length: usize,
}

/// Result of syncing short tokens
#[derive(Debug, Serialize)]
pub struct SyncShortTokensResult {
    pub initial_vocab_size: usize,
    pub initial_merges_count: usize,
    pub final_vocab_size: usize,
    pub final_merges_count: usize,
    pub tokens_in_source: usize,
    pub tokens_already_present: usize,
    pub tokens_added: Vec<ShortTokenAddInfo>,
    pub merges_added: usize,
    pub merges_already_present: usize,
    pub tokens_removed: Vec<TokenRemovalInfo>,
    pub total_tokens_removed: usize,
    pub total_merges_removed: usize,
}

/// Information about an ID remapping
#[derive(Debug, Clone, Serialize)]
pub struct IdRemapInfo {
    pub token: String,
    pub old_id: u32,
    pub new_id: u32,
}

/// Result of vocabulary reindexing
#[derive(Debug, Serialize)]
pub struct ReindexResult {
    pub vocab_size: usize,
    pub merges_count: usize,
    pub old_min_id: u32,
    pub old_max_id: u32,
    pub new_min_id: u32,
    pub new_max_id: u32,
    pub ids_remapped: usize,
    pub gaps_removed: usize,
}
