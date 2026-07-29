# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
# Build all targets
cargo build --release

# Run all tests (lib + doctests)
cargo test --all-targets

# Run a single test
cargo test -- exchange_rs_with_preserve_ext_false_swaps_full_names

# Type-check all targets without compiling
cargo check --all-targets

# Verify packaging
cargo package

# Windows: build both x86 and x64 static libs (MSVC only)
./build.bat
# Requires: rustup target add i686-pc-windows-msvc
```

## Architecture

This is a cross-platform Rust library that atomically swaps names of two files or directories, exposed via both a Rust API and a C FFI interface. It builds three crate types: `rlib`, `cdylib`, and `staticlib`.

### Module structure (`src/`)

| File               | Role                                                                                                                                                                  |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lib.rs`           | Entry point. C FFI `exchange()` function, Rust `exchange_rs()` API, C-string input sanitization.                                                                      |
| `types.rs`         | Data types (`NameExchange`, `FileInfos`, `MetadataCollection`, `PrepareName`, `GetPathInfo`) and the `RenameError` enum with C-compatible error codes (0–5, 255).     |
| `exchange.rs`      | Core orchestration: path resolution, validation, conflict detection, and dispatching the correct rename strategy based on file-vs-directory and nesting relationship. |
| `path_checkout.rs` | Path analysis: file/dir detection, parent-child nesting detection (`if_root` returns 0/1/2), metadata extraction (stem, extension, parent dir).                       |
| `file_rename.rs`   | Rename execution: 3-step temp-file swap algorithm with rollback on failure, safe handling of nested (parent-child) renames.                                           |

### Swap algorithm

The core challenge is atomically swapping two names. The library handles two cases:

1. **No nesting** (paths are independent): 3-step swap using a temp file — `B → temp`, `A → B_final`, `temp → A_final`. Each step can roll back prior steps on failure.

2. **Nesting** (one path is a parent of the other): Direct sequential rename without temp files, since moving a parent would break the child's path. Order is determined by `if_root()` (returns 1 = path1 contains path2, 2 = path2 contains path1, 0 = no nesting).

### Extension handling

When `preserve_ext=true`: each file keeps its own extension (stems are swapped only).
When `preserve_ext=false`: extensions are swapped along with the full name.

### Platform specifics

- **Windows**: MSVC toolchain required (build.rs panics on GNU). Backslash path normalization. `USERPROFILE` env var for `~` expansion. Windows resource file (`res.rc`) embeds version metadata into the DLL.
- **Unix**: Forward-slash normalization. `HOME` env var for `~` expansion.
- `resolve_path()` handles relative paths, `~`, `.`, and `canonicalize()` for symlink/real-path resolution across both platforms.

### Key details

- The `GUID` constant in `types.rs` (`5E702FA07C2FB332B76B`) is the base for temp filenames to avoid collisions.
- `DEBUG_MODE` (derived from `cfg!(debug_assertions)`) enables verbose path-resolution logging.
- `ResolveError` implements `From<io::Error>` to map OS errors to the library's error enum.
- C FFI return codes: 0=success, 1=not found, 2=permission denied, 3=already exists, 4=same path, 5=invalid path, 255=unknown.
