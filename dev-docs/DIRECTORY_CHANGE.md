# Directory Structure Changed: .kitkat → .kitcat

## Summary

The repository directory has been changed from `.kitkat` to `.kitcat` throughout the entire codebase.

## What Changed

### Repository Directory
- **Old**: `.kitcat/` (all internal paths)
- **New**: `.kitcat/` (all internal paths)

This affects all internal paths:
- `.kitcat/` - Repository root directory
- `.kitcat/objects/` - Object storage
- `.kitcat/refs/` - Branch references
- `.kitcat/HEAD` - Current branch pointer
- `.kitcat/index` - Staging area
- `.kitcat/MERGE_HEAD` - Merge state
- `.kitcat/MERGE_MODE` - Merge mode tracking
- `.kitcat/config` - Repository configuration
- `.kitcat/hooks/` - Git hooks (future feature)
- `.kitcat/pack/` - Packfiles for large objects

### Files Modified

**All Rust source files** (`src/**/*.rs`):
- Every reference to `.kitkat` changed to `.kitcat`
- Total files affected: All modules in src/

**All documentation files**:
- README.md
- CONTRIBUTING.md
- RENAMING_SUMMARY.md
- NAMING_FIXES.md

## Migration for Existing Users

If you have existing repositories using the old `.kitkat` directory:

### Option 1: Rename Existing Repository (Recommended)
```bash
# In your project directory with a .kitkat folder
mv .kitkat .kitcat
```

This preserves all your history, branches, and commits.

### Option 2: Reinitialize
```bash
# Backup your files first!
rm -rf .kitkat
kitcat init
kitcat add .
kitcat commit -m "Reinitialize with .kitcat"
```

⚠️ **Warning**: Option 2 loses all history and branches!

## Verification

### Check Build Status
```bash
cargo build --release
# Result: ✅ Compiles successfully with 0 errors
```

### Check Source Code
```bash
grep -r "\.kitkat" src/ --include="*.rs"
# Result: ✅ No matches found (all changed to .kitcat)
```

### Check Documentation
```bash
grep -r "\.kitkat" *.md
# Result: ✅ No matches found (all changed to .kitcat)
```

## Testing After Change

### Basic Workflow Test
```bash
# Initialize new repository
kitcat init

# Verify .kitcat directory was created
ls -la .kitcat/
# Should show: HEAD, index, objects/, refs/

# Add and commit a file
echo "test" > test.txt
kitcat add test.txt
kitcat commit -m "Test commit with .kitcat"

# Verify commit was stored in .kitcat/objects/
ls .kitcat/objects/
```

### Branch Test
```bash
# Create and switch branches
kitcat branch feature
kitcat checkout feature

# Verify refs stored in .kitcat/refs/heads/
ls .kitcat/refs/heads/
# Should show: master, feature
```

## Implementation Details

### Change Method
```bash
# Command executed to change all source files:
find src -name "*.rs" -exec sed -i 's/\.kitkat/.kitcat/g' {} \;

# Command executed to change all documentation:
sed -i 's/\.kitkat/.kitcat/g' README.md CONTRIBUTING.md RENAMING_SUMMARY.md NAMING_FIXES.md
```

### Affected Code Patterns

**Repository Initialization** (src/commands/commands.rs):
```rust
// Before:
fs::create_dir_all(".kitkat/objects")?;
fs::create_dir_all(".kitkat/refs/heads")?;

// After:
fs::create_dir_all(".kitcat/objects")?;
fs::create_dir_all(".kitcat/refs/heads")?;
```

**Path Checks** (src/utils.rs):
```rust
// Before:
let kitcat_path = Path::new(".kitkat");

// After:
let kitcat_path = Path::new(".kitcat");
```

**Object Storage** (src/object/mod.rs):
```rust
// Before:
let object_dir = format!(".kitkat/objects/{}", &hash[..2]);

// After:
let object_dir = format!(".kitcat/objects/{}", &hash[..2]);
```

## Consistency Verification

✅ **Package name**: `kitcat`
✅ **Binary names**: `kitcat`, `kit-cat`, `kc`, `kit`
✅ **Repository directory**: `.kitcat`
✅ **Variable naming**: `kitcat_*` prefix
✅ **User messages**: All reference `kitcat` command
✅ **Documentation**: All updated to `.kitcat`

## Breaking Changes

### For Existing Users
- **BREAKING**: Existing repositories with `.kitkat` directories will need to be renamed to `.kitcat`
- **Migration**: Simple `mv .kitkat .kitcat` command preserves all data
- **No data loss**: The internal format of objects, commits, and index is unchanged

### For Developers
- **BREAKING**: All code referencing `.kitkat` paths will break
- **Fix**: Update all path references from `.kitkat` to `.kitcat`
- **Constants**: Update any hardcoded string literals

## Future Considerations

### Ignore File
When implementing ignore functionality:
- Use `.kitcatignore` (not `.kitkatignore`)
- Pattern: Consistent with `.kitcat` directory name

### Compatibility Check
Add version detection for old repositories:
```rust
// Future enhancement
if Path::new(".kitkat").exists() && !Path::new(".kitcat").exists() {
    eprintln!("Warning: Old .kitkat directory detected.");
    eprintln!("Please rename to .kitcat: mv .kitkat .kitcat");
    return Err(io::Error::new(
        io::ErrorKind::Other,
        "Incompatible repository format"
    ));
}
```

## Changelog Entry

### Version 0.2.0 (Upcoming)
**BREAKING CHANGE**: Repository directory changed from `.kitkat` to `.kitcat`

- Changed: All internal paths now use `.kitcat` instead of `.kitkat`
- Migration: Rename existing repositories with `mv .kitkat .kitcat`
- Reason: Consistency with package name `kitcat`
- Impact: Existing repositories require one-time directory rename

## Status

✅ **Complete**: All source code updated
✅ **Complete**: All documentation updated
✅ **Complete**: Build successful (0 errors)
✅ **Complete**: No remaining `.kitkat` references
🎉 **Ready**: Project fully migrated to `.kitcat` directory structure

---

**Change Date**: 2026-01-18
**Affected Version**: 0.2.0 (development)
**Status**: Complete ✅
