/// Represents an entry in the index
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Creation time (seconds since epoch)
    pub ctime_sec: u32,
    /// Creation time (nanoseconds)
    pub ctime_nsec: u32,
    /// Modification time (seconds since epoch)
    pub mtime_sec: u32,
    /// Modification time (nanoseconds)
    pub mtime_nsec: u32,
    /// Device ID
    pub dev: u32,
    /// Inode number
    pub ino: u32,
    /// File mode/permissions (e.g., 0o100644 for regular file, 0o100755 for executable)
    pub mode: u32,
    /// User ID
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// File size in bytes
    pub size: u32,
    /// SHA-1 hash of file content (40 hex characters)
    pub hash: String,
    /// Flags (includes name length and stage)
    pub flags: u16,
    /// File path relative to repository root
    pub path: String,
}

impl IndexEntry {
    /// Create a new index entry from file metadata
    pub fn from_file(path: String, hash: String, metadata: &std::fs::Metadata) -> Self {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        use std::time::SystemTime;

        let ctime = metadata.ctime();
        let ctime_nsec = metadata.ctime_nsec();
        let mtime = metadata.mtime();
        let mtime_nsec = metadata.mtime_nsec();

        // Determine file mode
        let mode = if metadata.is_file() {
            if metadata.permissions().mode() & 0o111 != 0 {
                0o100755 // Executable file
            } else {
                0o100644 // Regular file
            }
        } else if metadata.is_symlink() {
            0o120000 // Symbolic link
        } else {
            0o100644 // Default
        };

        // Calculate flags (lower 12 bits = name length, upper 4 bits = stage)
        let name_len = std::cmp::min(path.len(), 0xFFF) as u16;
        let flags = name_len; // Stage 0 (normal entry)

        Self {
            ctime_sec: ctime as u32,
            ctime_nsec: ctime_nsec as u32,
            mtime_sec: mtime as u32,
            mtime_nsec: mtime_nsec as u32,
            dev: metadata.dev() as u32,
            ino: metadata.ino() as u32,
            mode,
            uid: metadata.uid(),
            gid: metadata.gid(),
            size: metadata.len() as u32,
            hash,
            flags,
            path,
        }
    }

    /// Get the stage of this entry (0 = normal, 1 = base, 2 = ours, 3 = theirs)
    pub fn stage(&self) -> u16 {
        (self.flags >> 12) & 0x3
    }
}

/// Tree entry (for future tree implementation)
#[derive(Debug, Clone)]
pub struct TreeEntry {
    /// File mode (e.g., "100644", "100755", "040000" for directory)
    pub mode: String,
    /// Entry name (filename or directory name)
    pub name: String,
    /// SHA-1 hash of the object (blob or tree)
    pub hash: [u8; 20],
    /// Whether this entry is a tree (directory)
    pub is_tree: bool,
}

/// Commit object (for future commit implementation)
#[derive(Debug, Clone)]
pub struct Commit {
    /// SHA-1 hash of the tree object
    pub tree: String,
    /// Parent commit hashes (empty for initial commit, multiple for merge commits)
    pub parents: Vec<String>,
    /// Author name and email
    pub author: String,
    /// Author timestamp
    pub author_time: i64,
    /// Committer name and email
    pub committer: String,
    /// Committer timestamp
    pub committer_time: i64,
    /// Commit message
    pub message: String,
}
