use colored::{ColoredString, Colorize};
use crate::config::{ColorValue, Config, ColorsConfig};

/// A snapshot of color settings — owns the data, no lifetime issues.
#[derive(Clone)]
pub struct Painter {
    colors: ColorsConfig,
    enabled: bool,
}

impl Painter {
    pub fn new(config: &Config, enabled: bool) -> Self {
        Self {
            colors: config.colors.clone(),
            enabled,
        }
    }

    fn apply(&self, s: &str, color: &ColorValue) -> ColoredString {
        if !self.enabled {
            return s.normal();
        }
        if let Some((r, g, b)) = color.parse_rgb() {
            return s.truecolor(r, g, b);
        }
        match color.as_str() {
            "black"          => s.black(),
            "red"            => s.red(),
            "green"          => s.green(),
            "yellow"         => s.yellow(),
            "blue"           => s.blue(),
            "magenta"        => s.magenta(),
            "cyan"           => s.cyan(),
            "white"          => s.white(),
            "bright_black"   => s.bright_black(),
            "bright_red"     => s.bright_red(),
            "bright_green"   => s.bright_green(),
            "bright_yellow"  => s.bright_yellow(),
            "bright_blue"    => s.bright_blue(),
            "bright_magenta" => s.bright_magenta(),
            "bright_cyan"    => s.bright_cyan(),
            "bright_white"   => s.bright_white(),
            _                => s.normal(),
        }
    }

    pub fn dir(&self, s: &str)    -> ColoredString { self.apply(s, &self.colors.dir) }
    pub fn file(&self, s: &str)   -> ColoredString { self.apply(s, &self.colors.file) }
    pub fn link(&self, s: &str)   -> ColoredString { self.apply(s, &self.colors.link) }
    pub fn header(&self, s: &str) -> ColoredString { self.apply(s, &self.colors.header).bold() }
    pub fn size(&self, s: &str)   -> ColoredString { self.apply(s, &self.colors.size) }
    pub fn date(&self, s: &str)   -> ColoredString { self.apply(s, &self.colors.date) }
    pub fn perm(&self, s: &str)   -> ColoredString { self.apply(s, &self.colors.perm) }
    pub fn tree(&self, s: &str)   -> ColoredString { self.apply(s, &self.colors.tree) }
    pub fn warn(&self, s: &str)   -> ColoredString { self.apply(s, &self.colors.warn) }
    pub fn error(&self, s: &str)  -> ColoredString { self.apply(s, &self.colors.error).bold() }
    pub fn ok(&self, s: &str)     -> ColoredString { self.apply(s, &self.colors.ok) }
}

/// Get the emoji string for a path entry (returns from config).
pub fn get_emoji<'a>(
    name: &str,
    is_dir: bool,
    is_link: bool,
    config: &'a Config,
) -> &'a str {
    if !config.display.emoji {
        return "";
    }
    if is_link { return &config.emojis.link; }
    if is_dir  { return &config.emojis.dir; }

    let lower = name.to_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "tar" | "gz" | "bz2" | "xz" | "zst" | "tgz" | "tbz2" | "txz"
        | "zip" | "7z" | "rar"
            => &config.emojis.archive,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff"
            => &config.emojis.image,
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm"
            => &config.emojis.video,
        "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a"
            => &config.emojis.audio,
        "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go"
        | "rb" | "php" | "sh" | "bash" | "zsh" | "fish" | "ps1" | "lua"
        | "swift" | "kt" | "cs" | "toml" | "yaml" | "yml" | "json"
            => &config.emojis.code,
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "rst" | "tex"
            => &config.emojis.doc,
        _ => &config.emojis.file,
    }
}
