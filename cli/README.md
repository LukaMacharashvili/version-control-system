# Warning
### This is a toy project for educational purposes only. It is not intended for production use.

# How to use

## Build
```bash
cd path/to/this/project
cargo build
```

## Initialize a repository
```bash
cd path/to/your/project
cargo run --manifest-path path/to/this/project/Cargo.toml init
```

## Commit
```bash
cargo run --manifest-path path/to/this/project/Cargo.toml commit "Your commit message"
```

## View previous commits
```bash
cargo run --manifest-path path/to/this/project/Cargo.toml view "commit_id"
```

## TODO
- [ ] Add support for remote repositories with AWS S3
- [x] Add command for listing commits
- [x] Add .ignore file support
- [ ] Branching & Merging

