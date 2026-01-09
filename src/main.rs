//! BPE Tokenizer Editor - A tool for editing BPE tokenizers with consistency guarantees

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use bpe_tokenizer_editor::cli::{Args, Commands};
use bpe_tokenizer_editor::BPETokenizerEditor;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Validate {
            input,
            dry_run,
            output,
        } => {
            cmd_validate(&input, !dry_run, output)?;
        }
        Commands::Add {
            input,
            output,
            tokens,
            keep_size,
            whitelist,
        } => {
            cmd_add(&input, &output, &tokens, keep_size, whitelist)?;
        }
        Commands::Remove {
            input,
            output,
            tokens,
        } => {
            cmd_remove(&input, &output, &tokens)?;
        }
        Commands::Stats { input } => {
            cmd_stats(&input)?;
        }
        Commands::Shrink {
            input,
            output,
            count,
            min_id,
            dry_run,
            save_removed: _,
        } => {
            cmd_shrink(&input, &output, count, min_id, dry_run)?;
        }
        Commands::SyncChars {
            source,
            target,
            output,
            min_id,
            dry_run,
            save_report: _,
        } => {
            cmd_sync_chars(&source, &target, &output, min_id, dry_run)?;
        }
        Commands::SyncShortTokens {
            source,
            target,
            output,
            min_len,
            max_len,
            min_id,
            dry_run,
            save_report: _,
        } => {
            cmd_sync_short_tokens(&source, &target, &output, min_len, max_len, min_id, dry_run)?;
        }
        Commands::Reindex {
            input,
            output,
            dry_run,
        } => {
            cmd_reindex(&input, &output, dry_run)?;
        }
    }

    Ok(())
}

fn load_tokens_from_json(path: &PathBuf) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read tokens file: {:?}", path))?;
    let tokens: Vec<String> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse tokens JSON: {:?}", path))?;
    Ok(tokens)
}

fn cmd_validate(input: &PathBuf, fix: bool, output: Option<PathBuf>) -> Result<()> {
    println!("Loading tokenizer from: {:?}", input);
    let mut editor = BPETokenizerEditor::load(input)?;

    println!("Vocab size: {}", editor.vocab_size());
    println!("Merge count: {}", editor.merges_count());

    let (_, invalid) = editor.validate_merges();

    if invalid.is_empty() {
        println!("\n✓ All {} merges are valid!", editor.merges_count());
    } else {
        println!("\n✗ Found {} invalid merges:", invalid.len());
        for (i, (idx, merge)) in invalid.iter().take(20).enumerate() {
            println!(
                "  {}. Merge[{}]: '{}' + '{}' -> '{}' (not in vocab)",
                i + 1,
                idx,
                merge.0,
                merge.1,
                merge.result()
            );
        }
        if invalid.len() > 20 {
            println!("  ... and {} more", invalid.len() - 20);
        }

        if fix {
            println!("\nRemoving invalid merges...");
            let removed = editor.remove_invalid_merges();
            println!("Removed {} invalid merges", removed);

            let out_path = output.unwrap_or_else(|| input.clone());
            editor.save(&out_path)?;
            println!("Saved fixed tokenizer to: {:?}", out_path);
        }
    }

    Ok(())
}

fn cmd_add(
    input: &PathBuf,
    output: &PathBuf,
    tokens_file: &PathBuf,
    keep_size: bool,
    whitelist_file: Option<PathBuf>,
) -> Result<()> {
    println!("Loading tokenizer from: {:?}", input);
    let mut editor = BPETokenizerEditor::load(input)?;

    let tokens = load_tokens_from_json(tokens_file)?;
    let whitelist: HashSet<String> = if let Some(wl_path) = whitelist_file {
        load_tokens_from_json(&wl_path)?.into_iter().collect()
    } else {
        HashSet::new()
    };

    let initial_size = editor.vocab_size();
    println!("Initial vocab size: {}", initial_size);
    println!("Adding {} tokens...", tokens.len());

    if keep_size {
        let result = editor.add_tokens_keep_size(&tokens, &whitelist);

        println!("\n=== Batch Add Results ===");
        println!("Tokens requested: {}", result.tokens_requested);
        println!("Tokens added: {}", result.tokens_added);
        println!("Already present: {}", result.tokens_already_present);
        println!("Merges added: {}", result.merges_added);
        println!("Tokens removed: {}", result.tokens_removed);
        println!("Final vocab size: {}", result.final_vocab_size);

        for add in result.additions.iter().take(10) {
            println!(
                "  + '{}' via {} ({} merges)",
                add.token,
                add.method,
                add.added_merges.len()
            );
        }

        for (rem, reason) in result.removals.iter().take(10) {
            println!(
                "  - '{}' (cascade: {} tokens, {} merges): {}",
                rem.root_token,
                rem.removed_tokens.len(),
                rem.removed_merges.len(),
                reason
            );
        }
    } else {
        for token in &tokens {
            let result = editor.add_token_with_merges(token);
            if result.added {
                println!(
                    "  + '{}' via {} ({} merges added)",
                    result.token,
                    result.method,
                    result.added_merges.len()
                );
            } else {
                println!("  = '{}' already exists", result.token);
            }
        }
    }

    editor.save(output)?;
    println!("\nSaved to: {:?}", output);
    println!("Final vocab size: {}", editor.vocab_size());
    println!("Final merges: {}", editor.merges_count());

    Ok(())
}

fn cmd_remove(input: &PathBuf, output: &PathBuf, tokens_file: &PathBuf) -> Result<()> {
    println!("Loading tokenizer from: {:?}", input);
    let mut editor = BPETokenizerEditor::load(input)?;

    let tokens = load_tokens_from_json(tokens_file)?;

    println!("Initial vocab size: {}", editor.vocab_size());
    println!("Removing {} tokens...", tokens.len());

    for token in &tokens {
        if !editor.has_token(token) {
            println!("  ? '{}' not found", token);
            continue;
        }

        let result = editor.remove_token_and_dependencies(token);
        println!(
            "  - '{}': removed {} tokens, {} merges",
            result.root_token,
            result.removed_tokens.len(),
            result.removed_merges.len()
        );
    }

    editor.save(output)?;
    println!("\nSaved to: {:?}", output);
    println!("Final vocab size: {}", editor.vocab_size());
    println!("Final merges: {}", editor.merges_count());

    Ok(())
}

fn cmd_stats(input: &PathBuf) -> Result<()> {
    println!("Loading tokenizer from: {:?}", input);
    let editor = BPETokenizerEditor::load(input)?;

    println!("\n=== Tokenizer Statistics ===");
    println!("Vocab size: {}", editor.vocab_size());
    println!("Merge count: {}", editor.merges_count());

    // Token length distribution
    let mut len_dist: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for tok in editor.tokenizer.model.vocab.keys() {
        let count = tok.chars().count();
        *len_dist.entry(count).or_default() += 1;
    }

    println!("\nToken length distribution:");
    let mut lengths: Vec<usize> = len_dist.keys().copied().collect();
    lengths.sort();
    for len in &lengths {
        println!("  {} chars: {} tokens", len, len_dist[len]);
    }

    // Special tokens
    let special_count = editor
        .tokenizer
        .model
        .vocab
        .keys()
        .filter(|t: &&String| {
            (t.starts_with('<') && t.ends_with('>')) || (t.starts_with('[') && t.ends_with(']'))
        })
        .count();
    println!("\nSpecial tokens (<...>, [...]): {}", special_count);

    // ID range
    let ids: Vec<u32> = editor.tokenizer.model.vocab.values().copied().collect();
    if let (Some(&min_id), Some(&max_id)) = (ids.iter().min(), ids.iter().max()) {
        println!("ID range: {} - {}", min_id, max_id);
    }

    // Validation
    let (_, invalid) = editor.validate_merges();
    if invalid.is_empty() {
        println!("\n✓ All merges are valid");
    } else {
        println!("\n✗ {} invalid merges found", invalid.len());
    }

    Ok(())
}

fn cmd_shrink(
    input: &PathBuf,
    output: &PathBuf,
    count: usize,
    min_id: u32,
    dry_run: bool,
) -> Result<()> {
    println!("Loading tokenizer from: {:?}", input);
    let mut editor = BPETokenizerEditor::load(input)?;

    println!("Current vocab size: {}", editor.vocab_size());
    println!("Current merge count: {}", editor.merges_count());
    println!(
        "\nFinding {} longest tokens with ID >= {}...",
        count, min_id
    );

    let candidates = editor.find_tokens_to_shrink(count, min_id);

    println!("\nTokens to remove ({} found):", candidates.len());
    for (i, (tok, id, len)) in candidates.iter().take(20).enumerate() {
        println!("  {}. '{}' (ID: {}, len: {})", i + 1, tok, id, len);
    }
    if candidates.len() > 20 {
        println!("  ... and {} more", candidates.len() - 20);
    }

    if dry_run {
        println!("\n[DRY RUN] No changes made.");
        return Ok(());
    }

    println!("\nRemoving tokens...");
    let result = editor.shrink_vocab(count, min_id);

    println!("\n=== Shrink Results ===");
    println!("Initial vocab: {}", result.initial_vocab_size);
    println!("Initial merges: {}", result.initial_merges_count);
    println!("Tokens requested: {}", result.tokens_requested);
    println!("Tokens found: {}", result.tokens_found);
    println!("Total tokens removed: {}", result.total_tokens_removed);
    println!("Total merges removed: {}", result.total_merges_removed);
    println!("Final vocab: {}", result.final_vocab_size);
    println!("Final merges: {}", result.final_merges_count);

    editor.save(output)?;
    println!("\nSaved to: {:?}", output);

    Ok(())
}

fn cmd_sync_chars(
    source: &PathBuf,
    target: &PathBuf,
    output: &PathBuf,
    min_id: u32,
    dry_run: bool,
) -> Result<()> {
    println!("Loading source tokenizer from: {:?}", source);
    let source_editor = BPETokenizerEditor::load(source)?;

    println!("Loading target tokenizer from: {:?}", target);
    let mut target_editor = BPETokenizerEditor::load(target)?;

    let source_chars = source_editor.get_single_char_tokens();
    let target_chars = target_editor.get_single_char_tokens();

    println!("\n=== Single-Char Token Analysis ===");
    println!("Source single-char tokens: {}", source_chars.len());
    println!("Target single-char tokens: {}", target_chars.len());

    let source_set: HashSet<String> = source_chars
        .iter()
        .map(|(t, _): &(String, u32)| t.clone())
        .collect();
    let target_set: HashSet<String> = target_chars
        .iter()
        .map(|(t, _): &(String, u32)| t.clone())
        .collect();

    let missing: Vec<&String> = source_set.difference(&target_set).collect();
    let extra: Vec<&String> = target_set.difference(&source_set).collect();

    println!("\nMissing in target: {} chars", missing.len());
    if !missing.is_empty() && missing.len() <= 50 {
        let sample: Vec<&str> = missing
            .iter()
            .take(50)
            .map(|s: &&String| s.as_str())
            .collect();
        println!("  Sample: {:?}", sample);
    }

    println!("Extra in target: {} chars", extra.len());

    if dry_run {
        println!("\n[DRY RUN] No changes made.");
        return Ok(());
    }

    if missing.is_empty() {
        println!("\nNo missing chars to sync.");
        return Ok(());
    }

    println!(
        "\n=== Syncing {} missing single-char tokens ===",
        missing.len()
    );
    println!(
        "Will remove longest tokens with ID >= {} to make room",
        min_id
    );

    let result = target_editor.sync_single_chars(&source_chars, min_id);

    println!("\n=== Sync Results ===");
    println!("Initial vocab: {}", result.initial_vocab_size);
    println!("Initial merges: {}", result.initial_merges_count);
    println!("Chars in source: {}", result.chars_in_source);
    println!("Already present: {}", result.chars_already_present);
    println!("Chars added: {}", result.chars_added.len());
    println!("Tokens removed: {}", result.tokens_removed.len());
    println!(
        "Total cascade tokens removed: {}",
        result.total_tokens_removed
    );
    println!("Total merges removed: {}", result.total_merges_removed);
    println!("Final vocab: {}", result.final_vocab_size);
    println!("Final merges: {}", result.final_merges_count);

    if result.chars_added.len() <= 20 {
        println!("\nChars added:");
        for info in &result.chars_added {
            println!("  + '{}' (source ID: {})", info.char_token, info.source_id);
        }
    }

    target_editor.save(output)?;
    println!("\nSaved to: {:?}", output);

    Ok(())
}

fn cmd_sync_short_tokens(
    source: &PathBuf,
    target: &PathBuf,
    output: &PathBuf,
    min_len: usize,
    max_len: usize,
    min_id: u32,
    dry_run: bool,
) -> Result<()> {
    println!("Loading source tokenizer from: {:?}", source);
    let source_editor = BPETokenizerEditor::load(source)?;

    println!("Loading target tokenizer from: {:?}", target);
    let mut target_editor = BPETokenizerEditor::load(target)?;

    let source_tokens = source_editor.get_tokens_by_length(min_len, max_len);
    let source_merges: Vec<(String, String)> = source_editor
        .tokenizer
        .model
        .merges
        .iter()
        .map(|m| (m.0.clone(), m.1.clone()))
        .collect();
    let target_tokens = target_editor.get_tokens_by_length(min_len, max_len);

    println!(
        "\n=== Short Token Analysis ({}-{} chars) ===",
        min_len, max_len
    );
    println!("Source tokens: {}", source_tokens.len());
    println!("Target tokens: {}", target_tokens.len());
    println!("Source merges: {}", source_merges.len());

    let source_set: HashSet<String> = source_tokens
        .iter()
        .map(|(t, _): &(String, u32)| t.clone())
        .collect();
    let target_set: HashSet<String> = target_tokens
        .iter()
        .map(|(t, _): &(String, u32)| t.clone())
        .collect();

    let missing: Vec<&String> = source_set.difference(&target_set).collect();
    let extra: Vec<&String> = target_set.difference(&source_set).collect();

    println!("\nMissing in target: {} tokens", missing.len());
    if !missing.is_empty() && missing.len() <= 30 {
        let sample: Vec<&str> = missing
            .iter()
            .take(30)
            .map(|s: &&String| s.as_str())
            .collect();
        println!("  Sample: {:?}", sample);
    }

    println!("Extra in target: {} tokens", extra.len());

    if dry_run {
        println!("\n[DRY RUN] No changes made.");
        return Ok(());
    }

    if missing.is_empty() {
        println!("\nNo missing tokens to sync.");
        return Ok(());
    }

    println!(
        "\n=== Syncing {} missing short tokens with merges ===",
        missing.len()
    );
    println!(
        "Will remove longest tokens with ID >= {} to make room",
        min_id
    );

    let result = target_editor.sync_short_tokens(&source_tokens, &source_merges, min_id);

    println!("\n=== Sync Results ===");
    println!("Initial vocab: {}", result.initial_vocab_size);
    println!("Initial merges: {}", result.initial_merges_count);
    println!("Tokens in source: {}", result.tokens_in_source);
    println!("Already present: {}", result.tokens_already_present);
    println!("Tokens added: {}", result.tokens_added.len());
    println!("Merges added: {}", result.merges_added);
    println!("Merges already present: {}", result.merges_already_present);
    println!("Tokens removed: {}", result.tokens_removed.len());
    println!(
        "Total cascade tokens removed: {}",
        result.total_tokens_removed
    );
    println!("Total merges removed: {}", result.total_merges_removed);
    println!("Final vocab: {}", result.final_vocab_size);
    println!("Final merges: {}", result.final_merges_count);

    if result.tokens_added.len() <= 20 {
        println!("\nTokens added:");
        for info in &result.tokens_added {
            println!(
                "  + '{}' (len: {}, source ID: {})",
                info.token, info.length, info.source_id
            );
        }
    }

    target_editor.save(output)?;
    println!("\nSaved to: {:?}", output);

    Ok(())
}

fn cmd_reindex(input: &PathBuf, output: &PathBuf, dry_run: bool) -> Result<()> {
    println!("Loading tokenizer from: {:?}", input);
    let mut editor = BPETokenizerEditor::load(input)?;

    println!("Vocab size: {}", editor.vocab_size());
    println!("Merge count: {}", editor.merges_count());

    // Check for gaps
    let (has_gaps, total_gaps, min_id, max_id) = editor.check_vocab_gaps();

    println!("\n=== Vocabulary ID Analysis ===");
    println!("Current ID range: {} - {}", min_id, max_id);
    println!(
        "Expected dense range: 0 - {}",
        editor.vocab_size().saturating_sub(1)
    );

    if has_gaps {
        println!("Total gaps in ID space: {}", total_gaps);
        if min_id > 0 {
            println!(
                "  - Gap at start (IDs 0-{}): {} unused IDs",
                min_id - 1,
                min_id
            );
        }
        let internal_gaps = total_gaps - min_id as usize;
        if internal_gaps > 0 {
            println!("  - Internal gaps: {} unused IDs", internal_gaps);
        }
    } else {
        println!("✓ Vocabulary IDs are already sequential (no gaps)");
        if !dry_run {
            // Still save in case user wants a copy
            editor.save(output)?;
            println!("\nSaved to: {:?}", output);
        }
        return Ok(());
    }

    if dry_run {
        println!("\n[DRY RUN] No changes made.");
        println!("Run without --dry-run to reindex the vocabulary.");
        return Ok(());
    }

    println!("\n=== Reindexing Vocabulary ===");
    let result = editor.reindex_vocab();

    println!("\n=== Reindex Results ===");
    println!("Vocab size: {}", result.vocab_size);
    println!("Merges count: {}", result.merges_count);
    println!(
        "Old ID range: {} - {}",
        result.old_min_id, result.old_max_id
    );
    println!(
        "New ID range: {} - {}",
        result.new_min_id, result.new_max_id
    );
    println!("IDs remapped: {}", result.ids_remapped);
    println!("Gaps removed: {}", result.gaps_removed);

    editor.save(output)?;
    println!("\n✓ Saved reindexed tokenizer to: {:?}", output);

    // Validate the result
    let (still_has_gaps, _, _, _) = editor.check_vocab_gaps();
    if still_has_gaps {
        println!("⚠ Warning: Vocabulary still has gaps after reindexing");
    } else {
        println!("✓ Vocabulary IDs are now sequential");
    }

    Ok(())
}
