.PHONY: build build-release develop test clean publish help

# Default target
help:
	@echo "BPE Tokenizer Editor - Build Commands"
	@echo ""
	@echo "Rust CLI:"
	@echo "  make rust-build      Build Rust CLI (debug)"
	@echo "  make rust-release    Build Rust CLI (release)"
	@echo ""
	@echo "Python Package:"
	@echo "  make develop         Build and install Python package (development mode)"
	@echo "  make build           Build Python wheel (release)"
	@echo "  make test            Run Python tests"
	@echo ""
	@echo "Publishing:"
	@echo "  make publish-test    Publish to TestPyPI"
	@echo "  make publish         Publish to PyPI"
	@echo ""
	@echo "Utility:"
	@echo "  make clean           Clean build artifacts"

# Rust targets
rust-build:
	cargo build

rust-release:
	cargo build --release

# Python targets
develop:
	maturin develop --features python

build:
	maturin build --release --features python

test: develop
	pytest tests/ -v

# Publishing
publish-test: build
	maturin upload --repository testpypi target/wheels/*.whl

publish: build
	maturin upload target/wheels/*.whl

# Clean
clean:
	cargo clean
	rm -rf target/wheels/
	rm -rf .pytest_cache/
	rm -rf __pycache__/
	rm -rf python/bpe_tokenizer_editor/__pycache__/
	rm -rf *.egg-info/
	find . -name "*.pyc" -delete
	find . -name "*.pyo" -delete
