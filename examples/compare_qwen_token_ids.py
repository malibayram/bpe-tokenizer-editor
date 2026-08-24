"""Check whether both Qwen tokenizers use the same ID for every token."""

from pathlib import Path

from tokenizers import Tokenizer


root = Path(__file__).resolve().parents[1] / "tokenizers"
qwen_35 = Tokenizer.from_file(str(root / "Qwen3.5-0.8B/tokenizer.json")).get_vocab()
qwen_38 = Tokenizer.from_file(str(root / "Qwen3.8-27B/tokenizer.json")).get_vocab()

different = {token for token in qwen_35 | qwen_38 if qwen_35.get(token) != qwen_38.get(token)}
print(f"Same token IDs: {not different}")
print(f"Different or missing tokens: {len(different):,}")
