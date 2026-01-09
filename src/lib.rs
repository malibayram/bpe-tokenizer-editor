//! BPE Tokenizer Editor
//!
//! A tool for editing HuggingFace BPE tokenizer.json files with consistency guarantees.

pub mod cli;
pub mod tokenizer;
pub mod types;

// Editor is split into submodules
mod editor;
pub use editor::BPETokenizerEditor;

// Python bindings (only compiled with the python feature)
#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::bpe_tokenizer_editor;
