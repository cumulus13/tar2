use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Resolved color: either a hex string like "#00FFFF" or a named color
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ColorValue {
    Hex(String),
    Named(String),
}

impl Default for ColorValue {
    fn default() -> Self {
        ColorValue::Named("white".to_string())
    }
}

impl ColorValue {
    pub fn as_str(&self) -> &str {
        match self {
            ColorValue::Hex(s) | ColorValue::Named(s) => s.as_str(),
        }
    }

    pub fn parse_rgb(&self) -> Option<(u8, u8, u8)> {
        let s = self.as_str();
        if let Some(hex) = s.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some((r, g, b));
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsConfig {
    /// Color for directory entries
    #[serde(default = "default_dir_color")]
    pub dir: ColorValue,
    /// Color for regular file entries
    #[serde(default = "default_file_color")]
    pub file: ColorValue,
    /// Color for symlink entries
    #[serde(default = "default_link_color")]
    pub link: ColorValue,
    /// Color for executable entries
    #[serde(default = "default_exec_color")]
    pub exec: ColorValue,
    /// Color for archive name header
    #[serde(default = "default_header_color")]
    pub header: ColorValue,
    /// Color for size values
    #[serde(default = "default_size_color")]
    pub size: ColorValue,
    /// Color for date/time values
    #[serde(default = "default_date_color")]
    pub date: ColorValue,
    /// Color for permissions
    #[serde(default = "default_perm_color")]
    pub perm: ColorValue,
    /// Color for tree branch characters
    #[serde(default = "default_tree_color")]
    pub tree: ColorValue,
    /// Color for warning messages
    #[serde(default = "default_warn_color")]
    pub warn: ColorValue,
    /// Color for error messages
    #[serde(default = "default_error_color")]
    pub error: ColorValue,
    /// Color for success messages
    #[serde(default = "default_ok_color")]
    pub ok: ColorValue,
}

fn default_dir_color() -> ColorValue { ColorValue::Hex("#00BFFF".to_string()) }
fn default_file_color() -> ColorValue { ColorValue::Named("white".to_string()) }
fn default_link_color() -> ColorValue { ColorValue::Hex("#FF69B4".to_string()) }
fn default_exec_color() -> ColorValue { ColorValue::Hex("#00FF7F".to_string()) }
fn default_header_color() -> ColorValue { ColorValue::Hex("#FFD700".to_string()) }
fn default_size_color() -> ColorValue { ColorValue::Hex("#00FFFF".to_string()) }
fn default_date_color() -> ColorValue { ColorValue::Hex("#DDA0DD".to_string()) }
fn default_perm_color() -> ColorValue { ColorValue::Hex("#FFA500".to_string()) }
fn default_tree_color() -> ColorValue { ColorValue::Hex("#808080".to_string()) }
fn default_warn_color() -> ColorValue { ColorValue::Hex("#FFFF00".to_string()) }
fn default_error_color() -> ColorValue { ColorValue::Hex("#FF4444".to_string()) }
fn default_ok_color() -> ColorValue { ColorValue::Hex("#44FF44".to_string()) }

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            dir: default_dir_color(),
            file: default_file_color(),
            link: default_link_color(),
            exec: default_exec_color(),
            header: default_header_color(),
            size: default_size_color(),
            date: default_date_color(),
            perm: default_perm_color(),
            tree: default_tree_color(),
            warn: default_warn_color(),
            error: default_error_color(),
            ok: default_ok_color(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojisConfig {
    #[serde(default = "default_dir_emoji")]
    pub dir: String,
    #[serde(default = "default_file_emoji")]
    pub file: String,
    #[serde(default = "default_link_emoji")]
    pub link: String,
    #[serde(default = "default_archive_emoji")]
    pub archive: String,
    #[serde(default = "default_image_emoji")]
    pub image: String,
    #[serde(default = "default_video_emoji")]
    pub video: String,
    #[serde(default = "default_audio_emoji")]
    pub audio: String,
    #[serde(default = "default_doc_emoji")]
    pub doc: String,
    #[serde(default = "default_code_emoji")]
    pub code: String,
    #[serde(default = "default_ok_emoji")]
    pub ok: String,
    #[serde(default = "default_warn_emoji")]
    pub warn: String,
    #[serde(default = "default_error_emoji")]
    pub error: String,
}

fn default_dir_emoji() -> String { "📁".to_string() }
fn default_file_emoji() -> String { "📄".to_string() }
fn default_link_emoji() -> String { "🔗".to_string() }
fn default_archive_emoji() -> String { "📦".to_string() }
fn default_image_emoji() -> String { "🖼️".to_string() }
fn default_video_emoji() -> String { "🎬".to_string() }
fn default_audio_emoji() -> String { "🎵".to_string() }
fn default_doc_emoji() -> String { "📝".to_string() }
fn default_code_emoji() -> String { "💻".to_string() }
fn default_ok_emoji() -> String { "✅".to_string() }
fn default_warn_emoji() -> String { "⚠️".to_string() }
fn default_error_emoji() -> String { "❌".to_string() }

impl Default for EmojisConfig {
    fn default() -> Self {
        Self {
            dir: default_dir_emoji(),
            file: default_file_emoji(),
            link: default_link_emoji(),
            archive: default_archive_emoji(),
            image: default_image_emoji(),
            video: default_video_emoji(),
            audio: default_audio_emoji(),
            doc: default_doc_emoji(),
            code: default_code_emoji(),
            ok: default_ok_emoji(),
            warn: default_warn_emoji(),
            error: default_error_emoji(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Show emoji in output
    #[serde(default = "default_true")]
    pub emoji: bool,
    /// Use colors in output
    #[serde(default = "default_true")]
    pub colors: bool,
    /// Default tree depth (0 = unlimited)
    #[serde(default)]
    pub tree_depth: u32,
    /// Show file sizes in human-readable format
    #[serde(default = "default_true")]
    pub human_readable: bool,
    /// Show verbose output by default
    #[serde(default)]
    pub verbose: bool,
    /// Progress bar style: "bar", "spinner", "none"
    #[serde(default = "default_progress_style")]
    pub progress_style: String,
    /// Date format string
    #[serde(default = "default_date_format")]
    pub date_format: String,
    /// Tree branch style: "unicode", "ascii"
    #[serde(default = "default_tree_style")]
    pub tree_style: String,
}

fn default_true() -> bool { true }
fn default_progress_style() -> String { "bar".to_string() }
fn default_date_format() -> String { "%Y-%m-%d %H:%M".to_string() }
fn default_tree_style() -> String { "unicode".to_string() }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            emoji: true,
            colors: true,
            tree_depth: 0,
            human_readable: true,
            verbose: false,
            progress_style: default_progress_style(),
            date_format: default_date_format(),
            tree_style: default_tree_style(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorsConfig,
    #[serde(default)]
    pub emojis: EmojisConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

impl Config {
    /// Get config value by dotted key path (e.g. "colors.dir", "display.emoji")
    pub fn get_value(&self, key: &str) -> Option<String> {
        let serialized = serde_json::to_value(self).ok()?;
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &serialized;
        for part in &parts {
            current = current.get(part)?;
        }
        Some(match current {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            other => other.to_string(),
        })
    }

    /// Set config value by dotted key path
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() < 2 {
            anyhow::bail!("Key must be in format 'section.field' (e.g. 'colors.dir')");
        }
        match parts[0] {
            "colors" => self.set_color_value(parts[1], value)?,
            "emojis" => self.set_emoji_value(parts[1], value)?,
            "display" => self.set_display_value(parts[1], value)?,
            s => anyhow::bail!("Unknown config section: '{}'. Valid: colors, emojis, display", s),
        }
        Ok(())
    }

    fn set_color_value(&mut self, key: &str, val: &str) -> Result<()> {
        let color = if val.starts_with('#') || is_named_color(val) {
            if val.starts_with('#') {
                ColorValue::Hex(val.to_string())
            } else {
                ColorValue::Named(val.to_string())
            }
        } else {
            anyhow::bail!("Invalid color '{}'. Use hex (#RRGGBB) or named color (red, green, blue, ...)", val);
        };
        match key {
            "dir" => self.colors.dir = color,
            "file" => self.colors.file = color,
            "link" => self.colors.link = color,
            "exec" => self.colors.exec = color,
            "header" => self.colors.header = color,
            "size" => self.colors.size = color,
            "date" => self.colors.date = color,
            "perm" => self.colors.perm = color,
            "tree" => self.colors.tree = color,
            "warn" => self.colors.warn = color,
            "error" => self.colors.error = color,
            "ok" => self.colors.ok = color,
            k => anyhow::bail!("Unknown color key: '{}'. Valid: dir, file, link, exec, header, size, date, perm, tree, warn, error, ok", k),
        }
        Ok(())
    }

    fn set_emoji_value(&mut self, key: &str, val: &str) -> Result<()> {
        match key {
            "dir" => self.emojis.dir = val.to_string(),
            "file" => self.emojis.file = val.to_string(),
            "link" => self.emojis.link = val.to_string(),
            "archive" => self.emojis.archive = val.to_string(),
            "image" => self.emojis.image = val.to_string(),
            "video" => self.emojis.video = val.to_string(),
            "audio" => self.emojis.audio = val.to_string(),
            "doc" => self.emojis.doc = val.to_string(),
            "code" => self.emojis.code = val.to_string(),
            "ok" => self.emojis.ok = val.to_string(),
            "warn" => self.emojis.warn = val.to_string(),
            "error" => self.emojis.error = val.to_string(),
            k => anyhow::bail!("Unknown emoji key: '{}'. Valid: dir, file, link, archive, image, video, audio, doc, code, ok, warn, error", k),
        }
        Ok(())
    }

    fn set_display_value(&mut self, key: &str, val: &str) -> Result<()> {
        match key {
            "emoji" => self.display.emoji = parse_bool(val)?,
            "colors" => self.display.colors = parse_bool(val)?,
            "tree_depth" => self.display.tree_depth = val.parse().context("tree_depth must be a number")?,
            "human_readable" => self.display.human_readable = parse_bool(val)?,
            "verbose" => self.display.verbose = parse_bool(val)?,
            "progress_style" => {
                if !["bar", "spinner", "none"].contains(&val) {
                    anyhow::bail!("progress_style must be 'bar', 'spinner', or 'none'");
                }
                self.display.progress_style = val.to_string();
            }
            "date_format" => self.display.date_format = val.to_string(),
            "tree_style" => {
                if !["unicode", "ascii"].contains(&val) {
                    anyhow::bail!("tree_style must be 'unicode' or 'ascii'");
                }
                self.display.tree_style = val.to_string();
            }
            k => anyhow::bail!("Unknown display key: '{}'. Valid: emoji, colors, tree_depth, human_readable, verbose, progress_style, date_format, tree_style", k),
        }
        Ok(())
    }

    /// List all config keys and their values
    pub fn list_all(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();
        let sections = ["colors", "emojis", "display"];
        let serialized = serde_json::to_value(self).unwrap_or_default();
        for section in &sections {
            if let Some(obj) = serialized.get(section).and_then(|v| v.as_object()) {
                for (k, v) in obj {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        other => other.to_string(),
                    };
                    result.push((format!("{}.{}", section, k), val));
                }
            }
        }
        result
    }
}

fn parse_bool(s: &str) -> Result<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => anyhow::bail!("Cannot parse '{}' as boolean (use true/false/yes/no/1/0)", s),
    }
}

fn is_named_color(s: &str) -> bool {
    matches!(s, "black" | "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "white"
        | "bright_black" | "bright_red" | "bright_green" | "bright_yellow"
        | "bright_blue" | "bright_magenta" | "bright_cyan" | "bright_white")
}

// ─── Config file resolution ───────────────────────────────────────────────────

/// Returns ordered candidate config file paths for the current platform.
/// The exe base name (e.g. "tar") is used so both "tar.toml" and ".tar.toml" are checked.
pub fn config_candidates(exe_name: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let names = [
        format!("{}.toml", exe_name),
        format!(".{}.toml", exe_name),
        format!("{}.json", exe_name),
        format!(".{}.json", exe_name),
    ];

    // 1. XDG_CONFIG_HOME / platform config dirs
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let base = PathBuf::from(xdg).join(exe_name);
            for n in &names { paths.push(base.join(n)); }
        }
        if let Some(home) = dirs::home_dir() {
            let base = home.join(".config").join(exe_name);
            for n in &names { paths.push(base.join(n)); }
            for n in &names { paths.push(home.join(n)); }
        }
        paths.push(PathBuf::from(format!("/etc/{}/{}.toml", exe_name, exe_name)));
        paths.push(PathBuf::from(format!("/etc/{}.toml", exe_name)));
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let app_support = home.join("Library").join("Application Support").join(exe_name);
            for n in &names { paths.push(app_support.join(n)); }
            let cfg = home.join(".config").join(exe_name);
            for n in &names { paths.push(cfg.join(n)); }
            for n in &names { paths.push(home.join(n)); }
        }
        paths.push(PathBuf::from(format!("/Library/Application Support/{}/{}.toml", exe_name, exe_name)));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = dirs::config_dir() {
            let base = appdata.join(exe_name);
            for n in &names { paths.push(base.join(n)); }
        }
        if let Some(home) = dirs::home_dir() {
            for n in &names { paths.push(home.join(n)); }
        }
    }

    // 2. Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        for n in &names { paths.push(cwd.join(n)); }
    }

    // 3. Executable's own directory (same name as binary)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for n in &names { paths.push(dir.join(n)); }
        }
    }

    paths
}

/// Load config from the first found candidate path.
pub fn load_config(exe_name: &str) -> (Config, Option<PathBuf>) {
    for path in config_candidates(exe_name) {
        if path.exists() {
            if let Ok(cfg) = load_from_path(&path) {
                return (cfg, Some(path));
            }
        }
    }
    (Config::default(), None)
}

/// Load config from an explicit path.
pub fn load_from_path(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read config file: {}", path.display()))?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "json" => serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in config: {}", path.display())),
        _ => toml::from_str(&content)
            .with_context(|| format!("Invalid TOML in config: {}", path.display())),
    }
}

/// Save config to path.
pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create config dir: {}", parent.display()))?;
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("toml");
    let content = if ext == "json" {
        serde_json::to_string_pretty(config)?
    } else {
        toml::to_string_pretty(config)?
    };
    std::fs::write(path, content)
        .with_context(|| format!("Cannot write config file: {}", path.display()))
}

/// Returns the default save path for the current platform.
pub fn default_config_path(exe_name: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = dirs::config_dir() {
            return appdata.join(exe_name).join(format!("{}.toml", exe_name));
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join(exe_name).join(format!("{}.toml", exe_name));
        }
    }
    PathBuf::from(format!("{}.toml", exe_name))
}
