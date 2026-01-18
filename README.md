# KitKat VCS

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.rst)
[![Status](https://img.shields.io/badge/status-alpha-yellow.svg)]()

**KitKat** is a Git-like version control system implemented in Rust, designed to be simple, fast, and Git-compatible where possible. It's an educational project demonstrating how version control systems work under the hood.

## ✨ Features

### ✅ Currently Implemented

- **Repository Management**
  - Initialize repositories (`.kitkat` directory structure)
  - Configuration management (user.name, user.email)

- **Object Storage**
  - Content-addressable blob storage with SHA-1 hashing
  - Zlib compression for efficient storage
  - Tree objects for directory snapshots
  - Commit objects with full metadata (author, timestamp, parents)

- **Staging Area**
  - Binary DIRC format index (Git-compatible)
  - File metadata tracking (permissions, timestamps)
  - Atomic writes with file locking

- **Version Control**
  - Create commits with messages
  - View commit history (`log`, `log --oneline`)
  - Branch management (create, list, delete, switch)
  - Checkout (branch switching, detached HEAD, file restoration)
  - Working tree status (staged, unstaged, untracked files)
  - Diff (compare working tree, index, and commits)
  - Merge (three-way merge with conflict detection and resolution)
  - Show commit details

- **CLI Interface**
  - Full command-line interface with help
  - Intuitive Git-like commands

- **Repository Optimization**
  - Garbage collection with object packing
  - Packfile format for efficient storage
  - Prune unreachable objects
  - Repository size optimization

### 📋 Planned

- Delta compression for packfiles
- LFS (Large File Storage) for very large files
- Remote repository support
- Network operations (push, pull, fetch)
- Advanced merge strategies
- Submodules

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/kitkat.git
cd kitkat

# Build in release mode
cargo build --release

# The binary is at target/release/kitkat
# Optionally, add to your PATH
```

See the [Installation Guide](docs/installation.md) for detailed instructions.

### Your First Repository

```bash
# Initialize a new repository
kitkat init

# Configure your identity
kitkat config user.name "Your Name"
kitkat config user.email "you@example.com"

# Create and add a file
echo "# My Project" > README.md
kitkat add README.md

# Create your first commit
kitkat commit -m "Initial commit"

# View history
kitkat log
```

See the [Quick Start Guide](docs/quick-start.md) for a complete tutorial.

## 📚 Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

- **Getting Started**
  - [Installation Guide](docs/installation.md)
  - [Quick Start (5 minutes)](docs/quick-start.md)
  - [Command Reference](docs/command-reference.md)

- **Guides**
  - [Testing Guide](docs/testing-guide.md) - How to test KitKat
  - [Architecture](docs/architecture.md) - System design and internals

## 🛠️ Development

### Prerequisites

- Rust 1.92.0 or later
- Cargo (comes with Rust)

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- init
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run manual tests
./test-kitkat.sh  # See docs/testing-guide.md
```

## 📖 Usage Examples

### Basic Workflow

```bash
# Initialize repository
kitkat init

# Configure identity
kitkat config user.name "Developer"
kitkat config user.email "dev@example.com"

# Add files
kitkat add file1.txt
kitkat add file2.txt

# Create commit
kitkat commit -m "Add initial files"

# View history
kitkat log --oneline
```

### Working with Branches

```bash
# Create a branch
kitkat branch feature-x

# List branches
kitkat branch

# Switch to branch
kitkat branch feature-x

# Make changes and commit
kitkat add newfile.txt
kitkat commit -m "Add feature"

# Switch back to master
kitkat branch master

# Delete branch
kitkat branch -d feature-x
```

### Merging Branches

```bash
# Create and switch to a feature branch
kitkat branch feature
kitkat checkout feature

# Make changes on feature branch
echo "new feature" > feature.txt
kitkat add feature.txt
kitkat commit -m "Add new feature"

# Switch back to master
kitkat checkout master

# Merge feature into master
kitkat merge feature

# If there are conflicts:
# - Edit files to resolve conflicts
# - Stage the resolved files
kitkat add resolved-file.txt
# - Continue the merge
kitkat merge --continue

# Or abort the merge
kitkat merge --abort
```

### Inspecting Objects

```bash
# Write tree from index
kitkat write-tree

# List tree contents
kitkat list-tree <tree-hash>

# Show commit details
kitkat show-commit <commit-hash>

# View commit history
kitkat log
kitkat log --oneline
kitkat log -n 5
```

## 🏗️ Architecture

KitKat uses a modular architecture:

```
┌─────────────────────────────────────┐
│           CLI Layer                 │
│      (Command Parsing)              │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│        Commands Layer               │
│   (Business Logic)                  │
└────┬───────┬────────┬───────────────┘
     │       │        │
┌────▼───┐ ┌─▼────┐ ┌▼────────┐
│ Index  │ │Object│ │  Repo   │
│ (Stage)│ │Storage│ │ (Config)│
└────────┘ └──────┘ └─────────┘
```

**Object Model**: Git-compatible
- Blobs: File content
- Trees: Directory snapshots
- Commits: Versioned snapshots with metadata

**Storage**: Content-addressable
- SHA-1 hashing for deduplication
- Zlib compression
- Split directories for performance

See [Architecture Documentation](docs/architecture.md) for details.

## 🎯 Project Goals

1. **Educational**: Demonstrate how VCS internals work
2. **Git-Compatible**: Use same object formats as Git
3. **Performance**: Leverage Rust's zero-cost abstractions
4. **Simplicity**: Clean, readable codebase
5. **Extensible**: Easy to add new features

## 📊 Project Status

**Version**: 0.1.0 (Alpha)
**Rust Edition**: 2024

### Feature Completeness

| Feature | Status |
|---------|--------|
| Repository initialization | ✅ Complete |
| Binary index (staging) | ✅ Complete |
| Blob objects | ✅ Complete |
| Tree objects | ✅ Complete |
| Commit objects | ✅ Complete |
| Branch management | ✅ Complete |
| Commit history (log) | ✅ Complete |
| Configuration | ✅ Complete |
| Status command | ✅ Complete |
| Checkout | ✅ Complete |
| Diff | ✅ Complete |
| Merge | ✅ Complete |
| Garbage collection | ✅ Complete |
| Packfiles | ✅ Complete |

**Phase 1 Progress**: 11/11 features complete (100%) ✨

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) (coming soon) for details on our code of conduct and the process for submitting pull requests.

### Areas Where We Need Help

- Large file support (packfiles, delta compression, LFS)
- Test coverage expansion
- Performance optimization
- Documentation improvements
- Remote repository support

## 📜 License

This project is licensed under the MIT License - see the [LICENSE.rst](LICENSE.rst) file for details.

## 🙏 Acknowledgments

- Inspired by [Git](https://git-scm.com/) and its excellent internals documentation
- Built with [Rust](https://www.rust-lang.org/) 🦀
- Uses [clap](https://docs.rs/clap/) for CLI parsing
- Compression with [flate2](https://docs.rs/flate2/)

## 📞 Support & Contact

- **Issues**: [GitHub Issues](https://github.com/yourusername/kitkat/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/kitkat/discussions)
- **Documentation**: [docs/](docs/)

## 🔗 Related Projects

- [Git](https://git-scm.com/) - The inspiration
- [libgit2](https://libgit2.org/) - Git implementation library
- [gitoxide](https://github.com/Byron/gitoxide) - Git implementation in Rust
- [jujutsu](https://github.com/martinvonz/jj) - Version control tool in Rust

---

**Built with ❤️ and Rust** 🦀

*KitKat: Because version control should be easy to understand!*
