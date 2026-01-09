//! Tokenizer JSON structures for HuggingFace BPE tokenizers

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Main tokenizer structure matching HuggingFace tokenizer.json format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tokenizer {
    pub version: String,
    pub truncation: Option<serde_json::Value>,
    pub padding: Option<serde_json::Value>,
    pub added_tokens: Vec<serde_json::Value>,
    pub normalizer: Option<serde_json::Value>,
    pub pre_tokenizer: Option<serde_json::Value>,
    pub post_processor: Option<serde_json::Value>,
    pub decoder: Option<serde_json::Value>,
    pub model: Model,
}

/// BPE model structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    #[serde(rename = "type")]
    pub model_type: String,
    pub dropout: Option<f64>,
    pub unk_token: String,
    pub continuing_subword_prefix: Option<String>,
    pub end_of_word_suffix: Option<String>,
    pub fuse_unk: bool,
    pub byte_fallback: bool,
    pub ignore_merges: bool,
    pub vocab: BTreeMap<String, u32>,
    pub merges: Vec<Merge>,
}

/// A BPE merge rule (pair of tokens)
#[derive(Debug, Clone)]
pub struct Merge(pub String, pub String);

impl Merge {
    /// Get the result of applying this merge
    pub fn result(&self) -> String {
        format!("{}{}", self.0, self.1)
    }
}

impl Serialize for Merge {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.0)?;
        seq.serialize_element(&self.1)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Merge {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: Vec<String> = Vec::deserialize(deserializer)?;
        if v.len() != 2 {
            return Err(serde::de::Error::custom(
                "Merge must have exactly 2 elements",
            ));
        }
        Ok(Merge(v[0].clone(), v[1].clone()))
    }
}
