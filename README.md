# KitKat

KitKat is a minimal Git-like VCS implemented in Rust. It stores content-addressed objects (blobs) compressed with zlib, a lightweight index, and basic HEAD/refs layout under `.kitkat`. The goal is to evolve toward tree structures, multi-file staging, snapshot/diff storage, and comparisons between working tree, index, and committed history.

## Features (current)
- Initialize repository: [`repo::init_repo`](src/repo/mod.rs)
- Blob creation and storage: [`object::hash_object`](src/object/hash_object.rs)
- Read objects with hash-prefix matching: [`object::read_object`](src/object/read_object.rs)
- Basic index reading: [`index::read_index`](src/index/read_index.rs)
- CLI commands: [`commands`](src/commands/commands.rs)

## Roadmap
- Trees and commits
- Persistent index with multiple entries
- Snapshot/delta storage
- Diffing between working tree, index, and commits

## Setup

### Prerequisites
- Rust (stable) and Cargo

### Build
```sh
cargo build
```

### Usage
Initialize a repo:
```sh
kitkat init
```

Create blob from a file:
```sh
kitkat hash-object app/testfile.txt
```

Read an object (prefix supported):
```sh
kitkat read-file -p -s <hash_or_prefix>
```

Add to index (currently prints added path and hash):
```sh
kitkat add app/testfile.txt
```

Read index:
```sh
kitkat read-index
```

HEAD operations:
```sh
kitkat write-head "ref: refs/heads/master"
kitkat read-head
```

## Project Structure
- CLI: [src/main.rs](src/main.rs)
- Commands: [src/commands/commands.rs](src/commands/commands.rs)
- Repo: [src/repo/mod.rs](src/repo/mod.rs)
- Objects: [src/object](src/object)
- Index: [src/index](src/index)
- Models: [src/models/mod.rs](src/models/mod.rs)
- Utils: [src/utils.rs](src/utils.rs)

## License
This project is licensed under the MIT License. See [LICENSE.rst](LICENSE.rst).

## Code of Conduct
Please read our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).