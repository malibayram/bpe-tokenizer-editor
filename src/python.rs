//! Python bindings for BPE Tokenizer Editor using PyO3

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::editor::BPETokenizerEditor;
use crate::tokenizer::Merge;

/// Python wrapper for BPETokenizerEditor
#[pyclass(name = "BPETokenizerEditor")]
pub struct PyBPETokenizerEditor {
    inner: BPETokenizerEditor,
}

/// Result of validating merges
#[pyclass(name = "ValidationResult")]
#[derive(Clone)]
pub struct PyValidationResult {
    #[pyo3(get)]
    pub valid_count: usize,
    #[pyo3(get)]
    pub invalid_count: usize,
    #[pyo3(get)]
    pub invalid_merges: Vec<(usize, String, String)>,
}

/// Result of token addition
#[pyclass(name = "AdditionResult")]
#[derive(Clone)]
pub struct PyAdditionResult {
    #[pyo3(get)]
    pub token: String,
    #[pyo3(get)]
    pub added: bool,
    #[pyo3(get)]
    pub method: String,
    #[pyo3(get)]
    pub added_merges: Vec<(String, String)>,
}

/// Result of token removal
#[pyclass(name = "RemovalResult")]
#[derive(Clone)]
pub struct PyRemovalResult {
    #[pyo3(get)]
    pub root_token: String,
    #[pyo3(get)]
    pub removed_tokens: Vec<String>,
    #[pyo3(get)]
    pub removed_merges: Vec<(String, String)>,
}

/// Result of shrink operation
#[pyclass(name = "ShrinkResult")]
#[derive(Clone)]
pub struct PyShrinkResult {
    #[pyo3(get)]
    pub initial_vocab_size: usize,
    #[pyo3(get)]
    pub initial_merges_count: usize,
    #[pyo3(get)]
    pub final_vocab_size: usize,
    #[pyo3(get)]
    pub final_merges_count: usize,
    #[pyo3(get)]
    pub tokens_removed_count: usize,
    #[pyo3(get)]
    pub total_tokens_removed: usize,
    #[pyo3(get)]
    pub total_merges_removed: usize,
}

/// Tokenizer statistics
#[pyclass(name = "TokenizerStats")]
#[derive(Clone)]
pub struct PyTokenizerStats {
    #[pyo3(get)]
    pub vocab_size: usize,
    #[pyo3(get)]
    pub merges_count: usize,
    #[pyo3(get)]
    pub single_char_count: usize,
    #[pyo3(get)]
    pub special_token_count: usize,
    #[pyo3(get)]
    pub min_token_id: u32,
    #[pyo3(get)]
    pub max_token_id: u32,
    #[pyo3(get)]
    pub length_distribution: Vec<(usize, usize)>,
}

/// Result of reindex operation
#[pyclass(name = "ReindexResult")]
#[derive(Clone)]
pub struct PyReindexResult {
    #[pyo3(get)]
    pub vocab_size: usize,
    #[pyo3(get)]
    pub merges_count: usize,
    #[pyo3(get)]
    pub old_min_id: u32,
    #[pyo3(get)]
    pub old_max_id: u32,
    #[pyo3(get)]
    pub new_min_id: u32,
    #[pyo3(get)]
    pub new_max_id: u32,
    #[pyo3(get)]
    pub ids_remapped: usize,
    #[pyo3(get)]
    pub gaps_removed: usize,
}

#[pymethods]
impl PyBPETokenizerEditor {
    /// Load a tokenizer from a JSON file
    ///
    /// Args:
    ///     path: Path to the tokenizer.json file
    ///
    /// Returns:
    ///     BPETokenizerEditor instance
    ///
    /// Raises:
    ///     IOError: If the file cannot be read
    ///     ValueError: If the tokenizer is not BPE type
    #[new]
    #[pyo3(signature = (path))]
    fn new(path: &str) -> PyResult<Self> {
        let path_buf = PathBuf::from(path);
        let inner = BPETokenizerEditor::load(&path_buf)
            .map_err(|e| PyIOError::new_err(format!("Failed to load tokenizer: {}", e)))?;
        Ok(Self { inner })
    }

    /// Create a new editor from JSON string
    ///
    /// Args:
    ///     json_str: JSON string containing the tokenizer
    ///
    /// Returns:
    ///     BPETokenizerEditor instance
    #[staticmethod]
    #[pyo3(signature = (json_str))]
    fn from_json(json_str: &str) -> PyResult<Self> {
        let tokenizer: crate::tokenizer::Tokenizer = serde_json::from_str(json_str)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse JSON: {}", e)))?;

        if tokenizer.model.model_type != "BPE" {
            return Err(PyValueError::new_err("Only BPE tokenizers are supported"));
        }

        Ok(Self {
            inner: BPETokenizerEditor::new(tokenizer),
        })
    }

    /// Save the tokenizer to a JSON file
    ///
    /// Args:
    ///     path: Output path for the tokenizer.json file
    #[pyo3(signature = (path))]
    fn save(&self, path: &str) -> PyResult<()> {
        let path_buf = PathBuf::from(path);
        self.inner
            .save(&path_buf)
            .map_err(|e| PyIOError::new_err(format!("Failed to save tokenizer: {}", e)))
    }

    /// Export tokenizer to JSON string
    ///
    /// Returns:
    ///     JSON string representation of the tokenizer
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.inner.tokenizer)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))
    }

    /// Get the current vocabulary size
    #[getter]
    fn vocab_size(&self) -> usize {
        self.inner.vocab_size()
    }

    /// Get the current number of merges
    #[getter]
    fn merges_count(&self) -> usize {
        self.inner.merges_count()
    }

    /// Check if a token exists in the vocabulary
    ///
    /// Args:
    ///     token: Token string to check
    ///
    /// Returns:
    ///     True if token exists, False otherwise
    #[pyo3(signature = (token))]
    fn has_token(&self, token: &str) -> bool {
        self.inner.has_token(token)
    }

    /// Get the ID of a token
    ///
    /// Args:
    ///     token: Token string
    ///
    /// Returns:
    ///     Token ID or None if not found
    #[pyo3(signature = (token))]
    fn get_token_id(&self, token: &str) -> Option<u32> {
        self.inner.tokenizer.model.vocab.get(token).copied()
    }

    /// Get a token by its ID
    ///
    /// Args:
    ///     id: Token ID
    ///
    /// Returns:
    ///     Token string or None if not found
    #[pyo3(signature = (id))]
    fn get_token_by_id(&self, id: u32) -> Option<String> {
        self.inner
            .tokenizer
            .model
            .vocab
            .iter()
            .find(|(_, &v)| v == id)
            .map(|(k, _)| k.clone())
    }

    /// Get all tokens in the vocabulary
    ///
    /// Returns:
    ///     Dictionary mapping token strings to their IDs
    fn get_vocab(&self) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (token, id) in &self.inner.tokenizer.model.vocab {
                dict.set_item(token, *id)?;
            }
            Ok(dict.into())
        })
    }

    /// Get all merges as list of tuples
    ///
    /// Returns:
    ///     List of (token_a, token_b) merge pairs
    fn get_merges(&self) -> Vec<(String, String)> {
        self.inner
            .tokenizer
            .model
            .merges
            .iter()
            .map(|m| (m.0.clone(), m.1.clone()))
            .collect()
    }

    /// Get comprehensive tokenizer statistics
    ///
    /// Returns:
    ///     TokenizerStats object with detailed statistics
    fn get_stats(&self) -> PyTokenizerStats {
        let vocab = &self.inner.tokenizer.model.vocab;

        let mut length_counts: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        let mut single_char_count = 0;
        let mut special_token_count = 0;
        let mut min_id = u32::MAX;
        let mut max_id = 0u32;

        for (token, &id) in vocab {
            let char_len = token.chars().count();
            *length_counts.entry(char_len).or_insert(0) += 1;

            if char_len == 1 {
                single_char_count += 1;
            }

            if (token.starts_with('<') && token.ends_with('>'))
                || (token.starts_with('[') && token.ends_with(']'))
            {
                special_token_count += 1;
            }

            min_id = min_id.min(id);
            max_id = max_id.max(id);
        }

        let mut length_distribution: Vec<(usize, usize)> = length_counts.into_iter().collect();
        length_distribution.sort_by_key(|(len, _)| *len);

        PyTokenizerStats {
            vocab_size: vocab.len(),
            merges_count: self.inner.merges_count(),
            single_char_count,
            special_token_count,
            min_token_id: if vocab.is_empty() { 0 } else { min_id },
            max_token_id: max_id,
            length_distribution,
        }
    }

    /// Validate all merges - check that each merge result exists in vocabulary
    ///
    /// Returns:
    ///     ValidationResult with valid/invalid merge counts and details
    fn validate_merges(&self) -> PyValidationResult {
        let (valid_indices, invalid) = self.inner.validate_merges();

        let invalid_merges: Vec<(usize, String, String)> = invalid
            .iter()
            .map(|(idx, merge)| (*idx, merge.0.clone(), merge.1.clone()))
            .collect();

        PyValidationResult {
            valid_count: valid_indices.len(),
            invalid_count: invalid.len(),
            invalid_merges,
        }
    }

    /// Remove all invalid merges from the tokenizer
    ///
    /// Returns:
    ///     Number of invalid merges removed
    fn remove_invalid_merges(&mut self) -> usize {
        self.inner.remove_invalid_merges()
    }

    /// Add a token with proper merge chain creation
    ///
    /// This method intelligently adds a token using one of these strategies:
    /// - Single character: Added directly to vocab
    /// - Longest prefix: If a prefix exists, adds prefix + suffix -> token merge
    /// - Character chain: Builds merge chain a+b->ab, ab+c->abc, etc.
    ///
    /// Args:
    ///     token: Token string to add
    ///
    /// Returns:
    ///     AdditionResult with details about how the token was added
    #[pyo3(signature = (token))]
    fn add_token(&mut self, token: &str) -> PyAdditionResult {
        let result = self.inner.add_token_with_merges(token);
        PyAdditionResult {
            token: result.token,
            added: result.added,
            method: result.method,
            added_merges: result.added_merges,
        }
    }

    /// Add multiple tokens
    ///
    /// Args:
    ///     tokens: List of token strings to add
    ///
    /// Returns:
    ///     List of AdditionResult for each token
    #[pyo3(signature = (tokens))]
    fn add_tokens(&mut self, tokens: Vec<String>) -> Vec<PyAdditionResult> {
        tokens
            .iter()
            .map(|t| {
                let result = self.inner.add_token_with_merges(t);
                PyAdditionResult {
                    token: result.token,
                    added: result.added,
                    method: result.method,
                    added_merges: result.added_merges,
                }
            })
            .collect()
    }

    /// Add a token without creating merge chains (for special tokens)
    ///
    /// Args:
    ///     token: Token string to add
    ///
    /// Returns:
    ///     True if token was added, False if it already existed
    #[pyo3(signature = (token))]
    fn add_token_atomic(&mut self, token: &str) -> bool {
        self.inner.add_token_atomic(token)
    }

    /// Remove a token and all its dependencies
    ///
    /// This performs cascade removal:
    /// - Removes all merges that use this token as input
    /// - Removes all merges that produce this token
    /// - Removes all tokens that can only be produced by removed merges
    ///
    /// Args:
    ///     token: Token string to remove
    ///
    /// Returns:
    ///     RemovalResult with details about what was removed
    #[pyo3(signature = (token))]
    fn remove_token(&mut self, token: &str) -> PyRemovalResult {
        let result = self.inner.remove_token_and_dependencies(token);
        PyRemovalResult {
            root_token: result.root_token,
            removed_tokens: result.removed_tokens,
            removed_merges: result.removed_merges,
        }
    }

    /// Remove multiple tokens
    ///
    /// Args:
    ///     tokens: List of token strings to remove
    ///
    /// Returns:
    ///     List of RemovalResult for each token
    #[pyo3(signature = (tokens))]
    fn remove_tokens(&mut self, tokens: Vec<String>) -> Vec<PyRemovalResult> {
        tokens
            .iter()
            .filter_map(|t| {
                if self.inner.has_token(t) {
                    let result = self.inner.remove_token_and_dependencies(t);
                    Some(PyRemovalResult {
                        root_token: result.root_token,
                        removed_tokens: result.removed_tokens,
                        removed_merges: result.removed_merges,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Shrink vocabulary by removing N longest non-special tokens
    ///
    /// Selection criteria (in order):
    /// - Longest tokens first (by character count)
    /// - Highest token IDs first (among same length)
    /// - Never removes single-character tokens
    /// - Never removes special tokens (<...> or [...])
    ///
    /// Args:
    ///     count: Number of tokens to remove
    ///     min_id: Only consider tokens with ID >= min_id (default: 0)
    ///
    /// Returns:
    ///     ShrinkResult with details about the shrink operation
    #[pyo3(signature = (count, min_id = 0))]
    fn shrink(&mut self, count: usize, min_id: u32) -> PyShrinkResult {
        let result = self.inner.shrink_vocab(count, min_id);
        PyShrinkResult {
            initial_vocab_size: result.initial_vocab_size,
            initial_merges_count: result.initial_merges_count,
            final_vocab_size: result.final_vocab_size,
            final_merges_count: result.final_merges_count,
            tokens_removed_count: result.tokens_removed.len(),
            total_tokens_removed: result.total_tokens_removed,
            total_merges_removed: result.total_merges_removed,
        }
    }

    /// Find tokens that would be removed by shrink operation (preview)
    ///
    /// Args:
    ///     count: Number of tokens to find
    ///     min_id: Only consider tokens with ID >= min_id (default: 0)
    ///
    /// Returns:
    ///     List of (token, id, char_length) tuples
    #[pyo3(signature = (count, min_id = 0))]
    fn find_tokens_to_shrink(&self, count: usize, min_id: u32) -> Vec<(String, u32, usize)> {
        self.inner.find_tokens_to_shrink(count, min_id)
    }

    /// Get all single-character tokens
    ///
    /// Returns:
    ///     List of (token, id) tuples
    fn get_single_char_tokens(&self) -> Vec<(String, u32)> {
        self.inner.get_single_char_tokens()
    }

    /// Sync single-character tokens from another tokenizer
    ///
    /// Copies missing single-char tokens from source, removing longest
    /// tokens to maintain vocabulary size.
    ///
    /// Args:
    ///     source_path: Path to source tokenizer.json
    ///     min_id: Only remove tokens with ID >= min_id (default: 0)
    ///
    /// Returns:
    ///     Dictionary with sync operation details
    #[pyo3(signature = (source_path, min_id = 0))]
    fn sync_single_chars(&mut self, source_path: &str, min_id: u32) -> PyResult<Py<PyDict>> {
        let source_path_buf = PathBuf::from(source_path);
        let source = BPETokenizerEditor::load(&source_path_buf)
            .map_err(|e| PyIOError::new_err(format!("Failed to load source tokenizer: {}", e)))?;

        let source_chars = source.get_single_char_tokens();
        let result = self.inner.sync_single_chars(&source_chars, min_id);

        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("initial_vocab_size", result.initial_vocab_size)?;
            dict.set_item("final_vocab_size", result.final_vocab_size)?;
            dict.set_item("chars_in_source", result.chars_in_source)?;
            dict.set_item("chars_already_present", result.chars_already_present)?;
            dict.set_item("chars_added_count", result.chars_added.len())?;
            dict.set_item("total_tokens_removed", result.total_tokens_removed)?;
            dict.set_item("total_merges_removed", result.total_merges_removed)?;
            Ok(dict.into())
        })
    }

    /// Add tokens while keeping vocabulary size fixed
    ///
    /// Adds new tokens and automatically removes other tokens to maintain
    /// the original vocabulary size.
    ///
    /// Args:
    ///     tokens: List of tokens to add
    ///     whitelist: Optional list of tokens that should never be removed
    ///
    /// Returns:
    ///     Dictionary with operation details
    #[pyo3(signature = (tokens, whitelist = None))]
    fn add_tokens_keep_size(
        &mut self,
        tokens: Vec<String>,
        whitelist: Option<Vec<String>>,
    ) -> PyResult<Py<PyDict>> {
        let initial_size = self.inner.vocab_size();
        let extra_protected: HashSet<String> = whitelist.unwrap_or_default().into_iter().collect();
        let protected = self.inner.build_protected_set(&extra_protected);

        let mut added_count = 0;
        let mut removed_count = 0;

        for token in &tokens {
            if self.inner.has_token(token) {
                continue;
            }

            // Add the token
            self.inner.add_token_with_merges(token);
            added_count += 1;

            // Try to remove a token to keep size
            if self.inner.vocab_size() > initial_size {
                if let Some((to_remove, _)) = self.inner.select_token_to_remove(&protected) {
                    self.inner.remove_token_and_dependencies(&to_remove);
                    removed_count += 1;
                }
            }
        }

        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("initial_vocab_size", initial_size)?;
            dict.set_item("final_vocab_size", self.inner.vocab_size())?;
            dict.set_item("tokens_requested", tokens.len())?;
            dict.set_item("tokens_added", added_count)?;
            dict.set_item("tokens_removed", removed_count)?;
            Ok(dict.into())
        })
    }

    /// Check if vocabulary has gaps in its ID space
    ///
    /// Returns:
    ///     Tuple of (has_gaps, total_gaps, min_id, max_id)
    fn check_vocab_gaps(&self) -> (bool, usize, u32, u32) {
        self.inner.check_vocab_gaps()
    }

    /// Reindex vocabulary to make all IDs sequential
    ///
    /// This removes all gaps in the vocabulary IDs, creating a dense/compact
    /// ID space. Tokens that already have correct sequential IDs (starting from 0)
    /// are preserved - only tokens after the first gap are remapped.
    ///
    /// Note: Merges don't contain IDs, they reference token strings,
    /// so they don't need to be updated.
    ///
    /// Returns:
    ///     ReindexResult with details about the reindexing operation
    fn reindex_vocab(&mut self) -> PyReindexResult {
        let result = self.inner.reindex_vocab();
        PyReindexResult {
            vocab_size: result.vocab_size,
            merges_count: result.merges_count,
            old_min_id: result.old_min_id,
            old_max_id: result.old_max_id,
            new_min_id: result.new_min_id,
            new_max_id: result.new_max_id,
            ids_remapped: result.ids_remapped,
            gaps_removed: result.gaps_removed,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "BPETokenizerEditor(vocab_size={}, merges_count={})",
            self.inner.vocab_size(),
            self.inner.merges_count()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Python module for BPE Tokenizer Editor
#[pymodule]
#[pyo3(name = "bpe_tokenizer_editor")]
pub fn bpe_tokenizer_editor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBPETokenizerEditor>()?;
    m.add_class::<PyValidationResult>()?;
    m.add_class::<PyAdditionResult>()?;
    m.add_class::<PyRemovalResult>()?;
    m.add_class::<PyShrinkResult>()?;
    m.add_class::<PyTokenizerStats>()?;
    m.add_class::<PyReindexResult>()?;

    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
