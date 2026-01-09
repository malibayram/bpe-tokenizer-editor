//! Token addition methods

use std::collections::HashSet;

use crate::tokenizer::Merge;
use crate::types::AdditionResult;

use super::core::BPETokenizerEditor;

impl BPETokenizerEditor {
    /// Add a token atomically (no merges) - for special tokens and single chars
    pub fn add_token_atomic(&mut self, token: &str) -> bool {
        if self.has_token(token) {
            return false;
        }
        let id = self.get_next_id();
        self.tokenizer.model.vocab.insert(token.to_string(), id);
        true
    }

    /// Add a token with proper merge chain (longest prefix strategy)
    pub fn add_token_with_merges(&mut self, token: &str) -> AdditionResult {
        if self.has_token(token) {
            return AdditionResult {
                token: token.to_string(),
                added: false,
                method: "already_exists".to_string(),
                added_merges: vec![],
            };
        }

        let chars: Vec<char> = token.chars().collect();
        if chars.len() == 1 {
            self.add_token_atomic(token);
            return AdditionResult {
                token: token.to_string(),
                added: true,
                method: "single_char".to_string(),
                added_merges: vec![],
            };
        }

        // Find longest prefix in vocab
        let mut prefix: Option<String> = None;
        for i in (1..=token.len()).rev() {
            let p = &token[..i];
            if self.has_token(p) {
                prefix = Some(p.to_string());
                break;
            }
        }

        let added_merges = if let Some(ref pref) = prefix {
            let suffix = &token[pref.len()..];
            if suffix.is_empty() {
                vec![]
            } else {
                self.build_suffix_and_merge(pref, suffix)
            }
        } else {
            self.build_char_chain(token)
        };

        if !self.has_token(token) {
            let id = self.get_next_id();
            self.tokenizer.model.vocab.insert(token.to_string(), id);
        }

        self.rebuild_indices();

        AdditionResult {
            token: token.to_string(),
            added: true,
            method: if prefix.is_some() {
                "longest_prefix"
            } else {
                "char_chain"
            }
            .to_string(),
            added_merges,
        }
    }

    /// Build a char chain for a token: a+b -> ab, ab+c -> abc, ...
    pub(crate) fn build_char_chain(&mut self, token: &str) -> Vec<(String, String)> {
        let chars: Vec<char> = token.chars().collect();
        if chars.is_empty() {
            return vec![];
        }

        let mut added_merges = vec![];
        let mut current = chars[0].to_string();

        if !self.has_token(&current) {
            let id = self.get_next_id();
            self.tokenizer.model.vocab.insert(current.clone(), id);
        }

        for ch in chars.iter().skip(1) {
            let ch_str = ch.to_string();

            if !self.has_token(&ch_str) {
                let id = self.get_next_id();
                self.tokenizer.model.vocab.insert(ch_str.clone(), id);
            }

            let new_token = format!("{}{}", current, ch_str);
            let merge_exists = self
                .tokenizer
                .model
                .merges
                .iter()
                .any(|m| m.0 == current && m.1 == ch_str);

            if !merge_exists {
                added_merges.push((current.clone(), ch_str.clone()));
                self.tokenizer
                    .model
                    .merges
                    .push(Merge(current.clone(), ch_str.clone()));
            }

            if !self.has_token(&new_token) {
                let id = self.get_next_id();
                self.tokenizer.model.vocab.insert(new_token.clone(), id);
            }

            current = new_token;
        }

        added_merges
    }

    /// Build suffix via char chain, then add merge (prefix, suffix) -> token
    fn build_suffix_and_merge(&mut self, prefix: &str, suffix: &str) -> Vec<(String, String)> {
        let mut added_merges = vec![];

        if !self.has_token(suffix) {
            let suffix_merges = self.build_char_chain(suffix);
            added_merges.extend(suffix_merges);
        }

        let merge_exists = self
            .tokenizer
            .model
            .merges
            .iter()
            .any(|m| m.0 == prefix && m.1 == suffix);

        if !merge_exists {
            added_merges.push((prefix.to_string(), suffix.to_string()));
            self.tokenizer
                .model
                .merges
                .push(Merge(prefix.to_string(), suffix.to_string()));
        }

        added_merges
    }

    /// Add a merge if it doesn't exist
    pub fn add_merge_if_missing(&mut self, a: &str, b: &str) -> bool {
        let merge_exists = self
            .tokenizer
            .model
            .merges
            .iter()
            .any(|m| m.0 == a && m.1 == b);

        if !merge_exists {
            self.tokenizer
                .model
                .merges
                .push(Merge(a.to_string(), b.to_string()));
            self.rebuild_indices();
            true
        } else {
            false
        }
    }

    /// Get all merges as a HashSet for fast lookup
    pub fn get_merge_set(&self) -> HashSet<(String, String)> {
        self.tokenizer
            .model
            .merges
            .iter()
            .map(|m| (m.0.clone(), m.1.clone()))
            .collect()
    }
}
