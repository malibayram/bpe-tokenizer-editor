"""Map merged tokenizer IDs to their original tokenizer model IDs."""

import json
from pathlib import Path

from tokenizers import Tokenizer


ROOT = Path(__file__).resolve().parents[1] / "tokenizers"
NAMES = ["Muse-Glimmer-30B", "Qwen3.8-27B", "gemma-4-31B-it"]


for name in NAMES:
    original = Tokenizer.from_file(str(ROOT / name / "tokenizer.json"))
    merged_dir = ROOT / f"{name}-merged"
    merged = Tokenizer.from_file(str(merged_dir / "tokenizer.json"))
    original_vocab = original.get_vocab()

    token_map = {}
    non_matched = 0
    for token, merged_id in sorted(merged.get_vocab().items(), key=lambda item: item[1]):
        if token in original_vocab:
            original_ids = [original_vocab[token]]
            non_matched += original_ids != [merged_id]
        else:
            original_ids = [piece.id for piece in original.model.tokenize(token)]
            non_matched += 1
        if not original_ids:
            raise RuntimeError(f"{name}: empty mapping for merged ID {merged_id}")
        token_map[merged_id] = original_ids

    output = merged_dir / "merged-to-original-token-ids.json"
    with output.open("w", encoding="utf-8") as file:
        json.dump(token_map, file, indent=2)
    print(f"{name}: {len(token_map):,} mappings; {non_matched:,} non-matched IDs")
