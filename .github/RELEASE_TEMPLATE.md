# Release Template

Use this template when creating release tags:

```bash
git tag -a vX.Y.Z -m "$(cat <<'EOF'
KitCat vX.Y.Z - Release Title

## 🎉 Highlights
- Major feature 1
- Major feature 2
- Major feature 3

## ✨ What's New
- New feature A
- New feature B
- Enhancement C

## 🐛 Bug Fixes
- Fixed issue #123
- Fixed crash when...
- Resolved memory leak in...

## 💥 Breaking Changes
- BREAKING: Changed API X to Y
- BREAKING: Removed deprecated feature Z

## 🔧 Installation

### Via Cargo
\`\`\`bash
cargo install kitcat
\`\`\`

### Via DEB Package (Debian/Ubuntu)
\`\`\`bash
wget https://github.com/itsSauraj/kit-kat/releases/download/vX.Y.Z/kitcat_X.Y.Z_amd64.deb
sudo dpkg -i kitcat_X.Y.Z_amd64.deb
\`\`\`

### Via MSI Installer (Windows)
Download `kitcat-X.Y.Z-windows-x86_64.msi` and run the installer.

### Via Tarball (Linux/macOS)
\`\`\`bash
# Linux
wget https://github.com/itsSauraj/kit-kat/releases/download/vX.Y.Z/kitcat-X.Y.Z-linux-x86_64.tar.gz
tar xzf kitcat-X.Y.Z-linux-x86_64.tar.gz
sudo mv kitcat-X.Y.Z-linux-x86_64/* /usr/local/bin/

# macOS (Intel)
wget https://github.com/itsSauraj/kit-kat/releases/download/vX.Y.Z/kitcat-X.Y.Z-macos-x86_64.tar.gz

# macOS (Apple Silicon)
wget https://github.com/itsSauraj/kit-kat/releases/download/vX.Y.Z/kitcat-X.Y.Z-macos-aarch64.tar.gz
\`\`\`

## 📦 Binary Aliases
All packages include 4 command aliases:
- \`kitcat\` - Full name
- \`kit-cat\` - Hyphenated variant
- \`kc\` - Quick shorthand
- \`kit\` - Shortest form

## 🔄 Migration Guide
[Add migration steps if this is a breaking change release]

### For Users Upgrading from vX.Y.Z-1
\`\`\`bash
# Step 1: Backup your data
# Step 2: ...
\`\`\`

## 📝 Changelog
See [CHANGELOG.md](https://github.com/itsSauraj/kit-kat/blob/master/CHANGELOG.md) for complete history.

## 🙏 Contributors
- @username1 - Contributed feature X
- @username2 - Fixed bug Y
- @username3 - Improved documentation

Special thanks to all contributors and testers!

## 📊 Statistics
- X commits since last release
- Y files changed
- Z issues closed

## 🔗 Links
- **Documentation**: https://github.com/itsSauraj/kit-kat#readme
- **Report Issues**: https://github.com/itsSauraj/kit-kat/issues
- **Discussions**: https://github.com/itsSauraj/kit-kat/discussions

---

**Full Changelog**: https://github.com/itsSauraj/kit-kat/compare/vX.Y.Z-1...vX.Y.Z

Built with 🦀 Rust
EOF
)"

# Push tag
git push origin vX.Y.Z
```

## Version Number Guidelines

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes
  - Changed CLI interface
  - Removed features
  - Incompatible data formats

- **MINOR** (0.Y.0): New features, backward-compatible
  - New commands
  - New options
  - Enhancements

- **PATCH** (0.0.Z): Bug fixes, backward-compatible
  - Bug fixes
  - Security patches
  - Minor improvements

### Pre-release suffixes:
- `v0.2.0-alpha.1` - Early testing, unstable
- `v0.2.0-beta.1` - Feature complete, needs testing
- `v0.2.0-rc.1` - Release candidate, final testing
- `v0.2.0` - Stable release

## Quick Commands

```bash
# Beta release
git tag -a v0.2.0-beta.2 -m "Beta 2: Bug fixes and improvements"

# Stable release
git tag -a v0.2.0 -m "Stable release v0.2.0"

# Patch release
git tag -a v0.2.1 -m "Patch: Critical security fix"

# Always push tags
git push origin --tags
```
