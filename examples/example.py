#!/usr/bin/env python3
"""
Example script demonstrating the usage of bpe_tokenizer_editor.

This script shows how to:
1. Load a tokenizer
2. Get statistics
3. Validate merges
4. Add and remove tokens
5. Shrink vocabulary
6. Save the modified tokenizer
"""

from bpe_tokenizer_editor import BPETokenizerEditor


def main():
    # Example tokenizer path - replace with your actual tokenizer
    tokenizer_path = "tokenizer.json"
    
    try:
        # Load the tokenizer
        print("=" * 60)
        print("Loading tokenizer...")
        editor = BPETokenizerEditor(tokenizer_path)
        print(f"Loaded: {editor}")
        
        # Get statistics
        print("\n" + "=" * 60)
        print("Tokenizer Statistics:")
        stats = editor.get_stats()
        print(f"  Vocabulary size: {stats.vocab_size:,}")
        print(f"  Number of merges: {stats.merges_count:,}")
        print(f"  Single-char tokens: {stats.single_char_count:,}")
        print(f"  Special tokens: {stats.special_token_count:,}")
        print(f"  Token ID range: {stats.min_token_id} - {stats.max_token_id}")
        print("  Length distribution (top 10):")
        for length, count in stats.length_distribution[:10]:
            print(f"    {length} chars: {count:,} tokens")
        
        # Validate merges
        print("\n" + "=" * 60)
        print("Validating merges...")
        validation = editor.validate_merges()
        print(f"  Valid merges: {validation.valid_count:,}")
        print(f"  Invalid merges: {validation.invalid_count:,}")
        
        if validation.invalid_count > 0:
            print("  First 5 invalid merges:")
            for idx, token_a, token_b in validation.invalid_merges[:5]:
                print(f"    [{idx}] '{token_a}' + '{token_b}' -> '{token_a}{token_b}'")
        
        # Check some tokens
        print("\n" + "=" * 60)
        print("Token examples:")
        sample_tokens = ["the", "hello", "<pad>", "a"]
        for token in sample_tokens:
            if editor.has_token(token):
                token_id = editor.get_token_id(token)
                print(f"  '{token}' -> ID {token_id}")
            else:
                print(f"  '{token}' -> NOT IN VOCAB")
        
        # Add a token (example)
        print("\n" + "=" * 60)
        print("Adding tokens example...")
        test_tokens = ["testtoken123", "merhaba", "dünya"]
        for token in test_tokens:
            if not editor.has_token(token):
                result = editor.add_token(token)
                print(f"  Added '{token}':")
                print(f"    Method: {result.method}")
                print(f"    Added merges: {len(result.added_merges)}")
            else:
                print(f"  '{token}' already exists")
        
        # Preview shrink
        print("\n" + "=" * 60)
        print("Preview: Tokens that would be removed by shrink(100)...")
        preview = editor.find_tokens_to_shrink(count=10, min_id=0)
        if preview:
            print("  Top 10 candidates for removal:")
            for token, token_id, length in preview:
                print(f"    '{token}' (ID={token_id}, length={length})")
        else:
            print("  No candidates found")
        
        # Show some merges
        print("\n" + "=" * 60)
        print("Sample merges (first 10):")
        merges = editor.get_merges()[:10]
        for a, b in merges:
            print(f"  '{a}' + '{b}' -> '{a}{b}'")
        
        print("\n" + "=" * 60)
        print("Done! The tokenizer was NOT saved (this is just a demo).")
        print("To save changes, call: editor.save('output.json')")
        
    except FileNotFoundError:
        print(f"Error: Could not find '{tokenizer_path}'")
        print("\nTo run this example, provide a valid tokenizer.json file.")
        print("You can download one from HuggingFace, e.g.:")
        print("  https://huggingface.co/gpt2/blob/main/tokenizer.json")
        
    except Exception as e:
        print(f"Error: {e}")
        raise


if __name__ == "__main__":
    main()
