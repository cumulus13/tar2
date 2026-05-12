// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-05-12
// Description: 
// License: MIT

mod archive;
mod cli;
mod colors;
mod config;
mod filter;
mod tree;

use anyhow::{Context, Result};
use clap::Parser;
use clap_version_flag::colorful_version;
use colored::Colorize;
use std::path::{Path, PathBuf};

use cli::{Cli, Commands, Compression};
use colors::Painter;
use config::{default_config_path, load_config, save_config};
use filter::FileFilter;
use tree::{TreeOptions, build_tree, render_flat, render_tree};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && (args[1] == "-V" || args[1] == "--version") {
        let version = colorful_version!();
        version.print_and_exit();
    }
    
    ctrlc::set_handler(|| {
        eprintln!("\n{}", "Interrupted.".yellow());
        std::process::exit(130);
    })
    .ok();

    if let Err(e) = run() {
        eprintln!("{} {}", "error:".bright_red().bold(), e);
        for cause in e.chain().skip(1) {
            eprintln!("  {} {}", "caused by:".red(), cause);
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();

    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "tar".to_string());

    let (mut cfg, cfg_path) = if let Some(ref explicit) = args.config {
        match config::load_from_path(explicit) {
            Ok(c) => (c, Some(explicit.clone())),
            Err(e) => {
                eprintln!("{} Cannot load config {}: {}", "warn:".yellow(), explicit.display(), e);
                (config::Config::default(), Some(explicit.clone()))
            }
        }
    } else {
        load_config(&exe_name)
    };

    if args.no_color { cfg.display.colors = false; }
    if args.no_emoji { cfg.display.emoji = false; }
    if std::env::var("NO_COLOR").is_ok() { cfg.display.colors = false; }

    if let Some(cmd) = &args.command {
        let colors_on = cfg.display.colors;
        return dispatch_subcommand(cmd, &mut cfg, cfg_path.as_deref(), &exe_name, colors_on);
    }

    let painter = Painter::new(&cfg, cfg.display.colors);

    let archive_path = match &args.file {
        Some(p) => p.clone(),
        None => anyhow::bail!(
            "No archive file specified. Use -f <archive>.\nRun 'tar --help' for usage."
        ),
    };

    let compression = resolve_compression(&args, &archive_path);

    let mut excludes = args.exclude.clone();
    if let Some(ref from_file) = args.exclude_from {
        let content = std::fs::read_to_string(from_file)
            .with_context(|| format!("Cannot read exclude-from file: {}", from_file.display()))?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                excludes.push(line.to_string());
            }
        }
    }

    let file_patterns: Vec<String> = if !args.files.is_empty() && (args.extract || args.list || args.test) {
        args.files.iter().map(|p| p.to_string_lossy().to_string()).collect()
    } else {
        Vec::new()
    };

    let includes: Vec<String> = {
        let mut v = args.include.clone();
        v.extend(file_patterns);
        v
    };

    let filter = FileFilter::new(&includes, &excludes, &args.include_regex, &args.exclude_regex)?;

    let sources: Vec<PathBuf> = if args.create || args.append || args.update {
        args.files.clone()
    } else {
        Vec::new()
    };

    if let Some(ref dir) = args.directory {
        std::env::set_current_dir(dir)
            .with_context(|| format!("Cannot chdir to: {}", dir.display()))?;
    }

    let verbose = args.verbose > 0 || cfg.display.verbose;
    let show_progress = args.progress && cfg.display.progress_style != "none";

    if args.test {
        println!("{} {}", painter.header("Verifying"), painter.file(&archive_path.display().to_string()));
        let (ok, errors) = archive::verify_archive(&archive_path, compression)?;
        if errors == 0 {
            println!("{} {} entries OK", painter.ok(&cfg.emojis.ok), ok);
        } else {
            eprintln!("{} {}/{} entries have errors", painter.error(&cfg.emojis.error), errors, ok + errors);
            std::process::exit(1);
        }
        return Ok(());
    }

    if args.create {
        if sources.is_empty() {
            anyhow::bail!("No source files specified for create (-c)");
        }
        println!("{} {} ({})",
            painter.header(&format!("{} Creating", &cfg.emojis.archive)),
            painter.file(&archive_path.display().to_string()),
            painter.size(compression.label()),
        );
        let count = archive::create(archive::CreateOptions {
            archive: &archive_path,
            sources: &sources,
            compression,
            comp_level: args.compression_level,
            filter: &filter,
            dereference: args.dereference,
            verbose,
            show_progress,
            preserve_permissions: args.preserve_permissions,
            numeric_owner: args.numeric_owner,
            transform: args.transform.as_deref(),
            config: &cfg,
        })?;
        println!("{} {} files archived to {}",
            painter.ok(&cfg.emojis.ok),
            painter.size(&count.to_string()),
            painter.file(&archive_path.display().to_string()),
        );
        return Ok(());
    }

    if args.extract {
        let dest = args.directory.clone().unwrap_or_else(|| PathBuf::from("."));
        println!("{} {} \u{2192} {}",
            painter.header(&format!("{} Extracting", &cfg.emojis.archive)),
            painter.file(&archive_path.display().to_string()),
            painter.dir(&dest.display().to_string()),
        );
        let count = archive::extract(archive::ExtractOptions {
            archive: &archive_path,
            compression,
            dest: &dest,
            filter: &filter,
            verbose,
            show_progress,
            preserve_permissions: args.preserve_permissions,
            strip_components: args.strip_components,
            config: &cfg,
        })?;
        println!("{} {} files extracted",
            painter.ok(&cfg.emojis.ok),
            painter.size(&count.to_string()),
        );
        return Ok(());
    }

    if args.list {
        let entries = archive::list_entries(&archive_path, compression, &filter)?;
        println!("{} {}",
            painter.header(&format!("{} Contents of", &cfg.emojis.archive)),
            painter.file(&archive_path.display().to_string()),
        );
        println!("{}", painter.tree(&"\u{2500}".repeat(60)));
        render_flat(&entries, verbose, &cfg);
        println!("{}", painter.tree(&"\u{2500}".repeat(60)));
        println!("{} {} entries", painter.ok("Total:"), painter.size(&entries.len().to_string()));
        return Ok(());
    }

    if args.append || args.update {
        if sources.is_empty() {
            anyhow::bail!("No source files specified");
        }
        let count = archive::update_archive(&archive_path, compression, &sources, &filter, &cfg)?;
        println!("{} {} files updated/appended", painter.ok(&cfg.emojis.ok), count);
        return Ok(());
    }

    anyhow::bail!("No operation specified. Use -c, -x, -t, -r, -u, or a subcommand.\nRun 'tar --help' for usage.")
}

fn dispatch_subcommand(
    cmd: &Commands,
    cfg: &mut config::Config,
    cfg_path: Option<&Path>,
    exe_name: &str,
    colors_on: bool,
) -> Result<()> {
    let painter = Painter::new(cfg, colors_on);
    match cmd {
        Commands::Tree { archive, depth, size, time, perm, ascii, include, exclude } => {
            cmd_tree(TreeArgs {
                archive,
                depth: *depth,
                show_size: *size,
                show_time: *time,
                show_perm: *perm,
                ascii: *ascii,
                include,
                exclude,
            }, cfg, &painter)
        }
        Commands::Config { action } => {
            cmd_config(action, cfg, cfg_path, exe_name, &painter)
        }
    }
}

struct TreeArgs<'a> {
    archive: &'a Path,
    depth: u32,
    show_size: bool,
    show_time: bool,
    show_perm: bool,
    ascii: bool,
    include: &'a [String],
    exclude: &'a [String],
}

fn cmd_tree(args: TreeArgs, cfg: &config::Config, painter: &Painter) -> Result<()> {
    let TreeArgs { archive, depth, show_size, show_time, show_perm, ascii, include, exclude } = args;
    let compression = archive::detect_compression(archive);
    let filter = FileFilter::new(include, exclude, &[], &[])?;
    let entries = archive::list_entries(archive, compression, &filter)?;

    let eff_depth = if depth > 0 { depth } else { cfg.display.tree_depth };
    let eff_ascii = ascii || cfg.display.tree_style == "ascii";

    println!("{} {}",
        painter.header(&format!("{} Tree:", &cfg.emojis.archive)),
        painter.file(&archive.display().to_string()),
    );
    println!("{}", painter.tree(&"\u{2500}".repeat(60)));

    let root = build_tree(&entries);
    let opts = TreeOptions {
        max_depth: eff_depth,
        show_size,
        show_date: show_time,
        show_perm,
        date_fmt: &cfg.display.date_format,
        unicode: !eff_ascii,
    };
    let rendered = render_tree(&root, &opts, cfg);
    print!("{}", rendered);

    println!("{}", painter.tree(&"\u{2500}".repeat(60)));
    println!("{} {} entries total", painter.ok("Total:"), painter.size(&entries.len().to_string()));
    Ok(())
}

fn cmd_config(
    action: &cli::ConfigAction,
    cfg: &mut config::Config,
    cfg_path: Option<&Path>,
    exe_name: &str,
    painter: &Painter,
) -> Result<()> {
    use cli::ConfigAction;
    match action {
        ConfigAction::Get { key } => {
            match cfg.get_value(key) {
                Some(val) => println!("{} = {}", painter.header(key), painter.ok(&val)),
                None => anyhow::bail!("Unknown config key: '{}'. Use 'tar config list' to see all.", key),
            }
        }
        ConfigAction::Set { key, value } => {
            cfg.set_value(key, value)?;
            let path = cfg_path
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| default_config_path(exe_name));
            save_config(cfg, &path)?;
            println!("{} {} = {} (saved to {})",
                painter.ok(&cfg.emojis.ok), painter.header(key), painter.ok(value),
                painter.file(&path.display().to_string()),
            );
        }
        ConfigAction::List => {
            println!("{}", painter.header("Configuration keys:"));
            println!("{}", painter.tree(&"\u{2500}".repeat(60)));
            let mut last_section = String::new();
            for (k, v) in cfg.list_all() {
                let dot = k.find('.').unwrap_or(k.len());
                let sec = k[..dot].to_string();
                let field = if dot < k.len() { k[dot+1..].to_string() } else { k.clone() };
                if sec != last_section {
                    last_section = sec.clone();
                    println!("\n{}", painter.dir(&format!("[{}]", sec)).to_string().bold());
                }
                println!("  {} = {}", painter.perm(&field), painter.ok(&v));
            }
        }
        ConfigAction::Paths => {
            println!("{}", painter.header("Config file search paths:"));
            println!("{}", painter.tree(&"\u{2500}".repeat(60)));
            for (i, path) in config::config_candidates(exe_name).iter().enumerate() {
                let exists = path.exists();
                let marker = if exists { painter.ok("\u{2713}").to_string() } else { painter.tree("\u{00B7}").to_string() };
                println!(" {} {:<3} {}", marker, i + 1, painter.file(&path.display().to_string()));
            }
        }
        ConfigAction::Which => {
            match cfg_path {
                Some(p) => println!("{} {}", painter.ok("Active config:"), painter.file(&p.display().to_string())),
                None => println!("{} (no config file found, using defaults)", painter.warn(&cfg.emojis.warn)),
            }
        }
        ConfigAction::Reset => {
            let path = cfg_path
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| default_config_path(exe_name));
            let default = config::Config::default();
            save_config(&default, &path)?;
            println!("{} Config reset to defaults: {}", painter.ok(&cfg.emojis.ok), painter.file(&path.display().to_string()));
            *cfg = default;
        }
        ConfigAction::Edit => {
            let path = cfg_path
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| default_config_path(exe_name));
            if !path.exists() {
                save_config(cfg, &path)?;
            }
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    if cfg!(windows) { "notepad".to_string() } else { "nano".to_string() }
                });
            let status = std::process::Command::new(&editor)
                .arg(&path)
                .status()
                .with_context(|| format!("Cannot launch editor: {}", editor))?;
            if !status.success() {
                anyhow::bail!("Editor exited with error");
            }
        }
    }
    Ok(())
}

fn resolve_compression(args: &Cli, path: &Path) -> Compression {
    if args.gzip  { return Compression::Gzip; }
    if args.bzip2 { return Compression::Bzip2; }
    if args.xz    { return Compression::Xz; }
    if args.zstd  { return Compression::Zstd; }
    archive::detect_compression(path)
}
