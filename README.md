# BPE Tokenizer Editor

A high-performance Rust tool for editing HuggingFace BPE tokenizer.json files with **consistency guarantees**. This tool ensures that all merge operations in a tokenizer remain valid after any modification.

## Features

- ✅ **Validate merges** - Check that all merge results exist in vocabulary
- ✅ **Add tokens** - Add new tokens with automatic merge chain creation
- ✅ **Remove tokens** - Remove tokens with cascade deletion of dependent merges
- ✅ **Shrink vocab** - Remove N longest tokens to reduce vocabulary size
- ✅ **Sync single-chars** - Copy all single-character tokens from source tokenizer
- ✅ **Sync short tokens** - Copy 2-3 letter tokens with their merges from source
- ✅ **Keep vocab size fixed** - Add tokens while automatically removing others to maintain size

## Installation

### Build from source

```bash
cd bpe-tokenizer-editor
cargo build --release
```

The binary will be available at `target/release/bpe-editor`.

### Dependencies

- Rust 2021 edition
- `anyhow` - Error handling
- `clap` - Command-line argument parsing
- `serde` / `serde_json` - JSON serialization

## Usage

### Validate Merges

Check if all merges in a tokenizer are valid (i.e., the merged result exists in the vocabulary):

```bash
# Just validate and report
bpe-editor validate --input tokenizer.json --dry-run

# Validate and fix (remove invalid merges)
bpe-editor validate --input tokenizer.json --output tokenizer_fixed.json
```

**What it does:**

- For each merge rule `A + B -> AB`, checks that `AB` exists in the vocabulary
- Reports all invalid merges with their indices
- Optionally removes invalid merges to restore consistency

### Show Statistics

Display comprehensive tokenizer statistics:

```bash
bpe-editor stats --input tokenizer.json
```

**Output includes:**

- Vocabulary size and merge count
- Token length distribution (how many 1-char, 2-char, ... N-char tokens)
- Count of special tokens (`<...>` and `[...]`)
- ID range (min and max token IDs)
- Merge validation status

### Add Tokens

Add new tokens to the tokenizer with automatic merge chain creation:

```bash
# Add tokens from a JSON file (array of strings)
bpe-editor add --input tokenizer.json --tokens new_tokens.json --output tokenizer_new.json

# Add tokens while keeping vocab size fixed
bpe-editor add --input tokenizer.json --tokens new_tokens.json --output tokenizer_new.json --keep-size

# Protect specific tokens from being removed
bpe-editor add --input tokenizer.json --tokens new_tokens.json --output tokenizer_new.json --keep-size --whitelist protected.json
```

**Token addition strategies:**

1. **Single character** - Added directly to vocab (no merge needed)
2. **Longest prefix** - If a prefix exists in vocab, adds `prefix + suffix -> token` merge
3. **Character chain** - Builds merge chain: `a + b -> ab`, `ab + c -> abc`, etc.

**Example `new_tokens.json`:**

```json
["merhaba", "dünya", "Türkiye", "şeker"]
```

### Remove Tokens

Remove tokens and all their dependencies:

```bash
bpe-editor remove --input tokenizer.json --tokens tokens_to_remove.json --output tokenizer_clean.json
```

**Cascade removal:**
When you remove a token, the tool also removes:

- All merges that use this token as input (`token + X -> Y`)
- All merges that produce this token (`A + B -> token`)
- All tokens that can only be produced by removed merges

### Shrink Vocabulary

Remove N longest non-special tokens with highest IDs:

```bash
# Preview what would be removed
bpe-editor shrink --input tokenizer.json --output tokenizer_shrunk.json --count 1000 --dry-run

# Actually shrink
bpe-editor shrink --input tokenizer.json --output tokenizer_shrunk.json --count 1000

# Only consider tokens with ID >= 50000
bpe-editor shrink --input tokenizer.json --output tokenizer_shrunk.json --count 1000 --min-id 50000
```

**Selection criteria (in order):**

1. Longest tokens first (by character count)
2. Highest token IDs first (among same length)
3. Never removes single-character tokens
4. Never removes special tokens (`<...>` or `[...]`)

### Sync Single-Character Tokens

Copy all single-character tokens from a source tokenizer to a target tokenizer:

```bash
# Preview sync operation
bpe-editor sync-chars \
  --source original_tokenizer.json \
  --target custom_tokenizer.json \
  --output synced_tokenizer.json \
  --dry-run

# Perform sync (removes longest tokens to make room)
bpe-editor sync-chars \
  --source original_tokenizer.json \
  --target custom_tokenizer.json \
  --output synced_tokenizer.json \
  --min-id 50000
```

**Use case:**
When you train a custom tokenizer on domain-specific text, you might lose some Unicode characters that were in the original tokenizer. This command copies them back while maintaining the vocab size.

**How it works:**

1. Finds all single-char tokens in source that are missing in target
2. Pre-computes N tokens to remove (longest tokens with ID >= min-id)
3. Removes those tokens (with cascade)
4. Adds the missing single-char tokens

### Sync Short Tokens (2-3 characters)

Copy short tokens (2-3 chars) along with their merges:

```bash
# Preview sync
bpe-editor sync-short-tokens \
  --source original_tokenizer.json \
  --target custom_tokenizer.json \
  --output synced_tokenizer.json \
  --min-len 2 \
  --max-len 3 \
  --dry-run

# Perform sync
bpe-editor sync-short-tokens \
  --source original_tokenizer.json \
  --target custom_tokenizer.json \
  --output synced_tokenizer.json \
  --min-len 2 \
  --max-len 3 \
  --min-id 50000
```

**Why include merges?**
Short tokens like `ab` need merge rules (`a + b -> ab`) to be created during tokenization. This command copies both the tokens AND their producing merges from the source.

## Architecture

The project is organized into focused modules:

```
src/
├── cli.rs              # CLI argument definitions (clap)
├── tokenizer.rs        # HuggingFace tokenizer.json structures
├── types.rs            # Result types for all operations
├── lib.rs              # Module exports
├── main.rs             # Command handlers
└── editor/
    ├── mod.rs          # Module exports
    ├── core.rs         # Core struct, load/save, indices
    ├── validation.rs   # Merge validation
    ├── removal.rs      # Token removal with cascade
    ├── addition.rs     # Token addition with merge chains
    ├── management.rs   # Vocab size management, shrinking
    └── sync.rs         # Cross-tokenizer sync operations
```

### Key Data Structures

```rust
/// The main editor with consistency guarantees
pub struct BPETokenizerEditor {
    pub tokenizer: Tokenizer,
    producer: HashMap<String, usize>,      // token -> merge index that produces it
    uses: HashMap<String, HashSet<usize>>, // token -> merge indices where used as input
    used_ids: HashSet<u32>,
    next_id: u32,
}
```

### Consistency Guarantees

1. **Valid merges** - After any operation, all merge results exist in vocab
2. **Cascade removal** - Removing a token removes all dependent merges
3. **Proper merge chains** - New tokens get valid merge rules
4. **No orphan merges** - Merges with missing inputs are removed

## Performance

The tool uses several optimizations for large tokenizers:

- **Index-based lookups** - O(1) lookup for token producers and users
- **Batch pre-computation** - Sync operations pre-compute all tokens to remove
- **Progress reporting** - Real-time progress with ETA for long operations
- **Efficient serialization** - Vocab is saved sorted by ID for consistency

### Example Performance

On a 256K vocab tokenizer:

- Validation: ~100ms
- Stats: ~50ms
- Shrink 1000 tokens: ~2-5s
- Sync 18K single-chars: ~30-60s

## Examples

### Complete Workflow: Training a Domain-Specific Tokenizer

```bash
# 1. Train new tokenizer on your corpus (using SentencePiece or similar)
# ... your training process ...

# 2. Check the new tokenizer
bpe-editor stats --input my_tokenizer.json

# 3. Validate merges
bpe-editor validate --input my_tokenizer.json --dry-run

# 4. Sync single-chars from original (keep Unicode coverage)
bpe-editor sync-chars \
  --source gemma3_original.json \
  --target my_tokenizer.json \
  --output my_tokenizer_synced.json \
  --min-id 50000

# 5. Sync short tokens for better subword coverage
bpe-editor sync-short-tokens \
  --source gemma3_original.json \
  --target my_tokenizer_synced.json \
  --output my_tokenizer_final.json \
  --min-id 50000

# 6. Final validation
bpe-editor validate --input my_tokenizer_final.json --dry-run
bpe-editor stats --input my_tokenizer_final.json
```

### Fixing a Corrupted Tokenizer

```bash
# Check for issues
bpe-editor validate --input broken_tokenizer.json --dry-run

# Fix invalid merges
bpe-editor validate --input broken_tokenizer.json --output fixed_tokenizer.json

# Verify fix
bpe-editor validate --input fixed_tokenizer.json --dry-run
```

### Reducing Vocabulary Size

```bash
# See current size
bpe-editor stats --input large_tokenizer.json

# Preview removal of 10000 tokens
bpe-editor shrink \
  --input large_tokenizer.json \
  --output smaller.json \
  --count 10000 \
  --min-id 50000 \
  --dry-run

# Perform shrink
bpe-editor shrink \
  --input large_tokenizer.json \
  --output smaller.json \
  --count 10000 \
  --min-id 50000

# Verify result
bpe-editor stats --input smaller.json
```

## Tokenizer Format

This tool works with HuggingFace `tokenizer.json` format:

```json
{
  "version": "1.0",
  "model": {
    "type": "BPE",
    "vocab": {
      "<pad>": 0,
      "<eos>": 1,
      "a": 100,
      "b": 101,
      "ab": 1000
    },
    "merges": ["a b"]
  }
}
```

**Requirements:**

- `model.type` must be `"BPE"`
- `vocab` is a map of token string to integer ID
- `merges` is an array of `"A B"` strings (space-separated pairs)

## Error Handling

The tool provides detailed error messages:

```
Error: Failed to read: "nonexistent.json"
Caused by: No such file or directory (os error 2)
```

```
Error: Failed to parse tokenizer.json
Caused by: expected value at line 1 column 1
```

```
Error: Only BPE tokenizers are supported
```

## License

MIT

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request
