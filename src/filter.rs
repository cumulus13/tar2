use anyhow::Result;
use glob::Pattern;
use regex::Regex;
use std::path::Path;

pub struct FileFilter {
    include_globs: Vec<Pattern>,
    exclude_globs: Vec<Pattern>,
    exclude_regex: Vec<Regex>,
    include_regex: Vec<Regex>,
}

impl FileFilter {
    pub fn new(
        include_globs: &[String],
        exclude_globs: &[String],
        include_re: &[String],
        exclude_re: &[String],
    ) -> Result<Self> {
        let mut ig = Vec::new();
        for p in include_globs {
            ig.push(Pattern::new(p).map_err(|e| anyhow::anyhow!("Invalid glob '{}': {}", p, e))?);
        }
        let mut eg = Vec::new();
        for p in exclude_globs {
            eg.push(Pattern::new(p).map_err(|e| anyhow::anyhow!("Invalid glob '{}': {}", p, e))?);
        }
        let mut iregex = Vec::new();
        for r in include_re {
            iregex.push(Regex::new(r).map_err(|e| anyhow::anyhow!("Invalid regex '{}': {}", r, e))?);
        }
        let mut eregex = Vec::new();
        for r in exclude_re {
            eregex.push(Regex::new(r).map_err(|e| anyhow::anyhow!("Invalid regex '{}': {}", r, e))?);
        }
        Ok(Self { include_globs: ig, exclude_globs: eg, exclude_regex: eregex, include_regex: iregex })
    }

    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let fname = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();

        // Exclude glob
        for pat in &self.exclude_globs {
            if pat.matches(&path_str) || pat.matches(&fname) {
                return false;
            }
        }
        // Exclude regex
        for re in &self.exclude_regex {
            if re.is_match(&path_str) {
                return false;
            }
        }

        // Include glob
        if !self.include_globs.is_empty() {
            let matched = self.include_globs.iter()
                .any(|p| p.matches(&path_str) || p.matches(&fname));
            if !matched {
                // Fall through to include_regex check
                if self.include_regex.is_empty() {
                    return false;
                }
                return self.include_regex.iter().any(|r| r.is_match(&path_str));
            }
            return true;
        }

        // Include regex only
        if !self.include_regex.is_empty() {
            return self.include_regex.iter().any(|r| r.is_match(&path_str));
        }

        true
    }
}
