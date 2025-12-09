/// Represents an entry in the git index
pub struct IndexEntry {
    /// The path of the file
    pub path: String,
    /// The hash of the file content
    pub hash: String,
    /// The file mode (permissions)
    pub _mode: String,
}
