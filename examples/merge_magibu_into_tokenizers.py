"""Merge Magibu into three ready-to-load tokenizer directories."""

import shutil
from pathlib import Path

from bpe_tokenizer_editor import BPETokenizerEditor


ROOT = Path(__file__).resolve().parents[1] / "tokenizers"
SOURCE = ROOT / "magibu-64-tokenizer.json"
NAMES = ["Muse-Glimmer-30B", "Qwen3.8-27B", "gemma-4-31B-it"]
SUPPORT_FILES = ["chat_template.jinja", "processor_config.json", "tokenizer_config.json"]


for name in NAMES:
    original = ROOT / name
    merged = ROOT / f"{name}-merged"
    merged.mkdir(exist_ok=True)

    for filename in SUPPORT_FILES:
        source_file = original / filename
        if source_file.exists():
            shutil.copy2(source_file, merged / filename)

    editor = BPETokenizerEditor(str(original / "tokenizer.json"))
    result = editor.merge_from(str(SOURCE), max_vocab_size=2**18)
    editor.save(str(merged / "tokenizer.json"))
    print(
        f"{name}: {result.final_vocab_size:,} tokens, "
        f"{result.tokens_injected:,} injected, {result.representation}"
    )
