# Warning
### This is a toy project for educational purposes only. It is not intended for production use.

# How to use

## Build
```bash
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
- [ ] Add command for listing commits
- [ ] Add .ignore file support
- [ ] Add flag to view command for going back to latest commit
- [ ] Branching & Merging

