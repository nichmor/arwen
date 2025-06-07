## Introduction

### What is `arwen`?

`arwen` is a command-line utility and Rust library designed to modify executable files and shared libraries.

Specifically, it targets the **ELF** format (commonly used on Linux, BSD, and other Unix-like systems) and the **Mach-O** format (used on macOS and iOS).

It allows you to inspect and rewrite various properties within these files that influence how they load and link with other libraries at runtime.

Think of `arwen` as a modern, unified, Rust-based alternative to the widely-used `patchelf` (for ELF files) and `install_name_tool` (for Mach-O files). It combines the core functionalities of both into a single tool.

## Quick Installation:

You can install `arwen` using Cargo, the Rust package manager:

```sh
cargo install arwen
```

## Why use `arwen`?

Modifying these executable properties is often necessary for:

- **Application Packaging:** Ensuring your application can find its bundled libraries regardless of where it's installed.
- **Relocation:** Making software work correctly after being moved to a different path.
- **Fixing Linkage Issues:** Correcting paths embedded during the build process that don't match the target environment.

## Core Features

### For ELF files:
- View and change the dynamic library search path (`RPATH` / `RUNPATH`).
- View and change the dynamic linker/interpreter path.
- View, add, remove, or replace required shared library dependencies (`DT_NEEDED` entries).

### For Mach-O files:
- View and change the library's identity (`Install Name` / `LC_ID_DYLIB`).
- View and change the paths to dependent dynamic libraries (`LC_LOAD_DYLIB`, etc.).
- View, add, or delete runtime search paths (`LC_RPATH`).

### General Features:
- **Cross-Platform:** Implemented in Rust, aiming for reliability and ease of building across different platforms.
- **Library Usage:** Can be used directly within your Rust projects to programmatically inspect or patch binaries.
- **Python Bindings:** Provides a Python interface for easier integration into Python-based workflows.


**Resources:**

* **Source Code Repository:** `[text](https://github.com/nichmor/arwen)`
* **Issue Tracker:** `[https://github.com/nichmor/arwen/issues]`
* **License:** `[https://github.com/nichmor/arwen/blob/main/LICENSE]`
