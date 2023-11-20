# Warning
### This is a toy project for educational purposes only. It is not intended for production use.

# How to use

## Build
```bash
cd path/to/this/project
cargo build
```

## Run help command for information on how to use the CLI
```bash
cd path/to/your/project
cargo run --manifest-path path/to/this/project/Cargo.toml help
```
```bash
A simple version control system written in Rust

Usage: cargo run --manifest-path path/to/this/project/Cargo.toml <command> [options]

Commands:
  init        Initializes a new repository
  commit      Commits the changes
  view        Views the commit
  commits     Views all the commits
  clone       Clones the remote repository
  set-remote  Sets the remote repository bucket
  pull        Pulls the changes
  push        Syncs the changes to the remote repository
  help        Print this message or the help of the given subcommand(s)
```

## TODO
- [x] Add API
- [ ] Add WASM support for browser (Github-like UI)
- [ ] Branching & Merging
- [x] Add support for remote repositories with AWS S3
- [x] Add command for listing commits
- [x] Add .ignore file support

