"""Type stubs for bpe_tokenizer_editor."""

from typing import Dict, List, Optional, Tuple

__version__: str

class ValidationResult:
    """Result of validating merges."""
    
    @property
    def valid_count(self) -> int:
        """Number of valid merges."""
        ...
    
    @property
    def invalid_count(self) -> int:
        """Number of invalid merges."""
        ...
    
    @property
    def invalid_merges(self) -> List[Tuple[int, str, str]]:
        """List of (index, token_a, token_b) for invalid merges."""
        ...

class AdditionResult:
    """Result of token addition operation."""
    
    @property
    def token(self) -> str:
        """The token that was added."""
        ...
    
    @property
    def added(self) -> bool:
        """Whether the token was actually added (False if already existed)."""
        ...
    
    @property
    def method(self) -> str:
        """Method used: 'single_char', 'longest_prefix', 'char_chain', or 'already_exists'."""
        ...
    
    @property
    def added_merges(self) -> List[Tuple[str, str]]:
        """List of (token_a, token_b) merges that were added."""
        ...

class RemovalResult:
    """Result of token removal operation."""
    
    @property
    def root_token(self) -> str:
        """The token that was requested to be removed."""
        ...
    
    @property
    def removed_tokens(self) -> List[str]:
        """All tokens that were removed (including cascade)."""
        ...
    
    @property
    def removed_merges(self) -> List[Tuple[str, str]]:
        """All merges that were removed."""
        ...

class ShrinkResult:
    """Result of vocabulary shrink operation."""
    
    @property
    def initial_vocab_size(self) -> int:
        """Vocabulary size before shrinking."""
        ...
    
    @property
    def initial_merges_count(self) -> int:
        """Number of merges before shrinking."""
        ...
    
    @property
    def final_vocab_size(self) -> int:
        """Vocabulary size after shrinking."""
        ...
    
    @property
    def final_merges_count(self) -> int:
        """Number of merges after shrinking."""
        ...
    
    @property
    def tokens_removed_count(self) -> int:
        """Number of root tokens removed."""
        ...
    
    @property
    def total_tokens_removed(self) -> int:
        """Total tokens removed including cascade."""
        ...
    
    @property
    def total_merges_removed(self) -> int:
        """Total merges removed."""
        ...

class TokenizerStats:
    """Comprehensive tokenizer statistics."""
    
    @property
    def vocab_size(self) -> int:
        """Total vocabulary size."""
        ...
    
    @property
    def merges_count(self) -> int:
        """Total number of merges."""
        ...
    
    @property
    def single_char_count(self) -> int:
        """Number of single-character tokens."""
        ...
    
    @property
    def special_token_count(self) -> int:
        """Number of special tokens (<...> or [...])."""
        ...
    
    @property
    def min_token_id(self) -> int:
        """Minimum token ID in vocabulary."""
        ...
    
    @property
    def max_token_id(self) -> int:
        """Maximum token ID in vocabulary."""
        ...
    
    @property
    def length_distribution(self) -> List[Tuple[int, int]]:
        """List of (char_length, count) pairs sorted by length."""
        ...

class BPETokenizerEditor:
    """
    High-performance editor for HuggingFace BPE tokenizer.json files.
    
    This class provides methods to load, edit, validate, and save BPE tokenizers
    with consistency guarantees. All operations ensure that merge rules remain
    valid after modifications.
    
    Example:
        >>> editor = BPETokenizerEditor("tokenizer.json")
        >>> print(f"Vocab: {editor.vocab_size}, Merges: {editor.merges_count}")
        >>> 
        >>> # Add a new token
        >>> result = editor.add_token("merhaba")
        >>> print(f"Added: {result.added}, Method: {result.method}")
        >>> 
        >>> # Validate and save
        >>> validation = editor.validate_merges()
        >>> print(f"Valid: {validation.valid_count}, Invalid: {validation.invalid_count}")
        >>> editor.save("tokenizer_modified.json")
    """
    
    def __init__(self, path: str) -> None:
        """
        Load a tokenizer from a JSON file.
        
        Args:
            path: Path to the tokenizer.json file
            
        Raises:
            IOError: If the file cannot be read
            ValueError: If the tokenizer is not BPE type
        """
        ...
    
    @staticmethod
    def from_json(json_str: str) -> "BPETokenizerEditor":
        """
        Create a new editor from JSON string.
        
        Args:
            json_str: JSON string containing the tokenizer
            
        Returns:
            BPETokenizerEditor instance
            
        Raises:
            ValueError: If JSON is invalid or tokenizer is not BPE type
        """
        ...
    
    def save(self, path: str) -> None:
        """
        Save the tokenizer to a JSON file.
        
        Args:
            path: Output path for the tokenizer.json file
            
        Raises:
            IOError: If the file cannot be written
        """
        ...
    
    def to_json(self) -> str:
        """
        Export tokenizer to JSON string.
        
        Returns:
            JSON string representation of the tokenizer
        """
        ...
    
    @property
    def vocab_size(self) -> int:
        """Get the current vocabulary size."""
        ...
    
    @property
    def merges_count(self) -> int:
        """Get the current number of merges."""
        ...
    
    def has_token(self, token: str) -> bool:
        """
        Check if a token exists in the vocabulary.
        
        Args:
            token: Token string to check
            
        Returns:
            True if token exists, False otherwise
        """
        ...
    
    def get_token_id(self, token: str) -> Optional[int]:
        """
        Get the ID of a token.
        
        Args:
            token: Token string
            
        Returns:
            Token ID or None if not found
        """
        ...
    
    def get_token_by_id(self, id: int) -> Optional[str]:
        """
        Get a token by its ID.
        
        Args:
            id: Token ID
            
        Returns:
            Token string or None if not found
        """
        ...
    
    def get_vocab(self) -> Dict[str, int]:
        """
        Get all tokens in the vocabulary.
        
        Returns:
            Dictionary mapping token strings to their IDs
        """
        ...
    
    def get_merges(self) -> List[Tuple[str, str]]:
        """
        Get all merges as list of tuples.
        
        Returns:
            List of (token_a, token_b) merge pairs
        """
        ...
    
    def get_stats(self) -> TokenizerStats:
        """
        Get comprehensive tokenizer statistics.
        
        Returns:
            TokenizerStats object with detailed statistics
        """
        ...
    
    def validate_merges(self) -> ValidationResult:
        """
        Validate all merges - check that each merge result exists in vocabulary.
        
        Returns:
            ValidationResult with valid/invalid merge counts and details
        """
        ...
    
    def remove_invalid_merges(self) -> int:
        """
        Remove all invalid merges from the tokenizer.
        
        Returns:
            Number of invalid merges removed
        """
        ...
    
    def add_token(self, token: str) -> AdditionResult:
        """
        Add a token with proper merge chain creation.
        
        This method intelligently adds a token using one of these strategies:
        - Single character: Added directly to vocab
        - Longest prefix: If a prefix exists, adds prefix + suffix -> token merge
        - Character chain: Builds merge chain a+b->ab, ab+c->abc, etc.
        
        Args:
            token: Token string to add
            
        Returns:
            AdditionResult with details about how the token was added
        """
        ...
    
    def add_tokens(self, tokens: List[str]) -> List[AdditionResult]:
        """
        Add multiple tokens.
        
        Args:
            tokens: List of token strings to add
            
        Returns:
            List of AdditionResult for each token
        """
        ...
    
    def add_token_atomic(self, token: str) -> bool:
        """
        Add a token without creating merge chains (for special tokens).
        
        Args:
            token: Token string to add
            
        Returns:
            True if token was added, False if it already existed
        """
        ...
    
    def remove_token(self, token: str) -> RemovalResult:
        """
        Remove a token and all its dependencies.
        
        This performs cascade removal:
        - Removes all merges that use this token as input
        - Removes all merges that produce this token
        - Removes all tokens that can only be produced by removed merges
        
        Args:
            token: Token string to remove
            
        Returns:
            RemovalResult with details about what was removed
        """
        ...
    
    def remove_tokens(self, tokens: List[str]) -> List[RemovalResult]:
        """
        Remove multiple tokens.
        
        Args:
            tokens: List of token strings to remove
            
        Returns:
            List of RemovalResult for each token
        """
        ...
    
    def shrink(self, count: int, min_id: int = 0) -> ShrinkResult:
        """
        Shrink vocabulary by removing N longest non-special tokens.
        
        Selection criteria (in order):
        - Longest tokens first (by character count)
        - Highest token IDs first (among same length)
        - Never removes single-character tokens
        - Never removes special tokens (<...> or [...])
        
        Args:
            count: Number of tokens to remove
            min_id: Only consider tokens with ID >= min_id (default: 0)
            
        Returns:
            ShrinkResult with details about the shrink operation
        """
        ...
    
    def find_tokens_to_shrink(self, count: int, min_id: int = 0) -> List[Tuple[str, int, int]]:
        """
        Find tokens that would be removed by shrink operation (preview).
        
        Args:
            count: Number of tokens to find
            min_id: Only consider tokens with ID >= min_id (default: 0)
            
        Returns:
            List of (token, id, char_length) tuples
        """
        ...
    
    def get_single_char_tokens(self) -> List[Tuple[str, int]]:
        """
        Get all single-character tokens.
        
        Returns:
            List of (token, id) tuples
        """
        ...
    
    def sync_single_chars(self, source_path: str, min_id: int = 0) -> Dict[str, int]:
        """
        Sync single-character tokens from another tokenizer.
        
        Copies missing single-char tokens from source, removing longest
        tokens to maintain vocabulary size.
        
        Args:
            source_path: Path to source tokenizer.json
            min_id: Only remove tokens with ID >= min_id (default: 0)
            
        Returns:
            Dictionary with sync operation details
        """
        ...
    
    def add_tokens_keep_size(
        self, 
        tokens: List[str], 
        whitelist: Optional[List[str]] = None
    ) -> Dict[str, int]:
        """
        Add tokens while keeping vocabulary size fixed.
        
        Adds new tokens and automatically removes other tokens to maintain
        the original vocabulary size.
        
        Args:
            tokens: List of tokens to add
            whitelist: Optional list of tokens that should never be removed
            
        Returns:
            Dictionary with operation details
        """
        ...
    
    def __repr__(self) -> str:
        ...
    
    def __str__(self) -> str:
        ...
