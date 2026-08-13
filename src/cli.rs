use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

/// tar2 — A feature-rich tar replacement with tree view, colors, emoji, and cross-platform config.
#[derive(Parser, Debug)]
#[command(
    name = "tar",
    bin_name = "tar",
    author = "Hadi Cahyadi <cumulus13@gmail.com>",
    version,
    about = "tar2 — archive tool with tree view, colors, emoji & smart config",
    long_about = None,
    after_help = "EXAMPLES:\n  tar -czf archive.tar.gz dir/          Create gzip-compressed archive\n  tar -xzf archive.tar.gz -C /tmp/      Extract to /tmp/\n  tar -tf archive.tar.gz                List contents\n  tar tree archive.tar.gz               Show tree view\n  tar config get colors.dir             Get a config value\n  tar config set colors.dir '#FF6600'   Set a config value"
)]
pub struct Cli {
    // ── Traditional tar flags (single-letter, combinable) ──────────────────

    /// Create archive
    #[arg(short = 'c', conflicts_with_all = ["extract", "list", "append", "update"])]
    pub create: bool,

    /// Extract archive
    #[arg(short = 'x', conflicts_with_all = ["create", "list", "append", "update"])]
    pub extract: bool,

    /// List contents
    #[arg(short = 't', conflicts_with_all = ["create", "extract", "append", "update"])]
    pub list: bool,

    /// Append files to archive
    #[arg(short = 'r', conflicts_with_all = ["create", "extract", "list", "update"])]
    pub append: bool,

    /// Update archive with newer files
    #[arg(short = 'u', conflicts_with_all = ["create", "extract", "list", "append"])]
    pub update: bool,

    /// Archive file name
    #[arg(short = 'f', value_name = "ARCHIVE")]
    pub file: Option<PathBuf>,

    /// Compress with gzip
    #[arg(short = 'z', conflicts_with_all = ["bzip2", "xz", "zstd"])]
    pub gzip: bool,

    /// Compress with bzip2
    #[arg(short = 'j', conflicts_with_all = ["gzip", "xz", "zstd"])]
    pub bzip2: bool,

    /// Compress with xz
    #[arg(short = 'J', conflicts_with_all = ["gzip", "bzip2", "zstd"])]
    pub xz: bool,

    /// Compress with zstd
    #[arg(long, conflicts_with_all = ["gzip", "bzip2", "xz"])]
    pub zstd: bool,

    /// Verbose output
    #[arg(short = 'v', action = ArgAction::Count)]
    pub verbose: u8,

    /// Change to directory before operation
    #[arg(short = 'C', value_name = "DIR")]
    pub directory: Option<PathBuf>,

    /// Dereference symlinks (follow them)
    #[arg(short = 'L', long = "dereference")]
    pub dereference: bool,

    /// Preserve file permissions
    #[arg(short = 'p', long = "preserve-permissions")]
    pub preserve_permissions: bool,

    /// Use numeric UID/GID
    #[arg(long = "numeric-owner")]
    pub numeric_owner: bool,

    /// Strip N leading path components
    #[arg(long = "strip-components", value_name = "N", default_value = "0")]
    pub strip_components: u32,

    /// Exclude files matching PATTERN (glob)
    #[arg(long = "exclude", value_name = "PATTERN", action = ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Exclude files matching REGEX
    #[arg(long = "exclude-regex", value_name = "REGEX", action = ArgAction::Append)]
    pub exclude_regex: Vec<String>,

    /// Exclude files listed in FILE (one per line)
    #[arg(long = "exclude-from", value_name = "FILE")]
    pub exclude_from: Option<PathBuf>,

    /// Include only files matching PATTERN (glob)
    #[arg(long = "include", value_name = "PATTERN", action = ArgAction::Append)]
    pub include: Vec<String>,

    /// Include only files matching REGEX
    #[arg(long = "include-regex", value_name = "REGEX", action = ArgAction::Append)]
    pub include_regex: Vec<String>,

    /// Load exclusion patterns from an ignore file (gitignore syntax). Repeatable.
    #[arg(long = "ignore-file", value_name = "FILE", action = ArgAction::Append)]
    pub ignore_file: Vec<PathBuf>,

    /// Do not auto-detect standard ignore files (.gitignore, .dockerignore,
    /// .tarignore, .tar2ignore, .npmignore, .hgignore, .ignore) alongside the
    /// working directory / source paths
    #[arg(long = "no-auto-ignore")]
    pub no_auto_ignore: bool,

    /// Include the .gitignore/.dockerignore/etc. files that were used to
    /// filter this archive inside the packed archive itself (default: they
    /// are left out once they've done their job, like `npm pack` leaves out
    /// .npmignore). Turn this on if the resulting archive may later be
    /// unpacked and re-filtered by another tool (plain tar, 7-Zip, WinRAR,
    /// etc.) that should see the same rules.
    #[arg(long = "keep-ignore-files")]
    pub keep_ignore_files: bool,

    /// Compression level (1-9 or 1-22 for zstd)
    #[arg(long, value_name = "LEVEL")]
    pub compression_level: Option<u32>,

    /// Verify/test archive integrity
    #[arg(long = "test", short = 'W')]
    pub test: bool,

    /// Show progress bar
    #[arg(long, default_value = "true")]
    pub progress: bool,

    /// Disable colors
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Disable emoji
    #[arg(long = "no-emoji")]
    pub no_emoji: bool,

    /// Override config file path
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// sed-style transform applied to file names (s/old/new/)
    #[arg(long = "transform", value_name = "EXPR")]
    pub transform: Option<String>,

    /// Subcommands (tree, config, etc.)
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Source files/dirs (for create) or patterns (for extract/list)
    #[arg(value_name = "FILE", trailing_var_arg = true)]
    pub files: Vec<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show archive contents as a directory tree
    Tree {
        /// Archive file
        archive: PathBuf,
        /// Maximum depth to display (0 = unlimited)
        #[arg(short = 'd', long = "depth", default_value = "0")]
        depth: u32,
        /// Show file sizes
        #[arg(short = 's', long = "size")]
        size: bool,
        /// Show modification times
        #[arg(short = 't', long = "time")]
        time: bool,
        /// Show permissions
        #[arg(short = 'p', long = "perm")]
        perm: bool,
        /// Use ASCII art instead of Unicode for tree lines
        #[arg(long = "ascii")]
        ascii: bool,
        /// Glob patterns to include
        #[arg(long = "include", value_name = "PATTERN", action = ArgAction::Append)]
        include: Vec<String>,
        /// Glob patterns to exclude
        #[arg(long = "exclude", value_name = "PATTERN", action = ArgAction::Append)]
        exclude: Vec<String>,
        /// Load exclusion patterns from an ignore file (gitignore syntax). Repeatable.
        #[arg(long = "ignore-file", value_name = "FILE", action = ArgAction::Append)]
        ignore_file: Vec<PathBuf>,
        /// Do not auto-detect standard ignore files (.gitignore, .dockerignore,
        /// .tarignore, .tar2ignore, .npmignore, .hgignore, .ignore) in the
        /// current directory
        #[arg(long = "no-auto-ignore")]
        no_auto_ignore: bool,
    },

    /// Get or set configuration values
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Get a config value (e.g. "colors.dir")
    Get {
        /// Dotted key path (section.key)
        key: String,
    },
    /// Set a config value (e.g. "colors.dir" "#FF6600")
    Set {
        /// Dotted key path (section.key)
        key: String,
        /// New value
        value: String,
    },
    /// List all config keys and values
    List,
    /// Print the path(s) where config is searched
    Paths,
    /// Show the active config file path
    Which,
    /// Reset config to defaults
    Reset,
    /// Open config file in $EDITOR
    Edit,
}

/// Resolved compression from flags or filename
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None,
    Gzip,
    Bzip2,
    Xz,
    Zstd,
}

impl Compression {
    pub fn label(self) -> &'static str {
        match self {
            Compression::None => "none",
            Compression::Gzip => "gzip",
            Compression::Bzip2 => "bzip2",
            Compression::Xz => "xz",
            Compression::Zstd => "zstd",
        }
    }
}
