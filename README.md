# tar2 🗜️

**A production-ready, feature-rich `tar` replacement** with tree view, full color support (hex + named), emoji file icons, and a smart cross-platform config system.

> **Binary name:** `tar2` — drop-in compatible with standard tar CLI flags.  
> **Author:** Hadi Cahyadi <cumulus13@gmail.com>  
> **Homepage:** https://github.com/cumulus13/tar2

[![Screenshot](https://raw.githubusercontent.com/cumulus13/tar2/master/screenshot.png)](https://raw.githubusercontent.com/cumulus13/tar2/master/screenshot.png)

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| **Full tar compatibility** | `-c`, `-x`, `-t`, `-r`, `-u`, `-f`, `-z`, `-j`, `-J`, `--zstd`, `-v`, `-C`, `-p`, `-W`, etc. |
| **Tree view** | `tar tree archive.tar.gz` — shows archive as a directory tree |
| **Depth control** | `-d N` on tree — limit how many levels deep to show |
| **Colors** | Hex colors (`#00FFFF`), named colors (`cyan`), per-category config |
| **Emoji icons** | File-type emoji for dirs, archives, images, video, audio, code, docs |
| **Config system** | TOML/JSON config with `get`/`set`/`list`/`reset`/`edit` |
| **Platform config paths** | Searches standard OS paths + exe directory |
| **Pattern filtering** | `--include`/`--exclude` (glob) + `--include-regex`/`--exclude-regex` |
| **Exclude from file** | `--exclude-from FILE` (one pattern per line, `#` comments) |
| **Ignore files** | Auto-detects `.gitignore`, `.dockerignore`, `.tarignore`, `.tar2ignore`, etc. for pack/list/extract/append/tree |
| **Strip components** | `--strip-components N` on extract |
| **Progress spinner** | Visual feedback during create/extract |
| **Verify/test** | `-W` / `--test` — integrity check |
| **Multi-compression** | gzip, bzip2, xz, zstd — auto-detected from filename |
| **Cross-platform** | Linux, macOS, Windows |

---

## 🚀 Quick Start

```bash
# Create a gzip archive
tar2 -czf archive.tar.gz src/ docs/

# Extract to a directory
tar2 -xzf archive.tar.gz -C /tmp/out/

# List contents (with verbose metadata)
tar2 -tvf archive.tar.gz

# Show as a tree (unlimited depth)
tar2 tree archive.tar.gz

# Show tree, 2 levels deep, with sizes and dates
tar2 tree archive.tar.gz -d 2 --size --time

# Include only .rs files
tar2 -tf archive.tar.gz --include '*.rs'

# Exclude build artifacts
tar2 -czf project.tar.gz . --exclude 'target/**' --exclude '*.o'

# Verify integrity
tar2 -Wf archive.tar.gz
```

---

## 🌲 Tree View

```
tar2 tree myproject.tar.gz -d 3 --size --time --perm
```

```
📦 Tree: myproject.tar.gz
────────────────────────────────────────────────────────────
├── 💻 Cargo.toml  -rw-r--r--  1.2 KiB  2024-11-01 14:30
├── 📁 src  drwxr-xr-x  2024-11-01 14:30
│   ├── 💻 main.rs  -rw-r--r--  8.4 KiB  2024-11-01 14:30
│   └── 📁 modules  drwxr-xr-x  2024-11-01 14:30
│       ├── 💻 archive.rs  -rw-r--r--  12.1 KiB  2024-11-01 14:29
│       └── 💻 config.rs  -rw-r--r--  6.8 KiB  2024-11-01 14:28
└── 📁 docs  drwxr-xr-x  2024-11-01 14:30
    └── 📝 README.md  -rw-r--r--  4.2 KiB  2024-11-01 14:27
────────────────────────────────────────────────────────────
Total: 8 entries total
```

Use `--ascii` for ASCII art connectors (`|-- ` / `` `-- ``) instead of Unicode box-drawing.

---

## ⚙️ Configuration

### Config file search order

`tar` searches for config in this order (first found wins):

**Linux:**
1. `$XDG_CONFIG_HOME/tar/tar.toml`
2. `~/.config/tar/tar.toml`
3. `~/tar.toml` / `~/.tar.toml`
4. `/etc/tar/tar.toml`
5. Current working directory
6. Same directory as the executable

**macOS:**
1. `~/Library/Application Support/tar/tar.toml`
2. `~/.config/tar/tar.toml`
3. `~/tar.toml` / `~/.tar.toml`
4. Current working directory / exe directory

**Windows:**
1. `%APPDATA%\tar\tar.toml`
2. `%USERPROFILE%\tar.toml`
3. Current working directory / exe directory

Both `.toml` and `.json` formats are supported, with and without a leading dot.

### Config commands

```bash
tar2 config list              # Show all keys and current values
tar2 config get colors.dir    # Get a single value
tar2 config set colors.dir '#FF6600'   # Set a value (saves immediately)
tar2 config set display.emoji false    # Disable emoji
tar2 config paths             # Show all searched paths
tar2 config which             # Show active config file
tar2 config reset             # Reset to defaults
tar2 config edit              # Open config in $EDITOR
```

### Full config reference

```toml
[colors]
dir    = "#00BFFF"    # Directory entries
file   = "white"      # Regular files
link   = "#FF69B4"    # Symlinks
exec   = "#00FF7F"    # Executables
header = "#FFD700"    # Archive name / headers
size   = "#00FFFF"    # File sizes
date   = "#DDA0DD"    # Modification times
perm   = "#FFA500"    # Permission strings
tree   = "#808080"    # Tree branch characters
warn   = "#FFFF00"    # Warnings
error  = "#FF4444"    # Errors
ok     = "#44FF44"    # Success messages

[emojis]
dir     = "📁"
file    = "📄"
link    = "🔗"
archive = "📦"
image   = "🖼️"
video   = "🎬"
audio   = "🎵"
doc     = "📝"
code    = "💻"
ok      = "✅"
warn    = "⚠️"
error   = "❌"

[display]
emoji           = true         # Show emoji icons
colors          = true         # Enable colored output
tree_depth      = 0            # Default tree depth (0 = unlimited)
human_readable  = true         # Human-readable sizes (KiB, MiB...)
verbose         = false        # Verbose by default
progress_style  = "bar"        # "bar" | "spinner" | "none"
date_format     = "%Y-%m-%d %H:%M"
tree_style      = "unicode"    # "unicode" | "ascii"
```

Colors accept:
- **Hex:** `#RRGGBB` (e.g. `#00FFFF`, `#FF6600`)
- **Named:** `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, and `bright_*` variants

---

## 🎯 Flag Reference

### Operations
| Flag | Description |
|------|-------------|
| `-c` | Create archive |
| `-x` | Extract archive |
| `-t` | List contents |
| `-r` | Append files |
| `-u` | Update (newer files only) |
| `-W`, `--test` | Verify/test integrity |

### Compression
| Flag | Format |
|------|--------|
| `-z` | gzip (`.tar.gz`, `.tgz`) |
| `-j` | bzip2 (`.tar.bz2`, `.tbz2`) |
| `-J` | xz (`.tar.xz`, `.txz`) |
| `--zstd` | zstd (`.tar.zst`) |
| `--compression-level N` | Override level (1–9 / 1–22 for zstd) |

Compression is **auto-detected** from the archive filename if no flag is given.

### Filtering
| Flag | Description |
|------|-------------|
| `--include PATTERN` | Include only matching files (glob, repeatable) |
| `--exclude PATTERN` | Exclude matching files (glob, repeatable) |
| `--include-regex RE` | Include by regex |
| `--exclude-regex RE` | Exclude by regex |
| `--exclude-from FILE` | Read exclude patterns from file |
| `--ignore-file FILE` | Load exclusion patterns from an ignore file (gitignore syntax, repeatable) |
| `--no-auto-ignore` | Disable auto-detection of standard ignore files |
| `--keep-ignore-files` | Include the loaded ignore file(s) inside the packed archive (default: left out) |

### 🙈 Ignore files

`tar2` understands `.gitignore`-style pattern files and applies them automatically
as extra exclusion rules — for **create/pack (`-c`)**, **extract (`-x`)**,
**list (`-t`)**, **append/update (`-r`/`-u`)**, and the **`tree`** subcommand.

On each run, `tar2` looks directly inside the current directory (and, for
`-c`/`-r`/`-u`, inside each source directory) for any of the following, in
priority order, and loads every one it finds:

```
.tar2ignore  .tarignore  .gitignore  .dockerignore  .npmignore  .hgignore  .ignore
```

All of them use the same pattern syntax as `.gitignore` — including `#` comments,
`!negation`, and `dir/`-only rules. When one or more is found, `tar2` prints
which files it used:

```
Using ignore rules from: /path/to/.gitignore, /path/to/.dockerignore
```

To turn this off entirely, pass `--no-auto-ignore`. To load extra patterns from
a file that isn't one of the standard names, pass `--ignore-file path/to/file`
(repeatable). Ignore rules combine with `--exclude`/`--include` — an ignored
path is dropped regardless of other filters.

**Scope:** ignore files are only ever read from disk, never from inside an
archive. For `-c`/`-r`/`-u` (pack/append/update) they're read from your
current directory and directly inside each source directory you're packing.
For `-x`/`-t`/`tree` (extract/list/tree) they're read from your current
directory only — a `.gitignore` that happens to be *inside* the archive you're
extracting is not consulted; it extracts like any other member.

**The ignore file itself is left out of the pack.** Once `.gitignore` (or
whichever file matched) has done its job filtering, it's excluded from the
resulting archive by default — the same way `npm pack` never ships
`.npmignore`/`.gitignore` in the published tarball. If you want it included
anyway (e.g. the archive will later be unpacked and re-filtered by plain
`tar`, 7-Zip, WinRAR, or some other tool that should see the same rules),
pass `--keep-ignore-files`.

```bash
# Pack a project dir, respecting its .gitignore/.dockerignore automatically
tar -czf project.tar.gz project/

# Ignore rules also apply when listing or extracting
tar -tf project.tar.gz
tar -xf project.tar.gz -C /tmp/out

# Use a one-off ignore file instead of auto-detected ones
tar -czf project.tar.gz --no-auto-ignore --ignore-file .buildignore project/
```

### Other options
| Flag | Description |
|------|-------------|
| `-v` | Verbose (repeat for more: `-vv`) |
| `-C DIR` | Change to DIR before operating |
| `-p` | Preserve permissions |
| `-L` | Dereference symlinks |
| `--numeric-owner` | Use numeric UID/GID |
| `--strip-components N` | Strip N path components on extract |
| `--transform EXPR` | Filename transform (e.g. `s/old/new/`) |
| `--no-color` | Disable colors |
| `--no-emoji` | Disable emoji |
| `--config FILE` | Use a specific config file |
| `--progress` | Show progress (default: on) |

---

## 📦 Prebuilt Binaries

Every tagged release publishes binaries for a broad platform matrix — see the
[Releases page](https://github.com/cumulus13/tar2/releases). Filenames follow
`tar2_<version>_<platform>`:

| Platform | Suffix |
|----------|--------|
| Linux amd64 / 386 | `linux_amd64`, `linux_386` |
| Linux ARM (v6, v7, arm64) | `linux_armv6`, `linux_armv7`, `linux_arm64` |
| Android (Termux / NetHunter, armv7/arm64) | `android_armv7`, `android_arm64` |
| Linux MIPS family — best-effort, Tier 3 upstream | `linux_mips`, `linux_mipsle`, `linux_mips64`, `linux_mips64le` |
| Linux ppc64le / riscv64 / s390x | `linux_ppc64le`, `linux_riscv64`, `linux_s390x` |
| macOS Intel / Apple Silicon | `darwin_amd64`, `darwin_arm64` |
| Windows amd64 / 386 / arm64 | `windows_amd64`, `windows_386`, `windows_arm64` |

Verify a download against the release's `checksums.txt` with `sha256sum -c`.

---

## 🔨 Building

```bash
git clone https://github.com/cumulus13/tar2
cd tar2
cargo build --release
# Binary at: target/release/tar2
```

**Requirements:** Rust 1.75+

Install globally:
```bash
cargo install --path .
# or
cp target/release/tar2 ~/.local/bin/tar2
```

---

## 🌍 Environment Variables

| Variable | Effect |
|----------|--------|
| `NO_COLOR` | Disable all colors (standard) |
| `EDITOR` / `VISUAL` | Editor for `tar2 config edit` |

---

## 📄 License

MIT — see [LICENSE](LICENSE)

---

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)
