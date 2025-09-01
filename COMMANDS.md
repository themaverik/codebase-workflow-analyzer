# Codebase Workflow Analyzer - Command Reference

## Clean Build Process

### Full Clean Rebuild (Recommended when encountering issues)
```bash
# Step 1: Clean all build artifacts
cargo clean

# Step 2: Remove target directory completely  
rm -rf target/

# Step 3: Build release version (optimized)
cargo build --release
```

## Core Commands

### Analysis Commands
```bash
# Basic analysis (framework detection only)
./target/release/codebase-analyzer analyze --path /path/to/project

# SOTA fusion analysis (hierarchical multi-tier)
./target/release/codebase-analyzer analyze --path /path/to/project --enable-fusion

# Full analysis with LLM business intelligence
./target/release/codebase-analyzer analyze --path /path/to/project --enable-fusion --enable-llm

# Full analysis with document generation
./target/release/codebase-analyzer analyze --path /path/to/project --enable-fusion --enable-llm --generate-docs /path/to/output
```

### Cache Management
```bash
# Show cache statistics
./target/release/codebase-analyzer cache stats

# Clear all cached results
./target/release/codebase-analyzer cache clear

# Show cache configuration
./target/release/codebase-analyzer cache info
```

### LLM Setup
```bash
# Interactive Ollama setup (first time)
./target/release/codebase-analyzer setup-ollama
```

### Testing
```bash
# Test basic functionality
./target/release/codebase-analyzer test-basic

# Test LLM integration
./target/release/codebase-analyzer test-llm --enable-llm
```

## Alternative: Using cargo run

If you prefer using `cargo run`, always use the `--release` flag for performance:

```bash
# Basic analysis
cargo run --release --bin codebase-analyzer analyze --path /path/to/project

# Full analysis with docs
cargo run --release --bin codebase-analyzer analyze --path /path/to/project --enable-fusion --enable-llm --generate-docs /path/to/output
```

## Troubleshooting

### Issue: Old output after code changes
**Solution**: Perform clean rebuild
```bash
cargo clean
rm -rf target/
cargo build --release
```

### Issue: Command not found or permission denied
**Solution**: Ensure binary is built and executable
```bash
ls -la target/release/codebase-analyzer
chmod +x target/release/codebase-analyzer
```

### Issue: Out of memory during analysis
**Solution**: Use basic analysis or increase system RAM
```bash
# Fallback to basic analysis
./target/release/codebase-analyzer analyze --path /path/to/project
```

## Performance Notes

- **Release builds** are 10-50x faster than debug builds
- **Clean rebuilds** are necessary after significant code changes
- **Cache** improves repeat analysis performance significantly
- **LLM analysis** requires 4GB+ RAM for optimal performance