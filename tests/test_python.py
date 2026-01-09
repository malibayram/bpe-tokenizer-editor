"""Tests for bpe_tokenizer_editor Python bindings."""

import json
import os
import tempfile

import pytest

# Sample minimal tokenizer for testing
SAMPLE_TOKENIZER = {
    "version": "1.0",
    "truncation": None,
    "padding": None,
    "added_tokens": [],
    "normalizer": None,
    "pre_tokenizer": None,
    "post_processor": None,
    "decoder": None,
    "model": {
        "type": "BPE",
        "dropout": None,
        "unk_token": "<unk>",
        "continuing_subword_prefix": None,
        "end_of_word_suffix": None,
        "fuse_unk": False,
        "byte_fallback": False,
        "ignore_merges": False,
        "vocab": {
            "<pad>": 0,
            "<eos>": 1,
            "<unk>": 2,
            "a": 100,
            "b": 101,
            "c": 102,
            "ab": 200,
            "abc": 300,
        },
        "merges": [
            ["a", "b"],
            ["ab", "c"],
        ]
    }
}


@pytest.fixture
def tokenizer_file():
    """Create a temporary tokenizer file for testing."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(SAMPLE_TOKENIZER, f)
        temp_path = f.name
    yield temp_path
    os.unlink(temp_path)


@pytest.fixture
def editor(tokenizer_file):
    """Create an editor instance for testing."""
    from bpe_tokenizer_editor import BPETokenizerEditor
    return BPETokenizerEditor(tokenizer_file)


class TestBPETokenizerEditor:
    """Test suite for BPETokenizerEditor."""
    
    def test_load_from_file(self, tokenizer_file):
        """Test loading tokenizer from file."""
        from bpe_tokenizer_editor import BPETokenizerEditor
        editor = BPETokenizerEditor(tokenizer_file)
        assert editor.vocab_size == 8
        assert editor.merges_count == 2
    
    def test_load_from_json(self):
        """Test loading tokenizer from JSON string."""
        from bpe_tokenizer_editor import BPETokenizerEditor
        json_str = json.dumps(SAMPLE_TOKENIZER)
        editor = BPETokenizerEditor.from_json(json_str)
        assert editor.vocab_size == 8
    
    def test_save_and_reload(self, editor):
        """Test saving and reloading tokenizer."""
        from bpe_tokenizer_editor import BPETokenizerEditor
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            temp_path = f.name
        
        try:
            editor.save(temp_path)
            reloaded = BPETokenizerEditor(temp_path)
            assert reloaded.vocab_size == editor.vocab_size
            assert reloaded.merges_count == editor.merges_count
        finally:
            os.unlink(temp_path)
    
    def test_has_token(self, editor):
        """Test has_token method."""
        assert editor.has_token("a") is True
        assert editor.has_token("ab") is True
        assert editor.has_token("<pad>") is True
        assert editor.has_token("xyz") is False
    
    def test_get_token_id(self, editor):
        """Test get_token_id method."""
        assert editor.get_token_id("a") == 100
        assert editor.get_token_id("ab") == 200
        assert editor.get_token_id("<pad>") == 0
        assert editor.get_token_id("nonexistent") is None
    
    def test_get_token_by_id(self, editor):
        """Test get_token_by_id method."""
        assert editor.get_token_by_id(100) == "a"
        assert editor.get_token_by_id(200) == "ab"
        assert editor.get_token_by_id(0) == "<pad>"
        assert editor.get_token_by_id(99999) is None
    
    def test_get_vocab(self, editor):
        """Test get_vocab method."""
        vocab = editor.get_vocab()
        assert isinstance(vocab, dict)
        assert vocab["a"] == 100
        assert vocab["<pad>"] == 0
        assert len(vocab) == 8
    
    def test_get_merges(self, editor):
        """Test get_merges method."""
        merges = editor.get_merges()
        assert isinstance(merges, list)
        assert len(merges) == 2
        assert ("a", "b") in merges
        assert ("ab", "c") in merges
    
    def test_get_stats(self, editor):
        """Test get_stats method."""
        stats = editor.get_stats()
        assert stats.vocab_size == 8
        assert stats.merges_count == 2
        assert stats.single_char_count == 3  # a, b, c
        assert stats.special_token_count == 3  # <pad>, <eos>, <unk>
    
    def test_validate_merges(self, editor):
        """Test validate_merges method."""
        result = editor.validate_merges()
        assert result.valid_count == 2
        assert result.invalid_count == 0
        assert len(result.invalid_merges) == 0
    
    def test_add_token_single_char(self, editor):
        """Test adding a single character token."""
        result = editor.add_token("x")
        assert result.added is True
        assert result.method == "single_char"
        assert editor.has_token("x")
    
    def test_add_token_existing(self, editor):
        """Test adding an existing token."""
        result = editor.add_token("a")
        assert result.added is False
        assert result.method == "already_exists"
    
    def test_add_token_with_prefix(self, editor):
        """Test adding a token that has a prefix in vocab."""
        result = editor.add_token("abx")
        assert result.added is True
        assert result.method == "longest_prefix"
        assert editor.has_token("abx")
    
    def test_add_token_char_chain(self, editor):
        """Test adding a token via character chain."""
        result = editor.add_token("xyz")
        assert result.added is True
        assert result.method == "char_chain"
        assert editor.has_token("xyz")
        assert editor.has_token("x")
        assert editor.has_token("y")
        assert editor.has_token("z")
    
    def test_add_tokens(self, editor):
        """Test adding multiple tokens."""
        results = editor.add_tokens(["x", "y", "z"])
        assert len(results) == 3
        assert all(r.added for r in results)
    
    def test_add_token_atomic(self, editor):
        """Test adding a token atomically."""
        result = editor.add_token_atomic("<special>")
        assert result is True
        assert editor.has_token("<special>")
        
        # Adding again should return False
        result = editor.add_token_atomic("<special>")
        assert result is False
    
    def test_remove_token(self, editor):
        """Test removing a token."""
        assert editor.has_token("abc")
        result = editor.remove_token("abc")
        assert result.root_token == "abc"
        assert "abc" in result.removed_tokens
        assert not editor.has_token("abc")
    
    def test_remove_token_cascade(self, editor):
        """Test cascade removal of tokens."""
        # Removing 'ab' should also affect 'abc' since it depends on 'ab'
        initial_vocab_size = editor.vocab_size
        result = editor.remove_token("ab")
        assert "ab" in result.removed_tokens
        assert len(result.removed_tokens) >= 1
        assert editor.vocab_size < initial_vocab_size
    
    def test_remove_tokens(self, editor):
        """Test removing multiple tokens."""
        results = editor.remove_tokens(["abc"])
        assert len(results) == 1
        assert not editor.has_token("abc")
    
    def test_find_tokens_to_shrink(self, editor):
        """Test finding tokens to shrink."""
        candidates = editor.find_tokens_to_shrink(count=2, min_id=0)
        assert isinstance(candidates, list)
        # Should find 'abc' (length 3) first, then 'ab' (length 2)
        if len(candidates) > 0:
            assert candidates[0][2] >= candidates[-1][2]  # Sorted by length desc
    
    def test_shrink(self, editor):
        """Test shrinking vocabulary."""
        initial_size = editor.vocab_size
        result = editor.shrink(count=1, min_id=0)
        assert result.initial_vocab_size == initial_size
        assert result.final_vocab_size <= initial_size
    
    def test_get_single_char_tokens(self, editor):
        """Test getting single character tokens."""
        single_chars = editor.get_single_char_tokens()
        assert isinstance(single_chars, list)
        tokens = [t for t, _ in single_chars]
        assert "a" in tokens
        assert "b" in tokens
        assert "c" in tokens
    
    def test_to_json(self, editor):
        """Test exporting to JSON string."""
        json_str = editor.to_json()
        data = json.loads(json_str)
        assert data["model"]["type"] == "BPE"
        assert "vocab" in data["model"]
    
    def test_repr(self, editor):
        """Test string representation."""
        repr_str = repr(editor)
        assert "BPETokenizerEditor" in repr_str
        assert "vocab_size" in repr_str


class TestEdgeCases:
    """Test edge cases and error handling."""
    
    def test_invalid_file_path(self):
        """Test loading from non-existent file."""
        from bpe_tokenizer_editor import BPETokenizerEditor
        with pytest.raises(IOError):
            BPETokenizerEditor("/nonexistent/path/tokenizer.json")
    
    def test_invalid_json(self):
        """Test loading from invalid JSON."""
        from bpe_tokenizer_editor import BPETokenizerEditor
        with pytest.raises(ValueError):
            BPETokenizerEditor.from_json("not valid json")
    
    def test_non_bpe_tokenizer(self):
        """Test loading a non-BPE tokenizer."""
        from bpe_tokenizer_editor import BPETokenizerEditor
        non_bpe = dict(SAMPLE_TOKENIZER)
        non_bpe["model"] = dict(non_bpe["model"])
        non_bpe["model"]["type"] = "WordPiece"
        
        with pytest.raises(ValueError):
            BPETokenizerEditor.from_json(json.dumps(non_bpe))
    
    def test_empty_token_list(self, editor):
        """Test adding empty token list."""
        results = editor.add_tokens([])
        assert len(results) == 0
    
    def test_remove_nonexistent_token(self, editor):
        """Test removing non-existent tokens."""
        results = editor.remove_tokens(["nonexistent"])
        assert len(results) == 0  # Should not return result for non-existent token


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
