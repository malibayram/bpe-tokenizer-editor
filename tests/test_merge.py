"""Regression tests for native-format tokenizer merging."""

import json

from bpe_tokenizer_editor import BPETokenizerEditor


def make_tokenizer(vocab, merges, normalizer, pre_tokenizer, added_tokens=None):
    return {
        "version": "1.0",
        "truncation": None,
        "padding": None,
        "added_tokens": added_tokens or [],
        "normalizer": normalizer,
        "pre_tokenizer": pre_tokenizer,
        "post_processor": None,
        "decoder": None,
        "model": {
            "type": "BPE",
            "dropout": None,
            "unk_token": None,
            "continuing_subword_prefix": None,
            "end_of_word_suffix": None,
            "fuse_unk": False,
            "byte_fallback": False,
            "ignore_merges": False,
            "vocab": vocab,
            "merges": merges,
        },
    }


def test_merge_converts_to_byte_level_and_prioritizes_source(tmp_path):
    source = make_tokenizer(
        {
            "<unk>": 0,
            "<0xC4>": 1,
            "<0xB1>": 2,
            "▁": 3,
            "n": 4,
            "a": 5,
            "s": 6,
            "ı": 7,
            "l": 8,
            "na": 9,
            "nas": 10,
            "ıl": 11,
            "nasıl": 12,
            "▁nasıl": 13,
        },
        [["n", "a"], ["na", "s"], ["ı", "l"], ["nas", "ıl"], ["▁", "nasıl"]],
        {"type": "Replace", "pattern": {"String": " "}, "content": "▁"},
        None,
        [{"id": 0, "content": "<unk>", "special": True}],
    )
    target = make_tokenizer(
        {
            "Ġ": 0,
            "n": 1,
            "a": 2,
            "s": 3,
            "l": 4,
            "Ä": 5,
            "±": 6,
            "na": 7,
            "nas": 8,
            "target": 9,
        },
        [["n", "a"], ["na", "s"]],
        {"type": "NFC"},
        {"type": "ByteLevel"},
        [{"id": 10, "content": "<target-special>", "special": True}],
    )
    source_path = tmp_path / "source.json"
    source_path.write_text(json.dumps(source), encoding="utf-8")

    editor = BPETokenizerEditor.from_json(json.dumps(target))
    result = editor.merge_from(str(source_path), max_vocab_size=30)
    merged = json.loads(editor.to_json())

    assert result.representation == "ByteLevel (Ġ)"
    assert result.final_vocab_size <= 30
    assert "ĠnasÄ±l" in merged["model"]["vocab"]
    assert "▁nasıl" not in merged["model"]["vocab"]
    assert "<unk>" not in merged["model"]["vocab"]
    assert merged["model"]["merges"][0] == ["Ä", "±"]
    assert editor.validate_merges().invalid_count == 0


def test_merge_preserves_target_space_marker(tmp_path):
    source = make_tokenizer(
        {"▁": 0, "a": 1, "▁a": 2},
        [["▁", "a"]],
        {"type": "Replace", "pattern": {"String": " "}, "content": "▁"},
        None,
    )
    source_path = tmp_path / "source.json"
    source_path.write_text(json.dumps(source), encoding="utf-8")

    editor = BPETokenizerEditor.from_json(json.dumps(source))
    result = editor.merge_from(str(source_path), max_vocab_size=8)

    assert result.representation == "space marker (▁)"
    assert editor.has_token("▁a")
    assert editor.get_merges()[0] == ("▁", "a")
