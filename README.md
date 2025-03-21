[![Pixi Badge][pixi-badge]][pixi-url]

[pixi-badge]:https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/prefix-dev/pixi/main/assets/badge/v0.json&style=flat-square
[pixi-url]: https://pixi.sh


# arwen: Cross-Platform Binary Patching Tool for Mach-O and ELF in Rust

## Overview

`arwen` is a cross-platform Rust implementation that combines functionality similar to `patchelf` (Linux) and `install_name_tool` (macOS) into a single, versatile tool for binary manipulation.

## Installation

You can install `arwen` using Cargo:

```sh
cargo install arwen
```

## Usage

### Mach-O Commands

#### RPath Operations
```sh
# Add an RPath
arwen macho add-rpath /usr/local/lib my_binary

# Change an existing RPath
arwen macho change-rpath /old/path /new/path my_binary

# Delete an RPath
arwen macho delete-rpath /unwanted/path my_binary
```

#### Library Install Name and library id Operations
```sh
# Change library install name
arwen macho change-install-name /old/libname.dylib /new/libname.dylib my_binary

# Change install ID of a shared library
arwen macho change-install-id /new/install/id.dylib my_library.dylib
```

### ELF Commands

#### Interpreter Operations
```sh
# Set ELF interpreter
arwen elf set-interpreter /path/to/new/interpreter my_elf_binary

# Print current interpreter
arwen elf print-interpreter my_elf_binary
```

#### ELF Header Operations
```sh
# Print OS ABI
arwen elf print-os-abi my_elf_binary

# Set OS ABI
arwen elf set-os-abi solaris my_elf_binary
```

#### RPATH Operations
```sh
# Set RPATH
arwen elf set-rpath /path1:/path2 my_elf_binary

# Add RPATH
arwen elf add-rpath /additional/path my_elf_binary

# Remove RPATH
arwen elf remove-rpath my_elf_binary

# Print current RPATH
arwen elf print-rpath my_elf_binary

# Shrink RPATH with allowed prefixes
arwen elf shrink-rpath --allowed-prefixes /usr/lib:/local/lib my_elf_binary
```

#### Dependency Management
```sh
# Add a needed library
arwen elf add-needed my_elf_binary libexample.so libotherexample.so

# Remove a needed library
arwen elf remove-needed my_elf_binary libexample.so libotherexample.so

# Replace a needed library
arwen elf replace-needed my_elf_binary old_lib.so new_lib.so

# Print needed libraries
arwen elf print-needed my_elf_binary
```

#### Executable Stack Control
```sh
# Check executable stack status
arwen elf print-exec-stack my_elf_binary

# Clear executable stack
arwen elf clear-exec-stack my_elf_binary

# Set executable stack
arwen elf set-exec-stack my_elf_binary
```

#### Symbol and Soname Operations
```sh
# Print soname
arwen elf print-soname my_elf_binary

# Set soname
arwen elf set-soname new_soname my_elf_binary

# Clear symbol version
arwen elf clear-symbol-version symbol_name my_elf_binary

# Rename dynamic symbols (using a map file)
arwen elf rename-dynamic-symbols symbol_map.txt my_elf_binary
```

### Post-Modification Considerations

#### Mach-O Re-signing (macOS)
After modifying a Mach-O binary, re-sign it:

```sh
codesign --force --sign - my_binary
```


## Integration Tests

We have comprehensive integration tests to validate feature parity with `install_name_tool` and `patchelf`, ensuring correctness and reliability.

## License

`arwen` is licensed under the MIT license.

## Contributions

Contributions are welcome! Feel free to open issues or submit pull requests.


## Status

`arwen` is currently in active development. The `API` and `CLI` are subject to change to improve user experience.

## Funding

This [project](https://nlnet.nl/project/ELF-rusttools/) is funded through [NGI0 Entrust](https://nlnet.nl/entrust), a fund established by [NLnet](https://nlnet.nl) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu) program.

Learn more at the [NLnet project page](https://nlnet.nl/project/ELF-rusttools).

[<img src="https://nlnet.nl/logo/banner.png" alt="NLnet foundation logo" width="40%" />](https://nlnet.nl)

[<img src="https://nlnet.nl/image/logos/NGI0_tag.svg" alt="NGI Zero Logo" width="40%" />](https://nlnet.nl/entrust)
