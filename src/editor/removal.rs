//! Token removal methods

use std::collections::HashSet;

use crate::types::RemovalResult;

use super::core::BPETokenizerEditor;

impl BPETokenizerEditor {
    /// Remove a token and all its dependencies (merges that use it, etc.)
    pub fn remove_token_and_dependencies(&mut self, token: &str) -> RemovalResult {
        let mut removed_tokens: HashSet<String> = HashSet::new();
        let mut removed_merge_indices: HashSet<usize> = HashSet::new();
        let mut stack = vec![token.to_string()];

        while let Some(t) = stack.pop() {
            if removed_tokens.contains(&t) {
                continue;
            }
            removed_tokens.insert(t.clone());

            // Find merges that use this token as input
            if let Some(indices) = self.uses.get(&t) {
                for &mi in indices {
                    if !removed_merge_indices.contains(&mi) {
                        removed_merge_indices.insert(mi);
                        let prod = self.tokenizer.model.merges[mi].result();
                        stack.push(prod);
                    }
                }
            }

            // Find merge that produces this token
            if let Some(&mi) = self.producer.get(&t) {
                removed_merge_indices.insert(mi);
            }
        }

        // Collect merge pairs before removing
        let removed_merges: Vec<(String, String)> = removed_merge_indices
            .iter()
            .map(|&i| {
                let m = &self.tokenizer.model.merges[i];
                (m.0.clone(), m.1.clone())
            })
            .collect();

        // Remove merges
        if !removed_merge_indices.is_empty() {
            self.tokenizer.model.merges = self
                .tokenizer
                .model
                .merges
                .iter()
                .enumerate()
                .filter(|(i, _)| !removed_merge_indices.contains(i))
                .map(|(_, m)| m.clone())
                .collect();
        }

        // Remove tokens from vocab
        for t in &removed_tokens {
            if let Some(&id) = self.tokenizer.model.vocab.get(t) {
                self.release_id(id);
                self.tokenizer.model.vocab.remove(t);
            }
        }

        self.rebuild_indices();

        RemovalResult {
            root_token: token.to_string(),
            removed_tokens: removed_tokens.into_iter().collect(),
            removed_merges,
        }
    }
}
