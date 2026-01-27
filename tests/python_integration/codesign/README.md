# Codesign Integration Tests

This directory contains Python integration tests for the `arwen-codesign` crate, which implements ad-hoc code signing for Mach-O binaries.

## Overview

These tests compare arwen's ad-hoc signing implementation against Apple's official `codesign` tool to ensure compatibility and correctness.

## Requirements

- **macOS only**: These tests require macOS and Apple's `codesign` tool
- **Python 3.8+** with pytest
- **Rust toolchain** to build the arwen codesign tool

## Running Tests

All tests are designed to run with pytest:

```bash
# Run all codesign tests
pytest tests/python_integration/codesign/

# Run with verbose output
pytest tests/python_integration/codesign/ -v

# Run in strict mode (require bit-for-bit identical output)
pytest tests/python_integration/codesign/ --strict-codesign

# Keep temporary files for inspection
pytest tests/python_integration/codesign/ --keep-temp

# Use a specific codesign tool binary
pytest tests/python_integration/codesign/ --goblin-tool=/path/to/tool

# Run specific test
pytest tests/python_integration/codesign/test_codesign.py::test_adhoc_signing_against_apple

# Run only basic (non-hardened) tests
pytest tests/python_integration/codesign/ -k basic

# Run only hardened runtime tests
pytest tests/python_integration/codesign/ -k hardened
```

## Test Files

- **`test_codesign.py`** - Main test suite with parametrized tests
  - `test_adhoc_signing_against_apple` - Tests basic and hardened runtime signing
  - `test_resign_linker_signed` - Tests re-signing linker-signed binaries
- **`analyze_codesign.py`** - Utility for analyzing and comparing code signatures
- **`conftest.py`** - Pytest configuration, fixtures, and helper functions

## Test Structure

Tests are parametrized to run against multiple binaries and configurations:
- **Binary types**: Executables, dylibs (if available)
- **Signing modes**: Basic ad-hoc signing, hardened runtime signing
- **Comparison modes**: Structural equivalence (default) or strict bit-for-bit matching

## Test Assets

Test binaries are located in `tests/data/macho/codesign/`:
- `test_exe_adhoc` - Ad-hoc signed executable
- `test_exe_fat` - Universal/fat binary
- `test_exe_hardened` - Executable with hardened runtime
- `test_exe_linker_signed` - Linker-signed executable
- `test_exe_unsigned` - Unsigned executable

## Test Modes

### Structural Equivalence (default)
Tests verify that signatures are functionally equivalent and pass Apple's `codesign -v` verification, even if not byte-for-byte identical.

### Strict Mode (`--strict`)
Tests require bit-for-bit identical output compared to Apple's codesign tool.

## Analyzing Signatures

Use `analyze_codesign.py` to inspect code signatures:

```bash
# Analyze a single binary
python tests/python_integration/codesign/analyze_codesign.py /path/to/binary

# Compare two binaries
python tests/python_integration/codesign/analyze_codesign.py binary1 binary2 -v
```
