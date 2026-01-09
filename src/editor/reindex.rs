//! Vocabulary reindexing to make IDs sequential

use crate::types::ReindexResult;

use super::BPETokenizerEditor;

impl BPETokenizerEditor {
    /// Reindex vocabulary to make all IDs sequential starting from 0.
    ///
    /// This removes all gaps in the vocabulary IDs, creating a dense/compact
    /// ID space. Tokens that already have correct sequential IDs (starting from 0)
    /// are preserved - only tokens after the first gap are remapped.
    ///
    /// Note: Merges don't contain IDs, they only reference token strings,
    /// so they don't need to be updated. Only the vocab map needs reindexing.
    ///
    /// # Returns
    ///
    /// A `ReindexResult` containing statistics about the reindexing operation.
    pub fn reindex_vocab(&mut self) -> ReindexResult {
        let vocab = &self.tokenizer.model.vocab;
        let vocab_size = vocab.len();

        if vocab_size == 0 {
            return ReindexResult {
                vocab_size: 0,
                merges_count: self.tokenizer.model.merges.len(),
                old_min_id: 0,
                old_max_id: 0,
                new_min_id: 0,
                new_max_id: 0,
                ids_remapped: 0,
                gaps_removed: 0,
            };
        }

        // Get current ID range
        let old_min_id = *vocab.values().min().unwrap();
        let old_max_id = *vocab.values().max().unwrap();

        // Calculate current number of gaps
        let current_range = (old_max_id - old_min_id + 1) as usize;
        let gaps_removed = if current_range > vocab_size {
            current_range - vocab_size
        } else {
            0
        };

        // Also count gap at start if min_id > 0
        let gaps_removed = gaps_removed + old_min_id as usize;

        // Sort tokens by their current ID to maintain relative ordering
        let mut token_id_pairs: Vec<(String, u32)> = vocab
            .iter()
            .map(|(token, id)| (token.clone(), *id))
            .collect();
        token_id_pairs.sort_by_key(|(_, id)| *id);

        // Find the first gap - tokens before this point keep their IDs
        // A gap occurs when token[i].id != i
        let mut first_gap_index: Option<usize> = None;
        for (expected_id, (_, actual_id)) in token_id_pairs.iter().enumerate() {
            if *actual_id != expected_id as u32 {
                first_gap_index = Some(expected_id);
                break;
            }
        }

        // If no gap found, vocabulary is already sequential
        let first_gap_index = match first_gap_index {
            Some(idx) => idx,
            None => {
                return ReindexResult {
                    vocab_size,
                    merges_count: self.tokenizer.model.merges.len(),
                    old_min_id,
                    old_max_id,
                    new_min_id: 0,
                    new_max_id: (vocab_size - 1) as u32,
                    ids_remapped: 0,
                    gaps_removed: 0,
                };
            }
        };

        // Count how many tokens will get a new ID (only those after first gap)
        let ids_remapped = vocab_size - first_gap_index;

        // Create new vocab: preserve IDs before first gap, remap after
        let mut new_vocab = std::collections::BTreeMap::new();
        for (new_id, (token, old_id)) in token_id_pairs.into_iter().enumerate() {
            if new_id < first_gap_index {
                // Keep original ID (it's already correct)
                new_vocab.insert(token, old_id);
            } else {
                // Assign new sequential ID
                new_vocab.insert(token, new_id as u32);
            }
        }

        // Replace the vocab
        self.tokenizer.model.vocab = new_vocab;

        // Update internal state
        self.used_ids.clear();
        for id in self.tokenizer.model.vocab.values() {
            self.used_ids.insert(*id);
        }
        self.next_id = vocab_size as u32;

        // New ID range
        let new_min_id = 0;
        let new_max_id = (vocab_size - 1) as u32;

        ReindexResult {
            vocab_size,
            merges_count: self.tokenizer.model.merges.len(),
            old_min_id,
            old_max_id,
            new_min_id,
            new_max_id,
            ids_remapped,
            gaps_removed,
        }
    }

    /// Check if vocabulary has gaps in its ID space
    ///
    /// Returns (has_gaps, total_gaps, min_id, max_id)
    pub fn check_vocab_gaps(&self) -> (bool, usize, u32, u32) {
        let vocab = &self.tokenizer.model.vocab;

        if vocab.is_empty() {
            return (false, 0, 0, 0);
        }

        let min_id = *vocab.values().min().unwrap();
        let max_id = *vocab.values().max().unwrap();
        let vocab_size = vocab.len();

        // Dense range would be: vocab_size tokens
        // Current range: max_id - min_id + 1
        let current_range = (max_id - min_id + 1) as usize;

        // Also check if min_id is not 0 (meaning there are gaps at the start)
        let gaps_at_start = min_id as usize;
        let gaps_in_range = if current_range > vocab_size {
            current_range - vocab_size
        } else {
            0
        };

        let total_gaps = gaps_at_start + gaps_in_range;
        let has_gaps = total_gaps > 0;

        (has_gaps, total_gaps, min_id, max_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{Merge, Model, Tokenizer};
    use std::collections::BTreeMap;

    fn create_test_tokenizer(vocab: Vec<(&str, u32)>, merges: Vec<(&str, &str)>) -> Tokenizer {
        let vocab_map: BTreeMap<String, u32> =
            vocab.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        let merge_list: Vec<Merge> = merges
            .into_iter()
            .map(|(a, b)| Merge(a.to_string(), b.to_string()))
            .collect();

        Tokenizer {
            version: "1.0".to_string(),
            truncation: None,
            padding: None,
            added_tokens: vec![],
            normalizer: None,
            pre_tokenizer: None,
            post_processor: None,
            decoder: None,
            model: Model {
                model_type: "BPE".to_string(),
                dropout: None,
                unk_token: "<unk>".to_string(),
                continuing_subword_prefix: None,
                end_of_word_suffix: None,
                fuse_unk: false,
                byte_fallback: false,
                ignore_merges: false,
                vocab: vocab_map,
                merges: merge_list,
            },
        }
    }

    #[test]
    fn test_reindex_with_gaps() {
        let tokenizer = create_test_tokenizer(
            vec![
                ("a", 0),
                ("b", 5),    // gap: 1-4
                ("c", 10),   // gap: 6-9
                ("ab", 100), // gap: 11-99
            ],
            vec![("a", "b")],
        );

        let mut editor = BPETokenizerEditor::new(tokenizer);

        // Check gaps before
        let (has_gaps, total_gaps, min_id, max_id) = editor.check_vocab_gaps();
        assert!(has_gaps);
        assert_eq!(min_id, 0);
        assert_eq!(max_id, 100);
        assert!(total_gaps > 0);

        // Reindex
        let result = editor.reindex_vocab();

        assert_eq!(result.vocab_size, 4);
        assert_eq!(result.old_min_id, 0);
        assert_eq!(result.old_max_id, 100);
        assert_eq!(result.new_min_id, 0);
        assert_eq!(result.new_max_id, 3);
        assert!(result.gaps_removed > 0);
        // Only tokens after ID 0 are remapped (b, c, ab = 3 tokens)
        assert_eq!(result.ids_remapped, 3);

        // Verify new IDs are sequential, "a" keeps its ID
        let vocab = &editor.tokenizer.model.vocab;
        assert_eq!(vocab.get("a"), Some(&0)); // preserved
        assert_eq!(vocab.get("b"), Some(&1)); // remapped from 5
        assert_eq!(vocab.get("c"), Some(&2)); // remapped from 10
        assert_eq!(vocab.get("ab"), Some(&3)); // remapped from 100

        // Check no gaps after
        let (has_gaps, _, _, _) = editor.check_vocab_gaps();
        assert!(!has_gaps);
    }

    #[test]
    fn test_reindex_preserves_correct_prefix() {
        // Test that tokens with correct IDs at the start are preserved
        let tokenizer = create_test_tokenizer(
            vec![
                ("a", 0),  // correct
                ("b", 1),  // correct
                ("c", 2),  // correct
                ("d", 10), // gap starts here (should be 3)
                ("e", 20), // should become 4
            ],
            vec![],
        );

        let mut editor = BPETokenizerEditor::new(tokenizer);

        let result = editor.reindex_vocab();

        // Only d and e should be remapped
        assert_eq!(result.ids_remapped, 2);

        let vocab = &editor.tokenizer.model.vocab;
        assert_eq!(vocab.get("a"), Some(&0)); // preserved
        assert_eq!(vocab.get("b"), Some(&1)); // preserved
        assert_eq!(vocab.get("c"), Some(&2)); // preserved
        assert_eq!(vocab.get("d"), Some(&3)); // remapped from 10
        assert_eq!(vocab.get("e"), Some(&4)); // remapped from 20
    }

    #[test]
    fn test_reindex_already_sequential() {
        let tokenizer = create_test_tokenizer(vec![("a", 0), ("b", 1), ("c", 2)], vec![]);

        let mut editor = BPETokenizerEditor::new(tokenizer);

        let (has_gaps, _, _, _) = editor.check_vocab_gaps();
        assert!(!has_gaps);

        let result = editor.reindex_vocab();

        assert_eq!(result.ids_remapped, 0);
        assert_eq!(result.gaps_removed, 0);
    }

    #[test]
    fn test_reindex_starting_from_nonzero() {
        let tokenizer = create_test_tokenizer(vec![("a", 100), ("b", 101), ("c", 102)], vec![]);

        let mut editor = BPETokenizerEditor::new(tokenizer);

        let (has_gaps, total_gaps, _, _) = editor.check_vocab_gaps();
        assert!(has_gaps);
        assert_eq!(total_gaps, 100); // IDs 0-99 are missing

        let result = editor.reindex_vocab();

        assert_eq!(result.old_min_id, 100);
        assert_eq!(result.new_min_id, 0);
        assert_eq!(result.ids_remapped, 3);

        let vocab = &editor.tokenizer.model.vocab;
        assert_eq!(vocab.get("a"), Some(&0));
        assert_eq!(vocab.get("b"), Some(&1));
        assert_eq!(vocab.get("c"), Some(&2));
    }
}
