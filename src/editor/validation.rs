//! Merge validation methods

use std::collections::HashSet;

use crate::tokenizer::Merge;

use super::core::BPETokenizerEditor;

impl BPETokenizerEditor {
    /// Validate all merges - check that the result of each merge exists in vocab
    pub fn validate_merges(&self) -> (Vec<usize>, Vec<(usize, Merge)>) {
        let mut valid_indices = Vec::new();
        let mut invalid = Vec::new();

        for (i, merge) in self.tokenizer.model.merges.iter().enumerate() {
            let result = merge.result();
            if self.tokenizer.model.vocab.contains_key(&result) {
                valid_indices.push(i);
            } else {
                invalid.push((i, merge.clone()));
            }
        }

        (valid_indices, invalid)
    }

    /// Remove all invalid merges from the tokenizer
    pub fn remove_invalid_merges(&mut self) -> usize {
        let (valid_indices, invalid) = self.validate_merges();
        let removed_count = invalid.len();

        if removed_count > 0 {
            let valid_set: HashSet<usize> = valid_indices.into_iter().collect();
            self.tokenizer.model.merges = self
                .tokenizer
                .model
                .merges
                .iter()
                .enumerate()
                .filter(|(i, _)| valid_set.contains(i))
                .map(|(_, m)| m.clone())
                .collect();
            self.rebuild_indices();
        }

        removed_count
    }
}
