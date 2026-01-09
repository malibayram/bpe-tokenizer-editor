//! Vocab size management methods

use std::collections::HashSet;

use crate::types::{BatchAddResult, ShrinkResult, TokenRemovalInfo};

use super::core::BPETokenizerEditor;

impl BPETokenizerEditor {
    /// Build a set of protected tokens that should never be removed
    pub fn build_protected_set(&self, extra: &HashSet<String>) -> HashSet<String> {
        let mut protected = extra.clone();

        // Protect single chars
        for tok in self.tokenizer.model.vocab.keys() {
            if tok.chars().count() == 1 {
                protected.insert(tok.clone());
            }
        }

        // Protect space marker
        protected.insert("▁".to_string());

        // Protect special tokens
        for tok in self.tokenizer.model.vocab.keys() {
            if (tok.starts_with('<') && tok.ends_with('>'))
                || (tok.starts_with('[') && tok.ends_with(']'))
            {
                protected.insert(tok.clone());
            }
        }

        protected
    }

    /// Select a token to remove based on heuristics
    pub fn select_token_to_remove(&self, protected: &HashSet<String>) -> Option<(String, String)> {
        let mut candidates: Vec<(usize, bool, &String)> = vec![];

        for tok in self.tokenizer.model.vocab.keys() {
            if protected.contains(tok) {
                continue;
            }
            let len = tok.chars().count();
            let is_produced = self.producer.contains_key(tok);
            candidates.push((len, is_produced, tok));
        }

        if candidates.is_empty() {
            return None;
        }

        // Sort: longest first, produced first, then alphabetic for stability
        candidates.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(b.2)));

        let (len, produced, tok) = candidates[0];
        let reason = format!(
            "Removed to keep vocab size fixed. len={}, merge_produced={}",
            len, produced
        );

        Some((tok.clone(), reason))
    }

    /// Find N tokens to remove: longest non-special tokens with ID >= min_id
    pub fn find_tokens_to_shrink(&self, count: usize, min_id: u32) -> Vec<(String, u32, usize)> {
        let mut candidates: Vec<(String, u32, usize)> = vec![];

        for (tok, &id) in &self.tokenizer.model.vocab {
            if id < min_id {
                continue;
            }

            let char_len = tok.chars().count();
            if char_len <= 1 {
                continue;
            }

            if (tok.starts_with('<') && tok.ends_with('>'))
                || (tok.starts_with('[') && tok.ends_with(']'))
            {
                continue;
            }

            candidates.push((tok.clone(), id, char_len));
        }

        // Sort by length DESC, then by ID DESC
        candidates.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)));

        candidates.into_iter().take(count).collect()
    }

    /// Remove N tokens by finding longest non-special tokens with highest IDs
    pub fn shrink_vocab(&mut self, count: usize, min_id: u32) -> ShrinkResult {
        let tokens_to_remove = self.find_tokens_to_shrink(count, min_id);

        let mut result = ShrinkResult {
            initial_vocab_size: self.vocab_size(),
            initial_merges_count: self.merges_count(),
            final_vocab_size: 0,
            final_merges_count: 0,
            tokens_requested: count,
            tokens_found: tokens_to_remove.len(),
            tokens_removed: vec![],
            total_tokens_removed: 0,
            total_merges_removed: 0,
        };

        for (token, id, len) in &tokens_to_remove {
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
        }

        result.final_vocab_size = self.vocab_size();
        result.final_merges_count = self.merges_count();
        result
    }

    /// Get all single-letter tokens from this tokenizer
    pub fn get_single_char_tokens(&self) -> Vec<(String, u32)> {
        self.tokenizer
            .model
            .vocab
            .iter()
            .filter(|(tok, _)| tok.chars().count() == 1)
            .map(|(tok, &id)| (tok.clone(), id))
            .collect()
    }

    /// Get tokens of specified length range with their IDs
    pub fn get_tokens_by_length(&self, min_len: usize, max_len: usize) -> Vec<(String, u32)> {
        self.tokenizer
            .model
            .vocab
            .iter()
            .filter(|(tok, _)| {
                let len = tok.chars().count();
                len >= min_len && len <= max_len
            })
            .map(|(tok, &id)| (tok.clone(), id))
            .collect()
    }

    /// Reassign sequential IDs to vocab
    pub fn reassign_ids(&mut self) {
        let mut sorted_tokens: Vec<_> = self.tokenizer.model.vocab.keys().cloned().collect();
        sorted_tokens
            .sort_by(|a, b| self.tokenizer.model.vocab[a].cmp(&self.tokenizer.model.vocab[b]));

        self.tokenizer.model.vocab.clear();
        self.used_ids.clear();

        for (i, tok) in sorted_tokens.into_iter().enumerate() {
            let id = i as u32;
            self.tokenizer.model.vocab.insert(tok, id);
            self.used_ids.insert(id);
        }

        self.next_id = self.used_ids.len() as u32;
    }

    /// Add tokens while keeping vocab size fixed
    pub fn add_tokens_keep_size(
        &mut self,
        tokens: &[String],
        whitelist: &HashSet<String>,
    ) -> BatchAddResult {
        let initial_size = self.vocab_size();
        let mut result = BatchAddResult {
            initial_vocab_size: initial_size,
            final_vocab_size: 0,
            tokens_requested: tokens.len(),
            tokens_added: 0,
            tokens_already_present: 0,
            merges_added: 0,
            tokens_removed: 0,
            additions: vec![],
            removals: vec![],
        };

        let mut protected = self.build_protected_set(whitelist);
        protected.extend(tokens.iter().cloned());

        for (i, token) in tokens.iter().enumerate() {
            if (i + 1) % 100 == 0 || i == tokens.len() - 1 {
                eprintln!(
                    "   [{:5.1}%] Processed {}/{} | Added: {} | Removed: {}",
                    (i + 1) as f64 / tokens.len() as f64 * 100.0,
                    i + 1,
                    tokens.len(),
                    result.tokens_added,
                    result.tokens_removed
                );
            }

            if self.has_token(token) {
                result.tokens_already_present += 1;
                continue;
            }

            let add_result = self.add_token_with_merges(token);
            if add_result.added {
                result.tokens_added += 1;
                result.merges_added += add_result.added_merges.len();
                result.additions.push(add_result);
            }

            while self.vocab_size() > initial_size {
                if let Some((victim, reason)) = self.select_token_to_remove(&protected) {
                    let removal = self.remove_token_and_dependencies(&victim);
                    result.tokens_removed += removal.removed_tokens.len();
                    result.removals.push((removal, reason));
                } else {
                    eprintln!("Warning: Cannot find token to remove. Protected set too large?");
                    break;
                }
            }
        }

        result.final_vocab_size = self.vocab_size();
        result
    }
}
