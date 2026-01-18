mod commands;
mod config;
mod diff;
mod index;
mod merge;
mod models;
mod object;
mod repo;
mod utils;

use clap::{Parser, Subcommand};
use commands::*;

/// Command line interface for KitCat VCS
#[derive(Parser)]
#[command(name = "kitcat")]
#[command(version = "0.2.0-beta.1")]
#[command(about = "A minimal Git-like version control system in Rust")]
#[command(long_about = "KitCat - A Git-like VCS written in Rust\n\nAliases: kitcat, kit-cat, kc, kit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available commands for KitCat VCS
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    Init,
    /// Compute object ID and optionally create a blob from a file
    HashObject {
        /// The file to hash
        file: String,
    },
    /// Read the content of an object
    ReadFile {
        /// Pretty-print the object content
        #[arg(short = 'p', long = "pretty")]
        pretty: bool,
        /// The hash of the object to read
        #[arg(short = 's', long = "hash")]
        hash: String,
    },
    /// Add file contents to the index
    Add {
        /// The file to add
        file: String,
    },
    /// Read the index
    ReadIndex,
    /// Write a value to HEAD
    WriteHead {
        /// The value to write
        value: String,
    },
    /// Read the current HEAD
    ReadHead,
    /// Write a tree from the current index
    WriteTree,
    /// List the contents of a tree
    ListTree {
        /// The hash of the tree to list
        hash: String,
    },
    /// Create a commit
    Commit {
        /// Commit message
        #[arg(short = 'm', long = "message")]
        message: String,
    },
    /// Show commit details
    ShowCommit {
        /// The hash of the commit to show
        hash: String,
    },
    /// Set or get a config value
    Config {
        /// Config key (e.g., user.name, user.email)
        key: String,
        /// Config value (if setting)
        value: Option<String>,
    },
    /// Branch operations
    Branch {
        /// Branch name (if creating or switching)
        name: Option<String>,
        /// Delete branch
        #[arg(short = 'd', long = "delete")]
        delete: bool,
        /// Force delete (ignore unmerged changes)
        #[arg(short = 'D', long = "force-delete")]
        force_delete: bool,
    },
    /// Show commit history
    Log {
        /// Show in oneline format
        #[arg(long = "oneline")]
        oneline: bool,
        /// Maximum number of commits to show
        #[arg(short = 'n', long = "max-count")]
        max_count: Option<usize>,
    },
    /// Show working tree status
    Status,
    /// Checkout a branch, commit, or restore files
    Checkout {
        /// Branch name, commit hash, or file path
        target: String,
        /// Force checkout even with uncommitted changes
        #[arg(short = 'f', long = "force")]
        force: bool,
        /// Restore file from index (use with -- before filename)
        #[arg(long = "file")]
        file: bool,
    },
    /// Show changes between commits, commit and working tree, etc
    Diff {
        /// Show staged changes (index vs HEAD)
        #[arg(long = "cached")]
        cached: bool,
        /// First commit to compare (optional)
        commit1: Option<String>,
        /// Second commit to compare (optional, requires commit1)
        commit2: Option<String>,
        /// Show statistics
        #[arg(long = "stat")]
        stat: bool,
        /// Disable color output
        #[arg(long = "no-color")]
        no_color: bool,
    },
    /// Join two or more development histories together
    Merge {
        /// Branch or commit to merge
        target: Option<String>,
        /// Abort the current merge
        #[arg(long = "abort")]
        abort: bool,
        /// Continue merge after resolving conflicts
        #[arg(long = "continue")]
        r#continue: bool,
        /// Create a merge commit even if fast-forward is possible
        #[arg(long = "no-ff")]
        no_ff: bool,
        /// Refuse to merge unless fast-forward is possible
        #[arg(long = "ff-only")]
        ff_only: bool,
        /// Commit message for merge commit
        #[arg(short = 'm', long = "message")]
        message: Option<String>,
    },
    /// Cleanup unnecessary files and optimize the local repository
    Gc {
        /// Run aggressive garbage collection
        #[arg(long = "aggressive")]
        aggressive: bool,
        /// Prune unreachable objects older than specified days
        #[arg(long = "prune")]
        prune_days: Option<u32>,
        /// Dry run - show what would be deleted without deleting
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::HashObject { file } => {
            let hash = hash_file(file);
            println!("{}", hash);
        }
        Commands::ReadFile { pretty, hash } => read_file(hash, pretty),
        Commands::Add { file } => add_to_index(file),
        Commands::ReadIndex => {
            let index = read_index();
            for entry in index {
                println!("{} {}", entry.hash, entry.path);
            }
        }
        Commands::WriteHead { value } => write_head(&value),
        Commands::ReadHead => {
            let head = read_head();
            println!("{}", head);
        }
        Commands::WriteTree => {
            write_tree();
        }
        Commands::ListTree { hash } => list_tree(hash),
        Commands::Commit { message } => commit(message),
        Commands::ShowCommit { hash } => show_commit_cmd(hash),
        Commands::Config { key, value } => {
            if let Some(val) = value {
                set_config_cmd(key, val);
            } else {
                get_config_cmd(key);
            }
        }
        Commands::Branch {
            name,
            delete,
            force_delete,
        } => {
            if delete || force_delete {
                // Delete branch
                if let Some(branch_name) = name {
                    if let Err(e) = delete_branch(&branch_name, force_delete) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: branch name required for deletion");
                    std::process::exit(1);
                }
            } else if let Some(branch_name) = name {
                // Create or switch branch
                // Try to switch first, if it exists
                if std::path::Path::new(&format!(".kitcat/refs/heads/{}", branch_name)).exists() {
                    if let Err(e) = switch_branch(&branch_name) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                } else {
                    // Create new branch
                    if let Err(e) = create_branch(&branch_name) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // List branches
                if let Err(e) = list_branches() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Log { oneline, max_count } => {
            let format = if oneline {
                LogFormat::Oneline
            } else {
                LogFormat::Full
            };

            if let Err(e) = log(format, max_count) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Status => {
            if let Err(e) = status() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Checkout {
            target,
            force,
            file,
        } => {
            if file {
                // Restore file from index
                if let Err(e) = checkout_file(&target) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            } else {
                // Checkout branch or commit
                if let Err(e) = checkout(&target, force) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Diff {
            cached,
            commit1,
            commit2,
            stat,
            no_color,
        } => {
            let mode = if cached {
                DiffMode::IndexVsHead
            } else if let (Some(_c1), Some(_c2)) = (&commit1, &commit2) {
                DiffMode::CommitVsCommit
            } else if commit1.is_some() {
                DiffMode::WorkingVsCommit
            } else {
                DiffMode::WorkingVsIndex
            };

            let options = DiffOptions {
                mode,
                commit1: commit1.clone(),
                commit2: commit2.clone(),
                paths: vec![],
                use_color: !no_color,
                show_stats: stat,
            };

            if let Err(e) = diff(options) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Merge {
            target,
            abort,
            r#continue,
            no_ff,
            ff_only,
            message,
        } => {
            if !abort && !r#continue && target.is_none() {
                eprintln!("Error: branch or commit to merge is required");
                std::process::exit(1);
            }

            let options = MergeOptions {
                target: target.unwrap_or_default(),
                abort,
                r#continue,
                no_ff,
                ff_only,
                message,
            };

            if let Err(e) = merge(options) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Gc {
            aggressive,
            prune_days,
            dry_run,
        } => {
            let options = GcOptions {
                aggressive,
                prune_days,
                dry_run,
            };

            if let Err(e) = gc(options) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
