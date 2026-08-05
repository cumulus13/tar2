use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use humansize::{format_size, BINARY};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

use crate::cli::Compression;
use crate::config::Config;
use crate::filter::FileFilter;

// ─── Compression wrappers ─────────────────────────────────────────────────────

pub fn open_reader(path: &Path, comp: Compression) -> Result<Box<dyn Read + Send>> {
    let file = File::open(path)
        .with_context(|| format!("Cannot open archive: {}", path.display()))?;
    let buf = BufReader::new(file);
    Ok(match comp {
        Compression::None => Box::new(buf),
        Compression::Gzip => Box::new(flate2::read::GzDecoder::new(buf)),
        Compression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(buf)),
        Compression::Xz => Box::new(xz2::read::XzDecoder::new(buf)),
        Compression::Zstd => Box::new(zstd::stream::read::Decoder::new(buf)?),
    })
}

pub fn open_writer(path: &Path, comp: Compression, level: Option<u32>) -> Result<Box<dyn Write + Send>> {
    let file = File::create(path)
        .with_context(|| format!("Cannot create archive: {}", path.display()))?;
    let buf = BufWriter::new(file);
    Ok(match comp {
        Compression::None => Box::new(buf),
        Compression::Gzip => {
            let lvl = level.unwrap_or(6);
            let lvl = flate2::Compression::new(lvl.min(9));
            Box::new(flate2::write::GzEncoder::new(buf, lvl))
        }
        Compression::Bzip2 => {
            let lvl = level.unwrap_or(6);
            let lvl = bzip2::Compression::new(lvl.min(9));
            Box::new(bzip2::write::BzEncoder::new(buf, lvl))
        }
        Compression::Xz => {
            let lvl = level.unwrap_or(6);
            Box::new(xz2::write::XzEncoder::new(buf, lvl.min(9)))
        }
        Compression::Zstd => {
            let lvl = level.unwrap_or(3) as i32;
            Box::new(zstd::stream::write::Encoder::new(buf, lvl)?.auto_finish())
        }
    })
}

// ─── Entry metadata ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EntryInfo {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub perm: Option<u32>,
    pub is_dir: bool,
    pub is_link: bool,
    pub link_target: Option<PathBuf>,
}

impl EntryInfo {
    pub fn size_human(&self) -> String {
        format_size(self.size, BINARY)
    }

    pub fn mtime_str(&self, fmt: &str) -> String {
        match &self.mtime {
            Some(dt) => dt.format(fmt).to_string(),
            None => "?".to_string(),
        }
    }

    pub fn perm_str(&self) -> String {
        match self.perm {
            #[cfg(unix)]
            Some(p) => unix_perm_string(p),
            _ => "?????????".to_string(),
        }
    }
}

#[cfg(unix)]
fn unix_perm_string(mode: u32) -> String {
    let chars = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    let type_char = if mode & 0o170000 == 0o040000 { 'd' }
        else if mode & 0o170000 == 0o120000 { 'l' }
        else { '-' };
    let mut s = String::with_capacity(10);
    s.push(type_char);
    for (bit, c) in &chars {
        s.push(if mode & bit != 0 { *c } else { '-' });
    }
    s
}

// ─── Create ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct CreateOptions<'a> {
    pub archive: &'a Path,
    pub sources: &'a [PathBuf],
    pub compression: Compression,
    pub comp_level: Option<u32>,
    pub filter: &'a FileFilter,
    pub dereference: bool,
    pub verbose: bool,
    pub show_progress: bool,
    pub preserve_permissions: bool,
    pub numeric_owner: bool,
    pub transform: Option<&'a str>,
    pub config: &'a Config,
}

pub fn create(opts: CreateOptions) -> Result<usize> {
    let writer = open_writer(opts.archive, opts.compression, opts.comp_level)?;
    let mut builder = tar::Builder::new(writer);
    builder.follow_symlinks(opts.dereference);

    let pb = if opts.show_progress {
        Some(make_spinner("Creating archive"))
    } else {
        None
    };

    let mut count = 0usize;

    for source in opts.sources {
        let source = if source.is_relative() {
            std::env::current_dir()?.join(source)
        } else {
            source.clone()
        };

        if source.is_dir() {
            for entry in WalkDir::new(&source)
                .follow_links(opts.dereference)
                .sort_by_file_name()
            {
                let entry = entry.with_context(|| format!("Walking {}", source.display()))?;
                let rel = entry.path().strip_prefix(&source)
                    .unwrap_or(entry.path());

                if rel == Path::new("") { continue; }
                if !opts.filter.matches(entry.path()) { continue; }

                let archive_path = apply_transform(rel, opts.transform);

                if opts.verbose {
                    let em = crate::colors::get_emoji(
                        &entry.file_name().to_string_lossy(),
                        entry.file_type().is_dir(),
                        entry.path_is_symlink(),
                        opts.config,
                    );
                    eprintln!("{} {}", em, archive_path.display());
                }

                if let Some(ref pb) = pb {
                    pb.set_message(format!("Adding {}", archive_path.display()));
                }

                add_path_to_builder(&mut builder, entry.path(), &archive_path, opts.preserve_permissions)?;
                count += 1;
            }
        } else if source.exists() {
            if !opts.filter.matches(&source) { continue; }
            let fname = source.file_name().unwrap_or(source.as_os_str());
            let archive_path = apply_transform(Path::new(fname), opts.transform);

            if opts.verbose {
                let em = crate::colors::get_emoji(
                    &source.file_name().unwrap_or_default().to_string_lossy(),
                    false, false, opts.config,
                );
                eprintln!("{} {}", em, archive_path.display());
            }

            add_path_to_builder(&mut builder, &source, &archive_path, opts.preserve_permissions)?;
            count += 1;
        } else {
            anyhow::bail!("Source not found: {}", source.display());
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Done");
    }

    builder.finish()?;
    Ok(count)
}

fn add_path_to_builder<W: Write>(
    builder: &mut tar::Builder<W>,
    real: &Path,
    name: &Path,
    preserve_perms: bool,
) -> Result<()> {
    let meta = real.symlink_metadata()
        .with_context(|| format!("Cannot stat: {}", real.display()))?;

    if meta.is_symlink() {
        let mut header = tar::Header::new_gnu();
        header.set_metadata(&meta);
        header.set_path(name)?;
        let target = fs::read_link(real)?;
        header.set_link_name(&target)?;
        header.set_size(0);
        header.set_cksum();
        builder.append(&header, io::empty())?;
    } else if meta.is_dir() {
        builder.append_dir(name, real)
            .with_context(|| format!("Cannot add dir: {}", real.display()))?;
    } else {
        let mut file = File::open(real)
            .with_context(|| format!("Cannot open: {}", real.display()))?;
        let mut header = tar::Header::new_gnu();
        header.set_metadata(&meta);
        if !preserve_perms {
            header.set_mode(0o644);
        }
        header.set_path(name)?;
        header.set_cksum();
        builder.append_data(&mut header, name, &mut file)
            .with_context(|| format!("Cannot archive: {}", real.display()))?;
    }
    Ok(())
}

fn apply_transform<'a>(path: &'a Path, transform: Option<&str>) -> std::borrow::Cow<'a, Path> {
    if let Some(_t) = transform {
        // TODO: sed-style transform; for now pass through
    }
    std::borrow::Cow::Borrowed(path)
}

// ─── Extract ──────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct ExtractOptions<'a> {
    pub archive: &'a Path,
    pub compression: Compression,
    pub dest: &'a Path,
    pub filter: &'a FileFilter,
    pub verbose: bool,
    pub show_progress: bool,
    pub preserve_permissions: bool,
    pub strip_components: u32,
    pub config: &'a Config,
}

pub fn extract(opts: ExtractOptions) -> Result<usize> {
    let reader = open_reader(opts.archive, opts.compression)?;
    let mut archive = tar::Archive::new(reader);

    #[cfg(unix)]
    archive.set_preserve_permissions(opts.preserve_permissions);
    archive.set_overwrite(true);

    fs::create_dir_all(opts.dest)
        .with_context(|| format!("Cannot create dest dir: {}", opts.dest.display()))?;

    let pb = if opts.show_progress {
        Some(make_spinner("Extracting"))
    } else {
        None
    };

    let mut count = 0usize;

    let reader2 = open_reader(opts.archive, opts.compression)?;
    let mut archive2 = tar::Archive::new(reader2);

    for entry in archive2.entries()? {
        let mut entry = entry?;
        let raw_path = entry.path()?.into_owned();

        // Strip components
        let stripped = strip_components(&raw_path, opts.strip_components);
        let stripped = match stripped {
            Some(p) => p,
            None => continue,
        };

        let entry_is_dir = entry.header().entry_type().is_dir();
        if !opts.filter.matches_typed(&stripped, entry_is_dir) { continue; }

        let dest_path = opts.dest.join(&stripped);

        if opts.verbose {
            let is_dir = entry.header().entry_type().is_dir();
            let em = crate::colors::get_emoji(
                &stripped.file_name().unwrap_or_default().to_string_lossy(),
                is_dir, false, opts.config,
            );
            eprintln!("{} {}", em, stripped.display());
        }

        if let Some(ref pb) = pb {
            pb.set_message(format!("Extracting {}", stripped.display()));
        }

        if stripped.components().count() > 0 {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        entry.unpack(&dest_path)
            .with_context(|| format!("Cannot extract: {}", stripped.display()))?;
        count += 1;
    }
    let _ = archive; // close original

    if let Some(pb) = pb {
        pb.finish_with_message("Done");
    }
    Ok(count)
}

fn strip_components(path: &Path, n: u32) -> Option<PathBuf> {
    if n == 0 { return Some(path.to_path_buf()); }
    let mut comps = path.components();
    for _ in 0..n {
        comps.next()?;
    }
    let rest: PathBuf = comps.collect();
    if rest.as_os_str().is_empty() { None } else { Some(rest) }
}

// ─── List ─────────────────────────────────────────────────────────────────────

pub fn list_entries(archive: &Path, compression: Compression, filter: &FileFilter) -> Result<Vec<EntryInfo>> {
    let reader = open_reader(archive, compression)?;
    let mut arch = tar::Archive::new(reader);
    let mut entries = Vec::new();

    for entry in arch.entries()? {
        let entry = entry?;
        let raw_path = entry.path()?.into_owned();

        let header = entry.header();
        if !filter.matches_typed(&raw_path, header.entry_type().is_dir()) { continue; }

        let size = header.size().unwrap_or(0);

        let mtime = header.mtime().ok().and_then(|secs| {
            UNIX_EPOCH.checked_add(std::time::Duration::from_secs(secs))
                .map(DateTime::<Local>::from)
        });

        let perm = header.mode().ok();
        let entry_type = header.entry_type();
        let is_dir = entry_type.is_dir();
        let is_link = entry_type.is_symlink() || entry_type.is_hard_link();
        let link_target = entry.link_name().ok().flatten().map(|p| p.into_owned());

        entries.push(EntryInfo {
            path: raw_path,
            size,
            mtime,
            perm,
            is_dir,
            is_link,
            link_target,
        });
    }

    Ok(entries)
}

// ─── Diff/Update ──────────────────────────────────────────────────────────────

pub fn update_archive(archive: &Path, compression: Compression, sources: &[PathBuf], filter: &FileFilter, config: &Config) -> Result<usize> {
    // Collect existing entries' mtimes from archive
    let existing: std::collections::HashMap<PathBuf, u64> = {
        let reader = open_reader(archive, compression)?;
        let mut arch = tar::Archive::new(reader);
        let mut map = std::collections::HashMap::new();
        for entry in arch.entries()? {
            let entry = entry?;
            let path = entry.path()?.into_owned();
            let mtime = entry.header().mtime().unwrap_or(0);
            map.insert(path, mtime);
        }
        map
    };

    // Determine which files are newer
    let mut to_add: Vec<PathBuf> = Vec::new();
    for source in sources {
        for entry in WalkDir::new(source).follow_links(false) {
            let entry = entry?;
            if entry.file_type().is_dir() { continue; }
            if !filter.matches(entry.path()) { continue; }

            let rel = entry.path().strip_prefix(source).unwrap_or(entry.path()).to_path_buf();
            let disk_mtime = entry.metadata()?.modified().ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let arch_mtime = existing.get(&rel).copied().unwrap_or(0);
            if disk_mtime > arch_mtime {
                to_add.push(entry.path().to_owned());
            }
        }
    }

    let count = to_add.len();
    if count == 0 {
        return Ok(0);
    }

    // Re-create archive including old + new entries (tar append is tricky with compression)
    let tmp = archive.with_extension("tmp.tar");
    let writer = open_writer(&tmp, compression, None)?;
    let mut builder = tar::Builder::new(writer);

    // Copy old entries not being replaced
    {
        let reader = open_reader(archive, compression)?;
        let mut arch = tar::Archive::new(reader);
        for entry in arch.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let rel = path.clone();
            let replacing = to_add.iter().any(|p| {
                p.file_name().map(|f| Path::new(f) == rel.as_path()).unwrap_or(false)
            });
            if !replacing {
                let header = entry.header().clone();
                builder.append(&header, &mut entry)?;
            }
        }
    }

    // Add new/updated files
    for src in &to_add {
        let fname = src.file_name().unwrap_or(src.as_os_str());
        let name = Path::new(fname);
        add_path_to_builder(&mut builder, src, name, true)?;
        if config.display.verbose {
            eprintln!("u {}", src.display());
        }
    }

    builder.finish()?;
    fs::rename(&tmp, archive)?;
    Ok(count)
}

// ─── Test/Verify ──────────────────────────────────────────────────────────────

pub fn verify_archive(archive: &Path, compression: Compression) -> Result<(usize, usize)> {
    let reader = open_reader(archive, compression)?;
    let mut arch = tar::Archive::new(reader);
    let mut ok = 0;
    let mut errors = 0;

    for entry in arch.entries()? {
        match entry {
            Ok(mut e) => {
                let mut buf = vec![0u8; 4096];
                loop {
                    match e.read(&mut buf) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("Error reading {}: {}", e.path().unwrap_or_default().display(), err);
                            errors += 1;
                            break;
                        }
                    }
                }
                ok += 1;
            }
            Err(e) => {
                eprintln!("Entry error: {}", e);
                errors += 1;
            }
        }
    }

    Ok((ok, errors))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub fn detect_compression(path: &Path) -> Compression {
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") { return Compression::Gzip; }
    if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".tbz") { return Compression::Bzip2; }
    if name.ends_with(".tar.xz") || name.ends_with(".txz") { return Compression::Xz; }
    if name.ends_with(".tar.zst") || name.ends_with(".tzst") { return Compression::Zstd; }
    Compression::None
}

fn make_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠸", "⠴", "⠦", "⠇"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
