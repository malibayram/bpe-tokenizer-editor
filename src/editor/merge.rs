//! Native-format tokenizer merging.

use anyhow::{bail, Result};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use unicode_normalization::UnicodeNormalization;

use crate::tokenizer::{Merge, Tokenizer};
use crate::types::MergeTokenizerResult;

use super::core::BPETokenizerEditor;

struct ConvertedSource {
    token_map: HashMap<String, String>,
    injection_order: Vec<String>,
    bridge_tokens: HashSet<String>,
    bridge_rules: Vec<Merge>,
    representation: String,
}

fn contains_type(value: &Value, wanted: &str) -> bool {
    match value {
        Value::Object(object) => {
            object.get("type").and_then(Value::as_str) == Some(wanted)
                || object.values().any(|child| contains_type(child, wanted))
        }
        Value::Array(array) => array.iter().any(|child| contains_type(child, wanted)),
        _ => false,
    }
}

fn find_space_marker(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => {
            let is_space_replace = object.get("type").and_then(Value::as_str) == Some("Replace")
                && object
                    .get("pattern")
                    .and_then(Value::as_object)
                    .and_then(|pattern| pattern.get("String"))
                    .and_then(Value::as_str)
                    == Some(" ");
            if is_space_replace {
                return object
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
            }
            object.values().find_map(find_space_marker)
        }
        Value::Array(array) => array.iter().find_map(find_space_marker),
        _ => None,
    }
}

fn parse_byte_fallback(token: &str) -> Option<u8> {
    let hex = token.strip_prefix("<0x")?.strip_suffix('>')?;
    (hex.len() == 2)
        .then(|| u8::from_str_radix(hex, 16).ok())
        .flatten()
}

fn byte_level_alphabet() -> [char; 256] {
    let mut alphabet = ['\0'; 256];
    let mut extra = 0u32;
    for byte in 0u16..=255 {
        let visible = (33..=126).contains(&byte)
            || (161..=172).contains(&byte)
            || (174..=255).contains(&byte);
        let codepoint = if visible {
            byte as u32
        } else {
            let codepoint = 256 + extra;
            extra += 1;
            codepoint
        };
        alphabet[byte as usize] = char::from_u32(codepoint).expect("valid ByteLevel codepoint");
    }
    alphabet
}

fn added_token_content(value: &Value) -> Option<&str> {
    value.get("content").and_then(Value::as_str)
}

fn set_added_token_id(value: &mut Value, id: u32) -> Result<()> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("added_tokens entries must be JSON objects"))?;
    object.insert("id".to_owned(), Value::Number(id.into()));
    Ok(())
}

fn is_special(token: &str) -> bool {
    (token.starts_with('<') && token.ends_with('>'))
        || (token.starts_with('[') && token.ends_with(']'))
}

fn push_rule(
    left: &str,
    right: &str,
    selected: &HashSet<String>,
    seen: &mut HashSet<(String, String)>,
    rules: &mut Vec<Merge>,
) -> bool {
    let pair = (left.to_owned(), right.to_owned());
    if seen.contains(&pair)
        || !selected.contains(left)
        || !selected.contains(right)
        || !selected.contains(&format!("{}{}", left, right))
    {
        return false;
    }
    seen.insert(pair.clone());
    rules.push(Merge(pair.0, pair.1));
    true
}

fn assign_token_id(
    token: &String,
    selected: &HashSet<String>,
    vocab: &mut BTreeMap<String, u32>,
    free_ids: &mut VecDeque<u32>,
) -> Result<()> {
    if selected.contains(token) && !vocab.contains_key(token) {
        let id = free_ids
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No free model ID for token {token:?}"))?;
        vocab.insert(token.clone(), id);
    }
    Ok(())
}

fn convert_source(source: &Tokenizer, target: &Tokenizer) -> Result<ConvertedSource> {
    let source_marker = source
        .normalizer
        .as_ref()
        .and_then(find_space_marker)
        .unwrap_or_else(|| "▁".to_owned());
    let uses_byte_level = target
        .pre_tokenizer
        .as_ref()
        .is_some_and(|value| contains_type(value, "ByteLevel"));
    let target_marker = target.normalizer.as_ref().and_then(find_space_marker);
    let target_uses_nfc = target
        .normalizer
        .as_ref()
        .is_some_and(|value| contains_type(value, "NFC"));

    if !uses_byte_level && target_marker.is_none() {
        bail!("Unsupported target: no ByteLevel pre-tokenizer or space marker found");
    }

    let source_specials: HashSet<String> = source
        .added_tokens
        .iter()
        .filter(|item| item.get("special").and_then(Value::as_bool) == Some(true))
        .filter_map(added_token_content)
        .filter(|token| source.model.vocab.contains_key(*token))
        .map(str::to_owned)
        .collect();
    let alphabet = byte_level_alphabet();
    let mut source_order: Vec<(&String, &u32)> = source.model.vocab.iter().collect();
    source_order.sort_by_key(|(token, id)| (**id, token.as_str()));

    let mut token_map = HashMap::new();
    let mut injection_order = Vec::new();
    let mut seen_injected = HashSet::new();
    for (token, _) in source_order {
        if source_specials.contains(token) {
            continue;
        }
        let converted = if let Some(byte) = parse_byte_fallback(token) {
            if uses_byte_level {
                alphabet[byte as usize].to_string()
            } else {
                token.clone()
            }
        } else {
            let text = token.replace(&source_marker, " ");
            let normalized = if target_uses_nfc {
                text.nfc().collect::<String>()
            } else {
                text
            };
            if uses_byte_level {
                normalized
                    .as_bytes()
                    .iter()
                    .map(|byte| alphabet[*byte as usize])
                    .collect()
            } else {
                normalized.replace(' ', target_marker.as_deref().expect("marker checked"))
            }
        };
        if seen_injected.insert(converted.clone()) {
            injection_order.push(converted.clone());
        }
        token_map.insert(token.clone(), converted);
    }

    let mut bridge_tokens = HashSet::new();
    let mut bridge_rules = Vec::new();
    if uses_byte_level {
        let merge_results: HashSet<String> =
            source.model.merges.iter().map(Merge::result).collect();
        let mut seen_bridges = HashSet::new();
        let mut base_tokens: Vec<(&String, &u32)> = source
            .model
            .vocab
            .iter()
            .filter(|(token, _)| token_map.contains_key(*token) && !merge_results.contains(*token))
            .collect();
        base_tokens.sort_by_key(|(token, id)| (**id, token.as_str()));

        for (token, _) in base_tokens {
            let converted = &token_map[token];
            let mut characters = converted.chars();
            let Some(first) = characters.next() else {
                continue;
            };
            let mut prefix = first.to_string();
            for character in characters {
                let right = character.to_string();
                let pair = (prefix.clone(), right.clone());
                prefix.push(character);
                bridge_tokens.insert(prefix.clone());
                if seen_bridges.insert(pair.clone()) {
                    bridge_rules.push(Merge(pair.0, pair.1));
                }
            }
        }
    }

    Ok(ConvertedSource {
        token_map,
        injection_order,
        bridge_tokens,
        bridge_rules,
        representation: if uses_byte_level {
            "ByteLevel (Ġ)".to_owned()
        } else {
            format!(
                "space marker ({})",
                target_marker.expect("marker checked above")
            )
        },
    })
}

impl BPETokenizerEditor {
    /// Merge a source BPE tokenizer into this target tokenizer.
    ///
    /// Source tokens are converted to the target's native ByteLevel or space-marker
    /// representation. Byte bridges and source merges receive higher priority than
    /// original target merges. The complete vocabulary, including added tokens, is
    /// capped at `max_vocab_size`.
    pub fn merge_from(
        &mut self,
        source: &Tokenizer,
        max_vocab_size: usize,
    ) -> Result<MergeTokenizerResult> {
        if source.model.model_type != "BPE" || self.tokenizer.model.model_type != "BPE" {
            bail!("Only BPE tokenizers are supported");
        }
        if max_vocab_size == 0 || max_vocab_size > u32::MAX as usize {
            bail!("max_vocab_size must be between 1 and {}", u32::MAX);
        }

        let initial_target_vocab_size = self.tokenizer.model.vocab.len();
        let source_vocab_size = source.model.vocab.len();
        let target_vocab = self.tokenizer.model.vocab.clone();
        let target_rules = self.tokenizer.model.merges.clone();
        let converted = convert_source(source, &self.tokenizer)?;

        let mut external_added_contents = Vec::new();
        let mut seen_external = HashSet::new();
        for item in &self.tokenizer.added_tokens {
            if let Some(content) = added_token_content(item) {
                if !target_vocab.contains_key(content) && seen_external.insert(content.to_owned()) {
                    external_added_contents.push(content.to_owned());
                }
            }
        }
        if external_added_contents.len() >= max_vocab_size {
            bail!("Added tokens alone exceed max_vocab_size");
        }
        let external_added_set: HashSet<String> = external_added_contents.iter().cloned().collect();
        let model_limit = max_vocab_size - external_added_contents.len();

        let converted_vocab: HashSet<String> = converted
            .token_map
            .values()
            .chain(converted.bridge_tokens.iter())
            .filter(|token| !external_added_set.contains(*token))
            .cloned()
            .collect();

        let mut selected = converted_vocab.clone();
        selected.extend(
            target_vocab
                .keys()
                .filter(|token| token.chars().count() == 1 || is_special(token))
                .cloned(),
        );
        selected.extend(
            self.tokenizer
                .added_tokens
                .iter()
                .filter_map(added_token_content)
                .filter(|token| target_vocab.contains_key(*token))
                .map(str::to_owned),
        );
        if let Some(unknown) = &self.tokenizer.model.unk_token {
            if target_vocab.contains_key(unknown) {
                selected.insert(unknown.clone());
            }
        }

        if selected.len() > model_limit {
            bail!(
                "Required converted and protected tokens ({}) exceed model limit ({})",
                selected.len(),
                model_limit
            );
        }

        let mut target_order: Vec<(&String, &u32)> = target_vocab.iter().collect();
        target_order.sort_by_key(|(token, id)| (**id, token.as_str()));
        for (token, _) in &target_order {
            if selected.len() == model_limit {
                break;
            }
            selected.insert((*token).clone());
        }

        let model_size = selected.len();
        let mut merged_vocab = BTreeMap::new();
        let mut used_ids = HashSet::new();
        for (token, id) in &target_order {
            if selected.contains(*token) && (**id as usize) < model_size {
                merged_vocab.insert((*token).clone(), **id);
                used_ids.insert(**id);
            }
        }
        let mut free_ids: VecDeque<u32> = (0..model_size as u32)
            .filter(|id| !used_ids.contains(id))
            .collect();

        for token in &converted.injection_order {
            assign_token_id(token, &selected, &mut merged_vocab, &mut free_ids)?;
        }
        let mut bridge_order: Vec<&String> = converted.bridge_tokens.iter().collect();
        bridge_order.sort();
        for token in bridge_order {
            assign_token_id(token, &selected, &mut merged_vocab, &mut free_ids)?;
        }
        for (token, _) in &target_order {
            assign_token_id(token, &selected, &mut merged_vocab, &mut free_ids)?;
        }
        let mut remaining: Vec<&String> = selected.iter().collect();
        remaining.sort();
        for token in remaining {
            assign_token_id(token, &selected, &mut merged_vocab, &mut free_ids)?;
        }

        if merged_vocab.len() != model_size || !free_ids.is_empty() {
            bail!("Failed to assign a contiguous model vocabulary");
        }

        let mut external_ids: HashMap<String, u32> = HashMap::new();
        let mut next_external_id = model_size as u32;
        for item in &mut self.tokenizer.added_tokens {
            let Some(content) = added_token_content(item).map(str::to_owned) else {
                continue;
            };
            let id = if let Some(model_id) = merged_vocab.get(&content) {
                *model_id
            } else if let Some(existing) = external_ids.get(&content) {
                *existing
            } else {
                let id = next_external_id;
                next_external_id += 1;
                external_ids.insert(content, id);
                id
            };
            set_added_token_id(item, id)?;
        }

        let final_vocab_size = model_size + external_ids.len();
        if final_vocab_size > max_vocab_size {
            bail!("Complete vocabulary exceeds max_vocab_size");
        }

        let mut merged_rules = Vec::new();
        let mut seen_rules = HashSet::new();
        let mut bridge_merges_added = 0;
        for rule in &converted.bridge_rules {
            bridge_merges_added += push_rule(
                &rule.0,
                &rule.1,
                &selected,
                &mut seen_rules,
                &mut merged_rules,
            ) as usize;
        }

        let mut source_merges_added = 0;
        for rule in &source.model.merges {
            let result = rule.result();
            let (Some(left), Some(right), Some(converted_result)) = (
                converted.token_map.get(&rule.0),
                converted.token_map.get(&rule.1),
                converted.token_map.get(&result),
            ) else {
                continue;
            };
            if format!("{}{}", left, right) != *converted_result {
                continue;
            }
            source_merges_added +=
                push_rule(left, right, &selected, &mut seen_rules, &mut merged_rules) as usize;
        }

        let mut target_merges_retained = 0;
        for rule in &target_rules {
            target_merges_retained += push_rule(
                &rule.0,
                &rule.1,
                &selected,
                &mut seen_rules,
                &mut merged_rules,
            ) as usize;
        }

        let target_tokens: HashSet<String> = target_vocab.keys().cloned().collect();
        let tokens_injected = converted_vocab.difference(&target_tokens).count();
        let target_tokens_removed = target_vocab
            .keys()
            .filter(|token| !selected.contains(*token))
            .count();
        let bridge_tokens_added = converted
            .bridge_tokens
            .iter()
            .filter(|token| converted_vocab.contains(*token) && !target_vocab.contains_key(*token))
            .count();

        self.tokenizer.model.vocab = merged_vocab;
        self.tokenizer.model.merges = merged_rules;
        self.used_ids = self.tokenizer.model.vocab.values().copied().collect();
        self.used_ids.extend(external_ids.values().copied());
        self.next_id = final_vocab_size as u32;
        self.rebuild_indices();

        let (_, invalid) = self.validate_merges();
        if !invalid.is_empty() {
            bail!("Merged tokenizer contains {} invalid merges", invalid.len());
        }
        let expected_ids: HashSet<u32> = (0..final_vocab_size as u32).collect();
        if self.used_ids != expected_ids {
            bail!("Merged tokenizer IDs are not contiguous");
        }

        Ok(MergeTokenizerResult {
            initial_target_vocab_size,
            source_vocab_size,
            final_model_vocab_size: model_size,
            final_vocab_size,
            tokens_injected,
            target_tokens_removed,
            bridge_tokens_added,
            bridge_merges_added,
            source_merges_added,
            target_merges_retained,
            representation: converted.representation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Model;

    fn tokenizer(
        vocab: &[(&str, u32)],
        merges: &[(&str, &str)],
        normalizer: Option<Value>,
        pre_tokenizer: Option<Value>,
        added_tokens: Vec<Value>,
    ) -> Tokenizer {
        Tokenizer {
            version: "1.0".to_owned(),
            truncation: None,
            padding: None,
            added_tokens,
            normalizer,
            pre_tokenizer,
            post_processor: None,
            decoder: None,
            model: Model {
                model_type: "BPE".to_owned(),
                dropout: None,
                unk_token: None,
                continuing_subword_prefix: None,
                end_of_word_suffix: None,
                fuse_unk: false,
                byte_fallback: false,
                ignore_merges: false,
                vocab: vocab
                    .iter()
                    .map(|(token, id)| ((*token).to_owned(), *id))
                    .collect(),
                merges: merges
                    .iter()
                    .map(|(left, right)| Merge((*left).to_owned(), (*right).to_owned()))
                    .collect(),
            },
        }
    }

    #[test]
    fn byte_level_merge_converts_marker_builds_bytes_and_prioritizes_source() {
        let source = tokenizer(
            &[
                ("<unk>", 0),
                ("<0xC4>", 1),
                ("<0xB1>", 2),
                ("▁", 3),
                ("n", 4),
                ("a", 5),
                ("s", 6),
                ("ı", 7),
                ("l", 8),
                ("na", 9),
                ("nas", 10),
                ("ıl", 11),
                ("nasıl", 12),
                ("▁nasıl", 13),
            ],
            &[
                ("n", "a"),
                ("na", "s"),
                ("ı", "l"),
                ("nas", "ıl"),
                ("▁", "nasıl"),
            ],
            Some(serde_json::json!({
                "type": "Replace",
                "pattern": {"String": " "},
                "content": "▁"
            })),
            None,
            vec![serde_json::json!({
                "id": 0,
                "content": "<unk>",
                "special": true
            })],
        );
        let target = tokenizer(
            &[
                ("Ġ", 0),
                ("n", 1),
                ("a", 2),
                ("s", 3),
                ("l", 4),
                ("Ä", 5),
                ("±", 6),
                ("na", 7),
                ("nas", 8),
                ("target", 9),
            ],
            &[("n", "a"), ("na", "s")],
            Some(serde_json::json!({"type": "NFC"})),
            Some(serde_json::json!({"type": "ByteLevel"})),
            vec![serde_json::json!({
                "id": 10,
                "content": "<target-special>",
                "special": true
            })],
        );

        let mut editor = BPETokenizerEditor::new(target);
        let result = editor.merge_from(&source, 30).unwrap();

        assert_eq!(result.representation, "ByteLevel (Ġ)");
        assert!(editor.has_token("ĠnasÄ±l"));
        assert!(!editor.has_token("▁nasıl"));
        assert!(!editor.has_token("<unk>"));
        assert_eq!(
            editor.tokenizer.model.merges[0],
            Merge("Ä".into(), "±".into())
        );
        let source_rank = editor
            .tokenizer
            .model
            .merges
            .iter()
            .position(|merge| merge.result() == "ĠnasÄ±l")
            .unwrap();
        assert!(source_rank < editor.tokenizer.model.merges.len());
        assert!(result.final_vocab_size <= 30);
        assert_eq!(editor.validate_merges().1.len(), 0);
    }

    #[test]
    fn marker_target_keeps_its_native_marker() {
        let source = tokenizer(
            &[("▁", 0), ("a", 1), ("▁a", 2)],
            &[("▁", "a")],
            Some(serde_json::json!({
                "type": "Replace",
                "pattern": {"String": " "},
                "content": "▁"
            })),
            None,
            vec![],
        );
        let target = source.clone();
        let mut editor = BPETokenizerEditor::new(target);

        let result = editor.merge_from(&source, 8).unwrap();

        assert_eq!(result.representation, "space marker (▁)");
        assert!(editor.has_token("▁a"));
        assert_eq!(editor.tokenizer.model.merges[0].result(), "▁a");
    }
}
