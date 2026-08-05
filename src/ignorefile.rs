// File: src\ignorefile.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Description: Loads standard "ignore" files (.gitignore, .dockerignore, .tarignore,
//              .tar2ignore, etc.) using gitignore-compatible pattern syntax, so their
//              rules can be applied as exclusion patterns for pack/list/extract/tree.
// License: MIT

use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};

/// Well-known ignore-file names that are auto-detected (in this order) when
/// auto-detection is enabled. All of them use standard gitignore pattern syntax.
pub const STANDARD_IGNORE_FILES: &[&str] = &[
    ".tar2ignore",
    ".tarignore",
    ".gitignore",
    ".dockerignore",
    ".npmignore",
    ".hgignore",
    ".ignore",
];

/// A compiled set of ignore patterns gathered from one or more ignore files
/// and/or ad-hoc pattern lines.
pub struct IgnoreSet {
    matcher: Gitignore,
    pub sources: Vec<PathBuf>,
}

impl IgnoreSet {
    /// Returns true if `path` (or any of its parent components) matches an
    /// ignore rule and isn't re-included by a later `!pattern` negation.
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        matches!(
            self.matcher.matched_path_or_any_parents(path, is_dir),
            ignore::Match::Ignore(_)
        )
    }
}

/// Builds an [`IgnoreSet`] from auto-detected standard ignore files, explicitly
/// named ignore files, and/or raw pattern lines.
pub struct IgnoreSetBuilder {
    builder: GitignoreBuilder,
    sources: Vec<PathBuf>,
    scanned_dirs: Vec<PathBuf>,
}

impl IgnoreSetBuilder {
    /// `root` is the base directory that relative patterns are resolved against.
    pub fn new(root: &Path) -> Self {
        Self { builder: GitignoreBuilder::new(root), sources: Vec::new(), scanned_dirs: Vec::new() }
    }

    /// Looks directly inside `dir` for any of [`STANDARD_IGNORE_FILES`] and
    /// loads every one that exists. Safe to call multiple times / on multiple
    /// directories (e.g. once per source passed to `create`) — a directory
    /// that's already been scanned (by canonical path) is skipped.
    pub fn autodetect(&mut self, dir: &Path) -> Result<&mut Self> {
        let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        if self.scanned_dirs.contains(&canonical) {
            return Ok(self);
        }
        self.scanned_dirs.push(canonical);

        for name in STANDARD_IGNORE_FILES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                self.add_file(&candidate)?;
            }
        }
        Ok(self)
    }

    /// Loads an explicit ignore file (any name, gitignore syntax).
    pub fn add_file(&mut self, path: &Path) -> Result<&mut Self> {
        if let Some(err) = self.builder.add(path) {
            return Err(err).with_context(|| format!("Invalid ignore file: {}", path.display()));
        }
        self.sources.push(path.to_path_buf());
        Ok(self)
    }

    /// Adds a single raw pattern line (e.g. from `--exclude` reused as an
    /// ignore-style exception), without needing a backing file.
    pub fn add_line(&mut self, line: &str) -> Result<&mut Self> {
        self.builder
            .add_line(None, line)
            .map_err(|e| anyhow::anyhow!("Invalid ignore pattern '{}': {}", line, e))?;
        Ok(self)
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Compiles the accumulated patterns. Returns `Ok(None)` if nothing was
    /// ever loaded, so callers can skip ignore-matching entirely.
    pub fn build(self) -> Result<Option<IgnoreSet>> {
        if self.sources.is_empty() {
            return Ok(None);
        }
        let matcher = self.builder.build().context("Failed to compile ignore patterns")?;
        Ok(Some(IgnoreSet { matcher, sources: self.sources }))
    }
}
