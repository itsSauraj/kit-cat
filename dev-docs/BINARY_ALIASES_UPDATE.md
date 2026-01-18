# Binary Aliases Updated

## Summary

The binary aliases have been updated to be more intuitive and consistent:

### Old Aliases ❌
- `kitcat`
- `kk`
- `kit-kat`
- `kit`

### New Aliases ✅
- `kitcat` - Full name (unchanged)
- `kit-cat` - Hyphenated variant (changed from `kit-kat`)
- `kc` - Quick shorthand (changed from `kk`)
- `kit` - Shortest form (unchanged)

## Rationale

The new aliases are more consistent with the package name "kitcat":
- **`kit-cat`**: Natural hyphenation of the name (was `kit-kat`)
- **`kc`**: Abbreviated from **K**it**C**at (was `kk` which didn't match the name)
- **`kit`**: Short and memorable (unchanged)

## Changes Made

### 1. Cargo.toml
Updated binary definitions:
```toml
[[bin]]
name = "kitcat"
path = "src/main.rs"

[[bin]]
name = "kit-cat"
path = "src/main.rs"

[[bin]]
name = "kc"
path = "src/main.rs"

[[bin]]
name = "kit"
path = "src/main.rs"
```

### 2. src/main.rs
Updated CLI help text:
```rust
#[command(long_about = "KitCat - A Git-like VCS written in Rust\n\nAliases: kitcat, kit-cat, kc, kit")]
```

### 3. Documentation
Updated all references in:
- README.md
- CONTRIBUTING.md
- RENAMING_SUMMARY.md
- NAMING_FIXES.md
- DIRECTORY_CHANGE.md

## Build Verification

```bash
$ cargo build --release
   Compiling kitcat v0.1.0
    Finished `release` profile [optimized] target(s) in 8.32s

$ ls -lh target/release/{kitcat,kit-cat,kc,kit}
-rwxrwxr-x 2.3M target/release/kc
-rwxrwxr-x 2.3M target/release/kit
-rwxrwxr-x 2.3M target/release/kit-cat
-rwxrwxr-x 2.3M target/release/kitcat
```

## Testing

All aliases work identically:

```bash
$ kitcat --version
kitcat 0.1.0

$ kit-cat --version
kitcat 0.1.0

$ kc --version
kitcat 0.1.0

$ kit --version
kitcat 0.1.0
```

## Help Text

```bash
$ kc --help
KitCat - A Git-like VCS written in Rust

Aliases: kitcat, kit-cat, kc, kit

Usage: kc <COMMAND>

Commands:
  init          Initialize a new repository
  hash-object   Compute object ID and optionally create a blob from a file
  read-file     Read and display an object file
  ...
```

## Installation

After building, all four binaries are available:

```bash
# Install from source
cargo install --path .

# All aliases will be available:
kitcat init
kit-cat add file.txt
kc commit -m "message"
kit status
```

## For Package Maintainers

When creating packages (DEB, RPM, MSI), ensure all four binaries are included:
- Primary binary: `kitcat`
- Aliases: `kit-cat`, `kc`, `kit`

All should be in the system PATH.

## Migration Guide

For users with existing installations:

```bash
# Uninstall old version
cargo uninstall kitcat

# Install new version with updated aliases
cargo install --path .

# Old aliases are replaced:
# kk → kc
# kit-kat → kit-cat
```

## Consistency Check

| Aspect | Name |
|--------|------|
| Package | `kitcat` |
| Primary Binary | `kitcat` |
| Hyphenated Alias | `kit-cat` |
| Short Alias | `kc` |
| Shortest Alias | `kit` |
| Repository Dir | `.kitcat` |
| Variables | `kitcat_*` prefix |

## Files Modified

1. **Cargo.toml** - Binary definitions
2. **src/main.rs** - CLI help text
3. **README.md** - Updated alias references
4. **CONTRIBUTING.md** - Installation instructions
5. **RENAMING_SUMMARY.md** - Complete rename documentation
6. **NAMING_FIXES.md** - Naming consistency document
7. **DIRECTORY_CHANGE.md** - Directory structure changes

## Status

✅ **Build**: Successful with 0 errors
✅ **Binaries**: All 4 aliases generated (2.3M each)
✅ **Documentation**: All files updated
✅ **Testing**: All aliases work correctly
✅ **Consistency**: Package name and aliases match

---

**Update Date**: 2026-01-18
**Status**: Complete ✅
