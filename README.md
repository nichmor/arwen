[![Pixi Badge][pixi-badge]][pixi-url]

[pixi-badge]:https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/prefix-dev/pixi/main/assets/badge/v0.json&style=flat-square
[pixi-url]: https://pixi.sh

# arwen: patching Macho and ELF binaries in rust.

## Overview

`arwen` is a cross-platform Rust implementation of  `"patchelf"` (Linux) and `"install_name_tool"` (macOS) in one tool.

## Usage

The cli looks like this:

```shell
Usage: arwen <COMMAND>

Commands:
  remove               Remove a run path
  change-rpath         Change already existing run path
  add-rpath            Add a run path
  change-install-name  Change existing dylib load name
  change-install-id    Change dylib id. Works only if your object is a shared library
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```



## Status

`arwen` is currently in active development and provides only `install_name_tool` implementation. `API` and `CLI` are subject to change to accommodate a more user-friendly experience.
