[![Pixi Badge][pixi-badge]][pixi-url]

[pixi-badge]:https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/prefix-dev/pixi/main/assets/badge/v0.json&style=flat-square
[pixi-url]: https://pixi.sh

# arwen: Patching Mach-O and ELF Binaries in Rust

## Overview

`arwen` is a cross-platform Rust implementation of `patchelf` (Linux) and `install_name_tool` (macOS) in one tool.

## Installation

You can install `arwen` using Cargo:

```sh
cargo install arwen
```

## Usage

The CLI looks like this:

```sh
Usage: arwen <COMMAND>

Commands:
  delete-rpath         Delete a run path
  change-rpath         Change already existing run path
  add-rpath            Add a run path
  change-install-name  Change existing dylib load name
  change-install-id    Change dylib id. Works only if your object is a shared library
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Examples (Mach-O only)

### Add an RPath

To add an RPath (`/usr/local/lib`) to an existing binary:

```sh
arwen add-rpath /usr/local/lib my_binary
```

### Change an Existing RPath

If your binary has an RPath that needs to be changed, for example, from `/old/path` to `/new/path`:

```sh
arwen change-rpath /old/path /new/path my_binary
```

### Delete an RPath

To remove an existing RPath (`/unwanted/path`) from a binary:

```sh
arwen delete-rpath /unwanted/path my_binary
```

### Change Install Name

If a Mach-O binary depends on a shared library and you want to change the library install name:

```sh
arwen change-install-name /old/libname.dylib /new/libname.dylib my_binary
```

### Change Install ID

For a Mach-O shared library, changing its install ID:

```sh
arwen change-install-id /new/install/id.dylib my_library.dylib
```

## Resigning the Binary After Changes

On macOS, after modifying a binary, you need to re-sign it to ensure it runs properly. You can do this using `codesign`:

```sh
codesign --force --sign - my_binary
```

## Integration Tests

We have integration tests that validate that `arwen` maintains feature parity with `install_name_tool` to ensure correctness and reliability.

## License

`arwen` is licensed under the MIT license.

## Contributions

Contributions are welcome! Feel free to open issues or submit pull requests.



## Status

`arwen` is currently in active development and provides only `install_name_tool` implementation. `API` and `CLI` are subject to change to accommodate a more user-friendly experience.

## Funding

This [project](https://nlnet.nl/project/ELF-rusttools/) is funded through [NGI0 Entrust](https://nlnet.nl/entrust), a fund established by [NLnet](https://nlnet.nl) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu) program.

Learn more at the [NLnet project page](https://nlnet.nl/project/ELF-rusttools).

[<img src="https://nlnet.nl/logo/banner.png" alt="NLnet foundation logo" width="40%" />](https://nlnet.nl)

[<img src="https://nlnet.nl/image/logos/NGI0_tag.svg" alt="NGI Zero Logo" width="40%" />](https://nlnet.nl/entrust)
