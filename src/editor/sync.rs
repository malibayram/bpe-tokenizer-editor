//! Sync operations between tokenizers

use std::collections::HashMap;

use crate::types::{
    CharAddInfo, ShortTokenAddInfo, SyncCharsResult, SyncShortTokensResult, TokenRemovalInfo,
};

use super::core::BPETokenizerEditor;

impl BPETokenizerEditor {
    /// Sync single-letter tokens from source, removing longest tokens to keep size constant
    pub fn sync_single_chars(
        &mut self,
        source_chars: &[(String, u32)],
        min_id: u32,
    ) -> SyncCharsResult {
        let initial_vocab_size = self.vocab_size();
        let initial_merges_count = self.merges_count();

        let mut result = SyncCharsResult {
            initial_vocab_size,
            initial_merges_count,
            final_vocab_size: 0,
            final_merges_count: 0,
            chars_in_source: source_chars.len(),
            chars_already_present: 0,
            chars_added: vec![],
            tokens_removed: vec![],
            total_tokens_removed: 0,
            total_merges_removed: 0,
        };

        let chars_to_add: Vec<_> = source_chars
            .iter()
            .filter(|(char_tok, _)| !self.has_token(char_tok))
            .cloned()
            .collect();

        result.chars_already_present = source_chars.len() - chars_to_add.len();
        let total_to_add = chars_to_add.len();

        if total_to_add == 0 {
            result.final_vocab_size = self.vocab_size();
            result.final_merges_count = self.merges_count();
            return result;
        }

        eprintln!("\n   Pre-computing {} tokens to remove...", total_to_add);
        let tokens_to_remove = self.find_tokens_to_shrink(total_to_add, min_id);
        eprintln!(
            "   Found {} candidate tokens for removal",
            tokens_to_remove.len()
        );

        if tokens_to_remove.len() < total_to_add {
            eprintln!(
                "   WARNING: Only found {} tokens to remove, but need {}",
                tokens_to_remove.len(),
                total_to_add
            );
        }

        // Phase 1: Remove tokens
        eprintln!(
            "\n   Phase 1: Removing {} tokens...",
            tokens_to_remove.len()
        );
        let start_time = std::time::Instant::now();
        let mut last_print = std::time::Instant::now();

        for (i, (token, id, len)) in tokens_to_remove.iter().enumerate() {
            if !self.has_token(token) {
                continue;
            }

            let removal = self.remove_token_and_dependencies(token);

            result.tokens_removed.push(TokenRemovalInfo {
                token: token.clone(),
                id: *id,
                length: *len,
                cascade_tokens_removed: removal.removed_tokens.len(),
                merges_removed: removal.removed_merges.len(),
            });

            result.total_tokens_removed += removal.removed_tokens.len();
            result.total_merges_removed += removal.removed_merges.len();

            let now = std::time::Instant::now();
            if now.duration_since(last_print).as_millis() >= 100
                || (i + 1) == tokens_to_remove.len()
            {
                let elapsed = start_time.elapsed().as_secs_f64();
                let progress = (i + 1) as f64 / tokens_to_remove.len() as f64;
                let eta = if progress > 0.0 {
                    elapsed / progress - elapsed
                } else {
                    0.0
                };
                eprint!(
                    "\r   [{:5.1}%] {}/{} removed | Elapsed: {:.1}s | ETA: {:.1}s    ",
                    progress * 100.0,
                    i + 1,
                    tokens_to_remove.len(),
                    elapsed,
                    eta
                );
                last_print = now;
            }
        }
        eprintln!();

        // Phase 2: Add single-char tokens
        eprintln!(
            "\n   Phase 2: Adding {} single-char tokens...",
            total_to_add
        );
        let start_time = std::time::Instant::now();

        for (char_tok, source_id) in &chars_to_add {
            self.add_token_atomic(char_tok);
            result.chars_added.push(CharAddInfo {
                char_token: char_tok.clone(),
                source_id: *source_id,
            });
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        eprintln!(
            "   Added {} tokens in {:.2}s",
            result.chars_added.len(),
            elapsed
        );

        // Phase 3: Reindex to remove gaps
        eprintln!("\n   Phase 3: Reindexing vocabulary to remove ID gaps...");
        let reindex_result = self.reindex_vocab();
        eprintln!(
            "   Reindexed {} IDs, removed {} gaps",
            reindex_result.ids_remapped, reindex_result.gaps_removed
        );

        result.final_vocab_size = self.vocab_size();
        result.final_merges_count = self.merges_count();
        result
    }

    /// Sync short tokens (2-3 chars) from source, including their merges
    pub fn sync_short_tokens(
        &mut self,
        source_tokens: &[(String, u32)],
        source_merges: &[(String, String)],
        min_id: u32,
    ) -> SyncShortTokensResult {
        let initial_vocab_size = self.vocab_size();
        let initial_merges_count = self.merges_count();

        let mut result = SyncShortTokensResult {
            initial_vocab_size,
            initial_merges_count,
            final_vocab_size: 0,
            final_merges_count: 0,
            tokens_in_source: source_tokens.len(),
            tokens_already_present: 0,
            tokens_added: vec![],
            merges_added: 0,
            merges_already_present: 0,
            tokens_removed: vec![],
            total_tokens_removed: 0,
            total_merges_removed: 0,
        };

        // Build source merge map: result -> (a, b)
        let source_merge_map: HashMap<String, (String, String)> = source_merges
            .iter()
            .map(|(a, b)| (format!("{}{}", a, b), (a.clone(), b.clone())))
            .collect();

        let current_merges = self.get_merge_set();

        let tokens_to_add: Vec<_> = source_tokens
            .iter()
            .filter(|(tok, _)| !self.has_token(tok))
            .cloned()
            .collect();

        result.tokens_already_present = source_tokens.len() - tokens_to_add.len();
        let total_to_add = tokens_to_add.len();

        if total_to_add == 0 {
            result.final_vocab_size = self.vocab_size();
            result.final_merges_count = self.merges_count();
            return result;
        }

        // Phase 1: Pre-compute and remove tokens
        eprintln!("\n   Pre-computing {} tokens to remove...", total_to_add);
        let tokens_to_remove = self.find_tokens_to_shrink(total_to_add, min_id);
        eprintln!(
            "   Found {} candidate tokens for removal",
            tokens_to_remove.len()
        );

        eprintln!(
            "\n   Phase 1: Removing {} tokens...",
            tokens_to_remove.len()
        );
        let start_time = std::time::Instant::now();
        let mut last_print = std::time::Instant::now();

        for (i, (token, id, len)) in tokens_to_remove.iter().enumerate() {
            if !self.has_token(token) {
                continue;
            }

            let removal = self.remove_token_and_dependencies(token);

            result.tokens_removed.push(TokenRemovalInfo {
                token: token.clone(),
                id: *id,
                length: *len,
                cascade_tokens_removed: removal.removed_tokens.len(),
                merges_removed: removal.removed_merges.len(),
            });

            result.total_tokens_removed += removal.removed_tokens.len();
            result.total_merges_removed += removal.removed_merges.len();

            let now = std::time::Instant::now();
            if now.duration_since(last_print).as_millis() >= 100
                || (i + 1) == tokens_to_remove.len()
            {
                let elapsed = start_time.elapsed().as_secs_f64();
                let progress = (i + 1) as f64 / tokens_to_remove.len() as f64;
                let eta = if progress > 0.0 {
                    elapsed / progress - elapsed
                } else {
                    0.0
                };
                eprint!(
                    "\r   [{:5.1}%] {}/{} removed | Elapsed: {:.1}s | ETA: {:.1}s    ",
                    progress * 100.0,
                    i + 1,
                    tokens_to_remove.len(),
                    elapsed,
                    eta
                );
                last_print = now;
            }
        }
        eprintln!();

        // Phase 2: Add tokens and their merges
        eprintln!(
            "\n   Phase 2: Adding {} tokens with merges...",
            total_to_add
        );
        let start_time = std::time::Instant::now();
        let mut last_print = std::time::Instant::now();

        for (i, (token, source_id)) in tokens_to_add.iter().enumerate() {
            // Check if we need to add the merge that produces this token
            if let Some((a, b)) = source_merge_map.get(token) {
                if !self.has_token(a) {
                    self.add_token_atomic(a);
                }
                if !self.has_token(b) {
                    self.add_token_atomic(b);
                }

                if !current_merges.contains(&(a.clone(), b.clone())) {
                    if self.add_merge_if_missing(a, b) {
                        result.merges_added += 1;
                    }
                } else {
                    result.merges_already_present += 1;
                }
            }

            self.add_token_atomic(token);
            result.tokens_added.push(ShortTokenAddInfo {
                token: token.clone(),
                source_id: *source_id,
                length: token.chars().count(),
            });

            let now = std::time::Instant::now();
            if now.duration_since(last_print).as_millis() >= 100 || (i + 1) == total_to_add {
                let elapsed = start_time.elapsed().as_secs_f64();
                let progress = (i + 1) as f64 / total_to_add as f64;
                let eta = if progress > 0.0 {
                    elapsed / progress - elapsed
                } else {
                    0.0
                };
                eprint!(
                    "\r   [{:5.1}%] {}/{} added | Merges: {} | Elapsed: {:.1}s | ETA: {:.1}s    ",
                    progress * 100.0,
                    i + 1,
                    total_to_add,
                    result.merges_added,
                    elapsed,
                    eta
                );
                last_print = now;
            }
        }
        eprintln!();

        // Phase 3: Reindex to remove gaps
        eprintln!("\n   Phase 3: Reindexing vocabulary to remove ID gaps...");
        let reindex_result = self.reindex_vocab();
        eprintln!(
            "   Reindexed {} IDs, removed {} gaps",
            reindex_result.ids_remapped, reindex_result.gaps_removed
        );

        result.final_vocab_size = self.vocab_size();
        result.final_merges_count = self.merges_count();
        result
    }
}
