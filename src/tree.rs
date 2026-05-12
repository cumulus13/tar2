use std::collections::BTreeMap;
use std::path::Component;

use crate::archive::EntryInfo;
use crate::colors::{Painter, get_emoji};
use crate::config::Config;

// ─── Tree Node ────────────────────────────────────────────────────────────────

pub struct TreeNode {
    name: String,
    info: Option<EntryInfo>,
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), info: None, children: BTreeMap::new() }
    }

    fn insert(&mut self, components: &[&str], info: EntryInfo) {
        if components.is_empty() {
            self.info = Some(info);
            return;
        }
        let child = self.children
            .entry(components[0].to_string())
            .or_insert_with(|| TreeNode::new(components[0]));
        child.insert(&components[1..], info);
    }
}

// ─── Build Tree ───────────────────────────────────────────────────────────────

pub fn build_tree(entries: &[EntryInfo]) -> TreeNode {
    let mut root = TreeNode::new("/");
    for entry in entries {
        let parts: Vec<&str> = entry.path.components()
            .filter_map(|c| match c {
                Component::Normal(s) => s.to_str(),
                _ => None,
            })
            .collect();
        if !parts.is_empty() {
            root.insert(&parts, entry.clone());
        }
    }
    root
}

// ─── Render Tree ─────────────────────────────────────────────────────────────

pub struct TreeOptions<'a> {
    pub max_depth: u32,
    pub show_size: bool,
    pub show_date: bool,
    pub show_perm: bool,
    pub date_fmt: &'a str,
    pub unicode: bool,
}

// Internal render context — avoids too-many-arguments clippy lint
struct RenderCtx<'a> {
    opts: &'a TreeOptions<'a>,
    painter: &'a Painter,
    config: &'a Config,
}

const BRANCH:   &str = "├── ";
const LAST:     &str = "└── ";
const PIPE:     &str = "│   ";
const SPACE:    &str = "    ";
const BRANCH_A: &str = "|-- ";
const LAST_A:   &str = "`-- ";
const PIPE_A:   &str = "|   ";

pub fn render_tree(root: &TreeNode, opts: &TreeOptions, config: &Config) -> String {
    let painter = Painter::new(config, config.display.colors);
    let ctx = RenderCtx { opts, painter: &painter, config };
    let mut out = String::new();
    render_node(root, "", true, 0, &ctx, &mut out);
    out
}

fn render_node(
    node: &TreeNode,
    prefix: &str,
    is_last: bool,
    depth: u32,
    ctx: &RenderCtx,
    out: &mut String,
) {
    let (connector, child_ext) = if ctx.opts.unicode {
        (if is_last { LAST } else { BRANCH }, if is_last { SPACE } else { PIPE })
    } else {
        (if is_last { LAST_A } else { BRANCH_A }, if is_last { SPACE } else { PIPE_A })
    };

    if depth > 0 {
        let tree_part = format!("{}{}", prefix, connector);
        let colored_tree = ctx.painter.tree(&tree_part).to_string();

        let is_dir  = node.info.as_ref().map(|i| i.is_dir).unwrap_or(!node.children.is_empty());
        let is_link = node.info.as_ref().map(|i| i.is_link).unwrap_or(false);

        let emoji = if ctx.config.display.emoji {
            format!("{} ", get_emoji(&node.name, is_dir, is_link, ctx.config))
        } else {
            String::new()
        };

        let name_str = if is_dir {
            ctx.painter.dir(&node.name).to_string()
        } else if is_link {
            let base = ctx.painter.link(&node.name).to_string();
            if let Some(ref info) = node.info {
                if let Some(ref target) = info.link_target {
                    format!("{} -> {}", base, ctx.painter.link(&target.display().to_string()))
                } else { base }
            } else { base }
        } else {
            ctx.painter.file(&node.name).to_string()
        };

        let mut meta = String::new();
        if let Some(ref info) = node.info {
            if ctx.opts.show_perm {
                meta.push_str(&format!("  {}", ctx.painter.perm(&info.perm_str())));
            }
            if ctx.opts.show_size && !is_dir {
                meta.push_str(&format!("  {}", ctx.painter.size(&info.size_human())));
            }
            if ctx.opts.show_date {
                meta.push_str(&format!("  {}", ctx.painter.date(&info.mtime_str(ctx.opts.date_fmt))));
            }
        }

        out.push_str(&format!("{}{}{}{}\n", colored_tree, emoji, name_str, meta));
    }

    // Recurse
    if ctx.opts.max_depth == 0 || depth < ctx.opts.max_depth {
        let children: Vec<_> = node.children.values().collect();
        let n = children.len();
        let new_prefix = if depth == 0 {
            String::new()
        } else {
            format!("{}{}", prefix, child_ext)
        };
        for (i, child) in children.iter().enumerate() {
            render_node(child, &new_prefix, i == n - 1, depth + 1, ctx, out);
        }
    }
}

// ─── Flat list ────────────────────────────────────────────────────────────────

pub fn render_flat(entries: &[EntryInfo], verbose: bool, config: &Config) {
    let painter = Painter::new(config, config.display.colors);
    let date_fmt = &config.display.date_format;

    for entry in entries {
        let is_dir  = entry.is_dir;
        let is_link = entry.is_link;

        let emoji = if config.display.emoji {
            format!("{} ", get_emoji(
                &entry.path.file_name().unwrap_or_default().to_string_lossy(),
                is_dir, is_link, config,
            ))
        } else {
            String::new()
        };

        let name = entry.path.display().to_string();
        let name_colored = if is_dir {
            painter.dir(&name).to_string()
        } else if is_link {
            painter.link(&name).to_string()
        } else {
            painter.file(&name).to_string()
        };

        if verbose {
            let perm = painter.perm(&entry.perm_str()).to_string();
            let size = painter.size(&format!("{:>10}", entry.size_human())).to_string();
            let date = painter.date(&entry.mtime_str(date_fmt)).to_string();
            println!("{} {} {}  {}{}", perm, size, date, emoji, name_colored);
        } else {
            println!("{}{}", emoji, name_colored);
        }
    }
}
