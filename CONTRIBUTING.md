# CONTRIBUTING

Contributions are welcome! Here's how you can help:

## Development Setup

```sh
# Clone the repository
git clone https://github.com/HairlessVillager/minecommit.git
cd minecommit

# Install Rust Nightly (required by simdnbt)
rustup toolchain install nightly

# Build the CLI
cargo build --release --bin minecommit

# Run tests
cargo test
```

For GUI development:

```sh
cd minecommit-gui
bun install        # or npm install
bun run tauri dev  # starts Vite dev server + Tauri window
```

## Project Structure

```text
minecommit/
├── minecommit/          # Core library (handlers, ODB, utilities)
│   └── src/
│       ├── handler/     # File-type handlers (ChunkRegion, Entities, POI, etc.)
│       ├── odb/         # Object database abstraction (Fs, Git backends)
│       └── utils/       # Shared utilities (NBT, region, git command helpers)
├── minecommit-cli/      # CLI binary (clap-based argument parsing)
├── minecommit-gui/      # Tauri + React GUI
│   ├── src/             # React frontend (pages, components)
│   └── src-tauri/       # Tauri Rust backend
```

## Commit Conventions

1. Big idea (> 100 lines) should be reviewed in an issue before PR
2. Write useful commit message (you can follow https://chris.beams.io/git-commit)
3. Keep PRs focused on a single change
4. Run `cargo fmt` and `cargo clippy` before submitting

## Adding a New Handler

1. Create a new module under `minecommit/src/handler/`
2. Implement the `Handler` trait with `workspace()`, `flatten()`, and `unflatten()`
3. Register it in `CrafterImpl::get_crafters()` in `minecommit/src/handler/mod.rs`
4. Add the handler's dependency if needed to `minecommit/Cargo.toml`
