# arwen-codesign

Ad-hoc code signing for Mach-O binaries, matching Apple's codesign behavior for linker-signed binaries.

This crate was originally part of [goblin-ext](https://github.com/wolfv/goblin-ext) and has been migrated into the arwen workspace.

## Features

- Ad-hoc code signing for Mach-O binaries
- SHA-256 hashing for code pages
- Support for hardened runtime flag (`CS_RUNTIME`)
- Support for linker-signed flag (`CS_LINKER_SIGNED`)
- Entitlements preservation or custom injection
- Both 32-bit and 64-bit binary support
- 4KB page-aligned signature blocks

## Usage

### Basic Ad-hoc Signing

```rust
use arwen_codesign::{adhoc_sign, AdhocSignOptions};

let signed = adhoc_sign(data, &AdhocSignOptions::new("com.example.myapp"))?;
```

### With Hardened Runtime and Preserved Entitlements

```rust
use arwen_codesign::{adhoc_sign, AdhocSignOptions, Entitlements};

let options = AdhocSignOptions::new("com.example.myapp")
    .with_hardened_runtime()
    .with_entitlements(Entitlements::Preserve);
let signed = adhoc_sign(data, &options)?;
```

### File-based API

```rust
use arwen_codesign::{adhoc_sign_file, AdhocSignOptions, Entitlements};
use std::path::Path;

let options = AdhocSignOptions::new("com.example.myapp")
    .with_entitlements(Entitlements::Preserve);
adhoc_sign_file(Path::new("/path/to/binary"), &options)?;
```

## Testing

Python tests are available in the workspace `tests/python_integration/codesign/`:

- `test_codesign.py` - Comprehensive tests comparing against Apple's codesign tool
- `analyze_codesign.py` - Analysis tool for inspecting code signatures

Test assets are located in `tests/data/macho/codesign/` and include various signed/unsigned Mach-O binaries:
- `test_exe_adhoc` - Ad-hoc signed executable
- `test_exe_fat` - Universal/fat binary
- `test_exe_hardened` - Executable with hardened runtime
- `test_exe_linker_signed` - Linker-signed executable
- `test_exe_unsigned` - Unsigned executable

## License

MIT
