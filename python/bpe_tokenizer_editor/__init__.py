"""
BPE Tokenizer Editor - Python bindings for editing HuggingFace BPE tokenizer.json files.

This module provides a high-performance Rust-powered editor for BPE tokenizer files
with consistency guarantees.

Example:
    >>> from bpe_tokenizer_editor import BPETokenizerEditor
    >>> editor = BPETokenizerEditor("tokenizer.json")
    >>> print(f"Vocab size: {editor.vocab_size}")
    >>> editor.add_token("merhaba")
    >>> editor.save("tokenizer_modified.json")
"""

from .bpe_tokenizer_editor import (AdditionResult, BPETokenizerEditor,
                                   RemovalResult, ShrinkResult, TokenizerStats,
                                   ValidationResult, __version__)

__all__ = [
    "BPETokenizerEditor",
    "ValidationResult",
    "AdditionResult",
    "RemovalResult",
    "ShrinkResult",
    "TokenizerStats",
    "__version__",
]
