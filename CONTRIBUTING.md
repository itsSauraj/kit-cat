# Contributing to KitCat VCS

Thank you for your interest in contributing to KitCat! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inspiring community for all. Please be respectful and constructive in your interactions.

### Our Standards

**Examples of behavior that contributes to a positive environment:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Examples of unacceptable behavior:**
- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate

## Getting Started

### Prerequisites

- **Rust**: 1.70 or later (edition 2024)
- **Git**: For version control
- **Cargo**: Comes with Rust installation

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/kitcat.git
cd kitcat

# Build the project
cargo build

# Run tests
cargo test

# Install locally
cargo install --path .
```

After installation, you can use any of these commands:
- `kitcat` - Full name
- `kc` - Short alias
- `kit-cat` - Hyphenated variant
- `kit` - Even shorter alias

## Development Setup

### Build Commands

```bash
# Development build (faster, with debug info)
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run -- init

# Format code
cargo fmt

# Check for common mistakes
cargo clippy -- -D warnings

# Run tests with output
cargo test -- --nocapture
```

### Editor Setup

**VS Code**:
- Install [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- Install [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) for debugging

**IntelliJ/CLion**:
- Install [Rust plugin](https://www.jetbrains.com/rust/)

## Project Structure

```
kitcat/
├── src/
│   ├── commands/       # Command implementations (init, add, commit, etc.)
│   ├── config/         # Configuration management
│   ├── diff/           # Diff algorithms (Myers diff)
│   ├── index/          # Staging area (binary DIRC format)
│   ├── merge/          # Three-way merge implementation
│   ├── models/         # Data structures (IndexEntry, Commit, etc.)
│   ├── object/         # Object storage (blob, tree, commit, pack)
│   ├── repo/           # Repository utilities
│   ├── utils.rs        # Utility functions
│   └── main.rs         # CLI entry point
├── dev-docs/           # Development documentation
├── docs/               # User-facing documentation (planned)
├── tests/              # Integration tests (planned)
├── Cargo.toml          # Package configuration
└── README.md           # Main documentation
```

### Key Modules

- **commands/**: Each VCS command (add, commit, merge, etc.) has its own module
- **object/**: Object storage layer (SHA-1 content-addressable)
- **index/**: Git-compatible binary index (DIRC format)
- **merge/**: Three-way merge with conflict detection
- **diff/**: Myers diff algorithm implementation

## How to Contribute

### Types of Contributions

1. **Bug Fixes**: Fix identified issues
2. **New Features**: Implement planned features from roadmap
3. **Documentation**: Improve docs, add examples, write tutorials
4. **Tests**: Add unit tests, integration tests, or test fixtures
5. **Performance**: Optimize existing code
6. **Code Quality**: Refactoring, cleanup, better error messages

### Finding Issues to Work On

- Look for issues labeled `good first issue` for beginner-friendly tasks
- Issues labeled `help wanted` are looking for contributors
- Check the [PROJECT_SUMMARY.md](dev-docs/PROJECT_SUMMARY.md) for roadmap items

## Coding Standards

### Rust Style Guide

We follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/):

```rust
// Good: Use descriptive names
fn parse_commit_object(hash: &str) -> io::Result<Commit> {
    // ...
}

// Bad: Cryptic abbreviations
fn pc_obj(h: &str) -> io::Result<Commit> {
    // ...
}
```

### Best Practices

1. **Error Handling**: Use `io::Result<T>` for operations that can fail
   ```rust
   // Good
   fn read_file(path: &str) -> io::Result<Vec<u8>> {
       fs::read(path)
   }

   // Bad
   fn read_file(path: &str) -> Vec<u8> {
       fs::read(path).unwrap()  // Don't panic!
   }
   ```

2. **Documentation**: Document public APIs
   ```rust
   /// Parse a commit object from its hash
   ///
   /// # Arguments
   /// * `hash` - The 40-character SHA-1 hash
   ///
   /// # Returns
   /// A `Commit` struct with parsed metadata
   ///
   /// # Errors
   /// Returns `Err` if the object doesn't exist or is malformed
   pub fn parse_commit(hash: &str) -> io::Result<Commit> {
       // ...
   }
   ```

3. **Testing**: Write tests for new functionality
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_parse_commit_basic() {
           // Arrange
           let hash = "abc123...";

           // Act
           let result = parse_commit(hash);

           // Assert
           assert!(result.is_ok());
       }
   }
   ```

4. **No Unsafe Code**: Avoid `unsafe` unless absolutely necessary
5. **No Panics**: Use `Result` instead of `panic!` or `unwrap()`
6. **Minimal Dependencies**: Only add crates if really needed

### Code Formatting

```bash
# Format all code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with pedantic warnings
cargo clippy -- -W clippy::pedantic
```

## Testing Guidelines

### Writing Tests

1. **Unit Tests**: Test individual functions
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_hash_computation() {
           let data = b"hello world";
           let hash = compute_hash(data);
           assert_eq!(hash.len(), 40);
       }
   }
   ```

2. **Integration Tests**: Test complete workflows
   ```rust
   // tests/basic_workflow.rs
   #[test]
   fn test_init_add_commit() {
       let temp_dir = tempfile::tempdir().unwrap();
       // ... test full workflow
   }
   ```

3. **Property Tests**: Use proptest for fuzzing
   ```rust
   use proptest::prelude::*;

   proptest! {
       #[test]
       fn test_hash_stability(data: Vec<u8>) {
           let hash1 = compute_hash(&data);
           let hash2 = compute_hash(&data);
           prop_assert_eq!(hash1, hash2);
       }
   }
   ```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_hash_computation

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run tests with coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding missing tests
- `chore`: Maintenance tasks, dependency updates

### Examples

```bash
# Feature
feat(merge): implement three-way merge algorithm

Adds support for merging branches with automatic conflict detection.
The algorithm finds the merge base and performs file-level merging.

Closes #42

# Bug fix
fix(index): prevent race condition in concurrent writes

Use file locking to ensure atomic index updates when multiple
processes try to modify the staging area simultaneously.

# Documentation
docs(readme): add installation instructions for Windows

# Refactoring
refactor(object): extract pack file logic into separate module
```

## Pull Request Process

### Before Submitting

1. **Update from main**: Rebase on latest main branch
   ```bash
   git checkout main
   git pull origin main
   git checkout your-feature-branch
   git rebase main
   ```

2. **Run checks**:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```

3. **Update documentation**: If you changed APIs, update docs
4. **Add tests**: Ensure your changes are tested

### Submitting a PR

1. **Push your branch**:
   ```bash
   git push origin your-feature-branch
   ```

2. **Create Pull Request**: Go to GitHub and create a PR

3. **Fill out PR template**:
   ```markdown
   ## Description
   Brief description of what this PR does

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   How did you test this change?

   ## Checklist
   - [ ] Code follows project style guidelines
   - [ ] Self-review completed
   - [ ] Comments added for complex code
   - [ ] Documentation updated
   - [ ] Tests added/updated
   - [ ] All tests pass
   - [ ] No new warnings
   ```

4. **Address review comments**: Make requested changes promptly

5. **Keep PR focused**: One feature or fix per PR

### PR Review Process

- Maintainers will review your PR within 3-5 business days
- You may be asked to make changes
- Once approved, a maintainer will merge your PR
- Your contribution will be credited in release notes

## Reporting Bugs

### Before Reporting

1. **Check existing issues**: Search for similar bugs
2. **Verify it's reproducible**: Try to reproduce consistently
3. **Check latest version**: Ensure you're using the latest code

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Run command '...'
2. See error

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.70.0]
- KitCat version: [e.g., 0.1.0]

**Additional context**
Any other relevant information.

**Logs**
```
Paste relevant log output here
```
```

## Feature Requests

We welcome feature requests! Please provide:

1. **Use case**: Why do you need this feature?
2. **Description**: What should it do?
3. **Examples**: How would you use it?
4. **Alternatives**: What alternatives have you considered?

**Example**:
```markdown
**Feature**: Support for git hooks

**Use case**: I want to run tests automatically before commits

**Description**: Implement pre-commit, post-commit, and pre-push hooks

**Example usage**:
```bash
# Create a pre-commit hook
echo "cargo test" > .kitcat/hooks/pre-commit
chmod +x .kitcat/hooks/pre-commit
```

**Alternatives**: Could use external tools like husky, but native support would be better
```

## Documentation

### Types of Documentation

1. **Code Documentation**: Inline comments and doc comments
2. **User Documentation**: README, guides, tutorials
3. **Development Documentation**: Architecture docs in dev-docs/
4. **API Documentation**: Generated from doc comments

### Writing Documentation

```bash
# Generate API docs
cargo doc --open

# Check for documentation warnings
cargo doc --no-deps
```

### Documentation Style

- Use clear, simple language
- Include code examples
- Explain "why" not just "what"
- Keep it up to date

## Community

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Q&A, ideas, general discussion
- **Pull Requests**: Code review, technical discussions

### Getting Help

- Check the [README](README.md) first
- Search existing issues
- Ask in GitHub Discussions
- Tag maintainers if urgent

### Recognition

Contributors are recognized in:
- Release notes
- CONTRIBUTORS.md file (planned)
- Git commit history

## Development Roadmap

See [PROJECT_SUMMARY.md](dev-docs/PROJECT_SUMMARY.md) for the complete roadmap.

### Current Phase: Phase 1 Complete ✅

All core VCS features implemented!

### Next Phase: Phase 2 (Testing Infrastructure)

- Unit test coverage (target: 80%+)
- Integration test suite
- Test fixtures and mocks
- CI/CD setup

### Help Wanted

Priority areas for contribution:
- [ ] Test coverage expansion
- [ ] Documentation improvements
- [ ] Performance optimization
- [ ] Windows compatibility testing
- [ ] Example repositories

## License

By contributing to KitCat, you agree that your contributions will be licensed under the MIT License.

## Questions?

If you have questions not covered here, feel free to:
- Open an issue with the `question` label
- Start a discussion in GitHub Discussions
- Reach out to maintainers

---

**Thank you for contributing to KitCat! 🐱**

We appreciate your time and effort in making this project better.
