# GitHub Actions CI/CD Guide for KitCat

## 🎯 Overview

Your project now has comprehensive CI/CD with:
- ✅ **Automated testing** on every push/PR
- ✅ **Multi-platform builds** (Linux, macOS, Windows)
- ✅ **Professional installers** (DEB, MSI, tarballs, zips)
- ✅ **Automatic releases** triggered by git tags
- ✅ **Cargo.io publishing** for stable releases
- ✅ **Security audits** and code coverage

## 📁 Workflows Created

### 1. `.github/workflows/ci.yml` - Continuous Integration

**Triggers**: Every push/PR to master/main/develop branches

**Jobs**:
- **Test on all platforms**: Ubuntu, Windows, macOS
- **Code formatting**: `cargo fmt --check`
- **Linting**: `cargo clippy`
- **Build**: Debug and release builds
- **Tests**: Run test suite
- **Coverage**: Generate code coverage reports
- **Security**: Audit dependencies for vulnerabilities

**Status**: Runs on every commit to ensure code quality

### 2. `.github/workflows/release.yml` - Automated Releases

**Triggers**: Git tags matching `v*.*.*` (e.g., `v0.2.0`, `v0.2.0-beta.1`)

**Jobs**:

1. **create-release**: Creates GitHub release from tag
2. **build-linux**: Builds DEB package + tarball
3. **build-macos**: Builds for Intel & Apple Silicon
4. **build-windows**: Builds MSI installer + zip
5. **publish-crates-io**: Publishes stable versions to crates.io

## 🚀 How to Create a Release

### Step 1: Prepare Your Code

```bash
# Make sure everything is committed
git status

# Update version in Cargo.toml if needed
# Update CHANGELOG.md (create if doesn't exist)

# Commit changes
git add -A
git commit -m "chore: prepare for v0.2.0-beta.1 release"
```

### Step 2: Create Annotated Tag

The tag message becomes your release notes!

```bash
# Create annotated tag with release notes
git tag -a v0.2.0-beta.1 -m "KitCat v0.2.0-beta.1 - Complete Rename

## What's New
- Renamed project from kitkat to kitcat
- Added 4 binary aliases: kitcat, kit-cat, kc, kit
- Changed repository directory: .kitkat → .kitcat
- Created comprehensive CONTRIBUTING.md

## Breaking Changes
- Package renamed: kitkat → kitcat
- Binary aliases: kk → kc, kit-kat → kit-cat
- Repository directory: .kitkat → .kitcat

## Installation

### Via Cargo
\`\`\`bash
cargo install kitcat --version 0.2.0-beta.1
\`\`\`

### Via DEB (Ubuntu/Debian)
\`\`\`bash
wget https://github.com/itsSauraj/kit-kat/releases/download/v0.2.0-beta.1/kitcat_0.2.0-beta.1_amd64.deb
sudo dpkg -i kitcat_0.2.0-beta.1_amd64.deb
\`\`\`

### Via MSI (Windows)
Download and run the MSI installer from the releases page.

## Migration
Rename existing repositories:
\`\`\`bash
mv .kitkat .kitcat
\`\`\`

## Contributors
- Saurabh Kumar (@itsSauraj)
- Co-Authored-By: Claude Sonnet 4.5"

# View the tag
git show v0.2.0-beta.1
```

### Step 3: Push Tag to GitHub

```bash
# Push the tag (this triggers the release workflow!)
git push origin v0.2.0-beta.1

# Or push all tags
git push origin --tags
```

### Step 4: Watch the Magic! ✨

Visit: https://github.com/itsSauraj/kit-kat/actions

The workflow will:
1. ⏳ Create GitHub release (1 minute)
2. ⏳ Build Linux artifacts (3-5 minutes)
3. ⏳ Build macOS artifacts (5-7 minutes)
4. ⏳ Build Windows artifacts (5-7 minutes)
5. ⏳ Publish to crates.io (if stable) (1 minute)

**Total time**: ~10-15 minutes for all platforms

## 📦 What Gets Built?

### Linux
- **DEB package**: `kitcat_0.2.0-beta.1_amd64.deb`
  - Installs to `/usr/bin/`
  - Includes all 4 aliases
  - Easy: `sudo dpkg -i kitcat_*.deb`

- **Tarball**: `kitcat-0.2.0-beta.1-linux-x86_64.tar.gz`
  - Contains all 4 binaries
  - For non-Debian systems (Arch, Fedora, Alpine)
  - Extract and copy to PATH

- **Checksums**: `SHA256SUMS-linux`

### macOS
- **Intel tarball**: `kitcat-0.2.0-beta.1-macos-x86_64.tar.gz`
- **Apple Silicon tarball**: `kitcat-0.2.0-beta.1-macos-aarch64.tar.gz`
- Both include all 4 binaries
- **Checksums**: `SHA256SUMS-macos-{arch}`

### Windows
- **MSI installer**: `kitcat-0.2.0-beta.1-windows-x86_64.msi`
  - Double-click to install
  - Automatically adds to PATH
  - Includes all 4 aliases (.exe files)
  - Configurable install directory
  - Uninstaller via Control Panel

- **Zip archive**: `kitcat-0.2.0-beta.1-windows-x86_64.zip`
  - Portable version
  - Extract and run
  - No installation needed

- **Checksums**: `SHA256SUMS-windows`

## 🔐 Required Secrets

Set these in GitHub: `Settings → Secrets and variables → Actions`

### 1. CARGO_TOKEN (Required for crates.io)

```bash
# Get token from https://crates.io/settings/tokens
# Click "New Token"
# Copy token and add to GitHub secrets as: CARGO_TOKEN
```

### 2. GITHUB_TOKEN (Automatic)

No setup needed - GitHub automatically provides this.

## 🔄 Release Workflow Details

### Automatic Release Detection

- **Stable**: `v1.0.0`, `v1.2.3` → Published to crates.io
- **Pre-release**: `v0.2.0-beta.1`, `v1.0.0-rc.1` → Marked as pre-release, NOT published to crates.io
- **Alpha/Beta/RC**: Automatically detected as pre-release

### MSI Installer Features

The Windows MSI installer includes:
- ✅ GUI installation wizard
- ✅ Choose installation directory
- ✅ Automatic PATH configuration
- ✅ Install all 4 aliases (kitcat, kit-cat, kc, kit)
- ✅ Uninstaller in Control Panel
- ✅ Upgrades (installs over old version)
- ✅ Silent install mode: `msiexec /i kitcat.msi /quiet`

### DEB Package Features

The Debian package includes:
- ✅ System integration (`dpkg`)
- ✅ Installs to `/usr/bin/`
- ✅ All 4 binary aliases
- ✅ Documentation in `/usr/share/doc/kitcat/`
- ✅ Easy removal: `sudo apt remove kitcat`

## 📝 Tag Message Best Practices

Your tag message becomes the release description. Structure it like:

```
Title: Brief summary

## What's New
- Feature 1
- Feature 2

## Breaking Changes
- Change 1
- Change 2

## Installation

### Via Cargo
```bash
cargo install kitcat
```

### Via Package Managers
[Platform-specific instructions]

## Migration
[Migration steps if needed]

## Contributors
- Names and credits
```

## 🐛 Troubleshooting

### Release workflow fails

**Check**:
1. Is CARGO_TOKEN set in GitHub secrets?
2. Does Cargo.toml have correct metadata?
3. Check Actions logs: https://github.com/itsSauraj/kit-kat/actions

### MSI creation fails

**Reason**: WiX toolset issues on Windows runner
**Solution**: The workflow has fallback - will still create ZIP

### crates.io publish fails

**Possible reasons**:
1. Version already published
2. Cargo.toml validation failed
3. Token expired or invalid
4. It's a pre-release (alpha/beta/rc) - these are skipped intentionally

### Tag already exists

```bash
# Delete local tag
git tag -d v0.2.0-beta.1

# Delete remote tag
git push origin :refs/tags/v0.2.0-beta.1

# Create new tag
git tag -a v0.2.0-beta.1 -m "New message"
git push origin v0.2.0-beta.1
```

## 📊 Monitoring Releases

### During Release

Watch progress: https://github.com/itsSauraj/kit-kat/actions

Each job shows:
- ✅ Green: Success
- ❌ Red: Failed
- 🟡 Yellow: In progress
- ⚪ Gray: Pending

### After Release

Check release page: https://github.com/itsSauraj/kit-kat/releases

Should show:
- Release notes (from tag message)
- All platform artifacts
- Download counts
- Pre-release badge (if applicable)

## 🎯 Release Checklist

Before creating a tag:

- [ ] Version updated in Cargo.toml
- [ ] CHANGELOG.md updated (create if doesn't exist)
- [ ] All tests passing locally: `cargo test`
- [ ] Code formatted: `cargo fmt`
- [ ] Lints clean: `cargo clippy`
- [ ] Build successful: `cargo build --release`
- [ ] All changes committed
- [ ] CARGO_TOKEN set in GitHub secrets (for stable releases)

Then:

- [ ] Create annotated tag with detailed message
- [ ] Push tag to GitHub
- [ ] Watch Actions workflow
- [ ] Verify release artifacts uploaded
- [ ] Test installation on each platform
- [ ] Announce release (Twitter, Reddit, Discord, etc.)

## 🔮 Future Enhancements

Consider adding:
- [ ] Homebrew formula auto-update
- [ ] Chocolatey package submission
- [ ] Scoop manifest creation
- [ ] AUR (Arch User Repository) package
- [ ] Snap package
- [ ] AppImage for Linux
- [ ] Docker image publishing
- [ ] Release announcement bot

## 📚 Example Release Commands

### Beta Release
```bash
git tag -a v0.2.0-beta.2 -m "Beta 2 with bug fixes"
git push origin v0.2.0-beta.2
```

### Release Candidate
```bash
git tag -a v0.2.0-rc.1 -m "Release Candidate 1"
git push origin v0.2.0-rc.1
```

### Stable Release
```bash
git tag -a v0.2.0 -m "Stable release v0.2.0"
git push origin v0.2.0
# This will also publish to crates.io!
```

### Patch Release
```bash
git tag -a v0.2.1 -m "Patch release with critical bugfix"
git push origin v0.2.1
```

## 🎉 Success Indicators

After successful release:

1. ✅ GitHub release created with all artifacts
2. ✅ 8+ files uploaded (Linux, macOS, Windows + checksums)
3. ✅ crates.io shows new version (for stable)
4. ✅ Download badges updated
5. ✅ Users can install via all methods

## 💬 Questions?

- **Issues**: https://github.com/itsSauraj/kit-kat/issues
- **Actions logs**: https://github.com/itsSauraj/kit-kat/actions
- **Releases**: https://github.com/itsSauraj/kit-kat/releases

---

**Ready to release?** Follow the steps above and push that tag! 🚀
