//! Core BPE Tokenizer Editor struct and basic methods

use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::tokenizer::Tokenizer;

/// BPE Tokenizer Editor with consistency guarantees
pub struct BPETokenizerEditor {
    pub tokenizer: Tokenizer,
    // Indices for fast lookup
    pub(crate) producer: HashMap<String, usize>, // token -> merge index that produces it
    pub(crate) uses: HashMap<String, HashSet<usize>>, // token -> merge indices where used as input
    pub(crate) used_ids: HashSet<u32>,
    pub(crate) next_id: u32,
}

impl BPETokenizerEditor {
    /// Create a new editor from a Tokenizer
    pub fn new(tokenizer: Tokenizer) -> Self {
        let used_ids: HashSet<u32> = tokenizer.model.vocab.values().copied().collect();
        let next_id = used_ids.iter().max().copied().unwrap_or(0) + 1;

        let mut editor = Self {
            tokenizer,
            producer: HashMap::new(),
            uses: HashMap::new(),
            used_ids,
            next_id,
        };
        editor.rebuild_indices();
        editor
    }

    /// Load a tokenizer from a JSON file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read: {:?}", path))?;
        let tokenizer: Tokenizer =
            serde_json::from_str(&content).with_context(|| "Failed to parse tokenizer.json")?;

        if tokenizer.model.model_type != "BPE" {
            bail!("Only BPE tokenizers are supported");
        }

        Ok(Self::new(tokenizer))
    }

    /// Save the tokenizer to a JSON file (vocab sorted by ID ascending)
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let mut vocab_vec: Vec<(&String, &u32)> = self.tokenizer.model.vocab.iter().collect();
        vocab_vec.sort_by_key(|(_, id)| *id);

        let vocab_ordered: serde_json::Map<String, serde_json::Value> = vocab_vec
            .into_iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::Number((*v).into())))
            .collect();

        let mut json_value = serde_json::to_value(&self.tokenizer)?;
        json_value["model"]["vocab"] = serde_json::Value::Object(vocab_ordered);

        let content = serde_json::to_string_pretty(&json_value)
            .with_context(|| "Failed to serialize tokenizer")?;
        fs::write(path, content).with_context(|| format!("Failed to write: {:?}", path))?;
        Ok(())
    }

    /// Rebuild internal indices for fast lookups
    pub fn rebuild_indices(&mut self) {
        self.producer.clear();
        self.uses.clear();

        for (i, merge) in self.tokenizer.model.merges.iter().enumerate() {
            let prod = merge.result();
            self.producer.entry(prod).or_insert(i);
            self.uses.entry(merge.0.clone()).or_default().insert(i);
            self.uses.entry(merge.1.clone()).or_default().insert(i);
        }
    }

    /// Get the current vocab size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.model.vocab.len()
    }

    /// Get the current number of merges
    pub fn merges_count(&self) -> usize {
        self.tokenizer.model.merges.len()
    }

    /// Check if a token exists in the vocabulary
    pub fn has_token(&self, token: &str) -> bool {
        self.tokenizer.model.vocab.contains_key(token)
    }

    pub(crate) fn get_next_id(&mut self) -> u32 {
        while self.used_ids.contains(&self.next_id) {
            self.next_id += 1;
        }
        let id = self.next_id;
        self.used_ids.insert(id);
        self.next_id += 1;
        id
    }

    pub(crate) fn release_id(&mut self, id: u32) {
        self.used_ids.remove(&id);
    }
}
