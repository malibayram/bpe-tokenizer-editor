# BPE Tokenizer Editor

[![PyPI version](https://badge.fury.io/py/bpe-tokenizer-editor.svg)](https://pypi.org/project/bpe-tokenizer-editor/)
[![Python versions](https://img.shields.io/pypi/pyversions/bpe-tokenizer-editor.svg)](https://pypi.org/project/bpe-tokenizer-editor/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Python library for editing HuggingFace BPE tokenizer.json files with **consistency guarantees**. Built with Rust for maximum performance.

## Features

- ✅ **Validate merges** - Check that all merge results exist in vocabulary
- ✅ **Add tokens** - Add new tokens with automatic merge chain creation
- ✅ **Remove tokens** - Remove tokens with cascade deletion of dependent merges
- ✅ **Shrink vocab** - Remove N longest tokens to reduce vocabulary size
- ✅ **Sync single-chars** - Copy all single-character tokens from source tokenizer
- ✅ **Keep vocab size fixed** - Add tokens while automatically removing others to maintain size
- 🚀 **Rust-powered** - Native performance with Python convenience

## Installation

```bash
pip install bpe-tokenizer-editor
```

### Requirements

- Python 3.8+
- No additional dependencies required (Rust code is compiled into the wheel)

## Quick Start

```python
from bpe_tokenizer_editor import BPETokenizerEditor

# Load a tokenizer
editor = BPETokenizerEditor("tokenizer.json")

# Check stats
print(f"Vocabulary size: {editor.vocab_size}")
print(f"Number of merges: {editor.merges_count}")

# Validate merges
validation = editor.validate_merges()
print(f"Valid merges: {validation.valid_count}")
print(f"Invalid merges: {validation.invalid_count}")

# Add a new token
result = editor.add_token("merhaba")
print(f"Added: {result.added}, Method: {result.method}")

# Save the modified tokenizer
editor.save("tokenizer_modified.json")
```

## API Reference

### Loading and Saving

```python
# Load from file
editor = BPETokenizerEditor("tokenizer.json")

# Load from JSON string
editor = BPETokenizerEditor.from_json(json_string)

# Save to file
editor.save("output.json")

# Export to JSON string
json_str = editor.to_json()
```

### Properties

```python
editor.vocab_size    # Current vocabulary size
editor.merges_count  # Current number of merge rules
```

### Token Operations

```python
# Check if token exists
editor.has_token("hello")  # Returns bool

# Get token ID
editor.get_token_id("hello")  # Returns int or None

# Get token by ID
editor.get_token_by_id(1000)  # Returns str or None

# Get all tokens as dict
vocab = editor.get_vocab()  # Returns Dict[str, int]

# Get all merges
merges = editor.get_merges()  # Returns List[Tuple[str, str]]

# Get single-character tokens
single_chars = editor.get_single_char_tokens()  # Returns List[Tuple[str, int]]
```

### Statistics

```python
stats = editor.get_stats()
print(f"Vocab size: {stats.vocab_size}")
print(f"Merges: {stats.merges_count}")
print(f"Single chars: {stats.single_char_count}")
print(f"Special tokens: {stats.special_token_count}")
print(f"ID range: {stats.min_token_id} - {stats.max_token_id}")
print(f"Length distribution: {stats.length_distribution}")
```

### Validation

```python
# Validate merges (check if merge results exist in vocab)
result = editor.validate_merges()
print(f"Valid: {result.valid_count}, Invalid: {result.invalid_count}")

for idx, token_a, token_b in result.invalid_merges:
    print(f"  Invalid merge at {idx}: '{token_a}' + '{token_b}'")

# Fix invalid merges (remove them)
removed_count = editor.remove_invalid_merges()
print(f"Removed {removed_count} invalid merges")
```

### Adding Tokens

```python
# Add a single token with automatic merge chain creation
result = editor.add_token("newtoken")
print(f"Added: {result.added}")
print(f"Method: {result.method}")  # 'single_char', 'longest_prefix', or 'char_chain'
print(f"Added merges: {result.added_merges}")

# Add multiple tokens
results = editor.add_tokens(["token1", "token2", "token3"])

# Add special token without merge chain
editor.add_token_atomic("<special>")

# Add tokens while keeping vocabulary size fixed
result = editor.add_tokens_keep_size(
    tokens=["new1", "new2", "new3"],
    whitelist=["protected_token"]  # Optional: tokens that should never be removed
)
print(f"Added: {result['tokens_added']}, Removed: {result['tokens_removed']}")
```

### Removing Tokens

```python
# Remove a token and all its dependencies (cascade removal)
result = editor.remove_token("unwanted")
print(f"Root token: {result.root_token}")
print(f"Removed tokens: {result.removed_tokens}")
print(f"Removed merges: {result.removed_merges}")

# Remove multiple tokens
results = editor.remove_tokens(["token1", "token2"])
```

### Shrinking Vocabulary

```python
# Preview what would be removed
tokens_to_remove = editor.find_tokens_to_shrink(count=1000, min_id=50000)
for token, id, length in tokens_to_remove[:10]:
    print(f"  {token} (ID: {id}, length: {length})")

# Actually shrink the vocabulary
result = editor.shrink(count=1000, min_id=50000)
print(f"Initial size: {result.initial_vocab_size}")
print(f"Final size: {result.final_vocab_size}")
print(f"Tokens removed: {result.total_tokens_removed}")
print(f"Merges removed: {result.total_merges_removed}")
```

### Syncing with Another Tokenizer

```python
# Sync single-character tokens from source tokenizer
# (useful for preserving Unicode coverage)
result = editor.sync_single_chars(
    source_path="original_tokenizer.json",
    min_id=50000  # Only remove tokens with ID >= 50000
)
print(f"Chars added: {result['chars_added_count']}")
print(f"Tokens removed: {result['total_tokens_removed']}")
```

## Examples

### Training a Domain-Specific Tokenizer

```python
from bpe_tokenizer_editor import BPETokenizerEditor

# After training your custom tokenizer, validate and fix it
editor = BPETokenizerEditor("my_custom_tokenizer.json")

# Check for problems
validation = editor.validate_merges()
if validation.invalid_count > 0:
    print(f"Found {validation.invalid_count} invalid merges, fixing...")
    editor.remove_invalid_merges()

# Sync Unicode characters from original tokenizer
result = editor.sync_single_chars(
    source_path="original_tokenizer.json",
    min_id=50000
)
print(f"Added {result['chars_added_count']} missing characters")

# Add domain-specific tokens
domain_tokens = ["merhaba", "dünya", "Türkiye"]
for token in domain_tokens:
    result = editor.add_token(token)
    if result.added:
        print(f"Added '{token}' using {result.method} method")

# Save the result
editor.save("my_final_tokenizer.json")
```

### Reducing Vocabulary Size

```python
from bpe_tokenizer_editor import BPETokenizerEditor

editor = BPETokenizerEditor("large_tokenizer.json")
print(f"Original size: {editor.vocab_size}")

# Preview what will be removed
preview = editor.find_tokens_to_shrink(count=10000, min_id=50000)
print(f"Will remove {len(preview)} tokens")
print("First 5 tokens to be removed:")
for token, id, length in preview[:5]:
    print(f"  '{token}' (length={length}, id={id})")

# Perform the shrink
result = editor.shrink(count=10000, min_id=50000)
print(f"New size: {editor.vocab_size}")
print(f"Total tokens removed (including cascade): {result.total_tokens_removed}")

editor.save("smaller_tokenizer.json")
```

### Adding Tokens with Fixed Vocabulary Size

```python
from bpe_tokenizer_editor import BPETokenizerEditor

editor = BPETokenizerEditor("tokenizer.json")
original_size = editor.vocab_size

# Add new tokens while keeping the same vocabulary size
result = editor.add_tokens_keep_size(
    tokens=["custom1", "custom2", "custom3"],
    whitelist=["<pad>", "<eos>", "<bos>"]  # Never remove these
)

print(f"Tokens added: {result['tokens_added']}")
print(f"Tokens removed to maintain size: {result['tokens_removed']}")
print(f"Final vocab size: {editor.vocab_size}")  # Should equal original_size

editor.save("tokenizer_with_custom_tokens.json")
```

## Performance

The library is written in Rust and compiled to native code, providing excellent performance:

| Operation | Time (256K vocab) |
|-----------|-------------------|
| Load tokenizer | ~50ms |
| Validate merges | ~100ms |
| Get stats | ~50ms |
| Shrink 1000 tokens | 2-5s |
| Sync 18K chars | 30-60s |

## Building from Source

If you want to build the package from source:

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop --features python

# Build wheel for distribution
maturin build --release --features python
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details.
