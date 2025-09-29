use crate::prelude::*;
use nmcr_catalog::{CatalogTree, TemplateCatalog};
use nmcr_types::{Arg, ArgKind, Location, TemplateFile};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const CONTENT_OFFSET: usize = 2;

fn spaces(count: usize) -> String {
    " ".repeat(count)
}

#[derive(Args, Debug)]
pub struct ListArgs {}

#[derive(Args)]
pub struct ListCmd {}

impl ListCmd {
    pub async fn run(args: &CliCommandProject<ListArgs>) -> Result<()> {
        let project = args.load_project()?;
        let project_root = project.path();
        let paths = project.template_paths()?;
        let catalog = TemplateCatalog::load(&paths)?;

        if catalog.is_empty() {
            println!("No templates found.");
            return Ok(());
        }

        render_catalog(&catalog, &project_root)
    }
}

fn render_catalog(catalog: &TemplateCatalog, project_root: &Path) -> Result<()> {
    let mut out = io::stdout().lock();
    render_catalog_to_writer(catalog, project_root, &mut out)?;
    Ok(())
}

fn render_catalog_to_writer<W: Write>(
    catalog: &TemplateCatalog,
    project_root: &Path,
    out: &mut W,
) -> io::Result<()> {
    let mut resolver = LocationResolver::new(project_root.to_path_buf());

    let mut entries: Vec<RootEntry<'_>> = Vec::new();
    entries.extend(catalog.tree_templates().iter().map(RootEntry::Tree));
    entries.extend(catalog.standalone_files().iter().map(RootEntry::File));

    for (index, entry) in entries.into_iter().enumerate() {
        if index > 0 {
            writeln!(out)?;
            writeln!(out)?;
        }

        match entry {
            RootEntry::Tree(tree) => render_tree(out, tree, &mut resolver)?,
            RootEntry::File(file) => render_file_entry(out, file, &mut resolver, 0)?,
        }
    }

    Ok(())
}

#[derive(Copy, Clone)]
enum RootEntry<'a> {
    Tree(&'a CatalogTree),
    File(&'a TemplateFile),
}

fn render_tree<W: Write>(
    out: &mut W,
    tree: &CatalogTree,
    resolver: &mut LocationResolver,
) -> io::Result<()> {
    let location = format_location(resolver, tree.location());
    writeln!(out, "📁 {} ({})", tree.id(), location)?;

    let base_indent = 1;
    let content_indent = spaces(base_indent + CONTENT_OFFSET);
    writeln!(out, "{}->", content_indent)?;

    let _ = print_tree_structure(out, tree, base_indent + CONTENT_OFFSET, resolver)?;
    writeln!(out)?;

    let description = clean_description(tree.description());
    let has_description = description
        .as_ref()
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false);

    if has_description {
        write_description_block(
            out,
            description.as_ref().unwrap(),
            base_indent + CONTENT_OFFSET,
        )?;
        writeln!(out)?;
    }

    if tree.files().is_empty() {
        writeln!(out, "{}Files: (none)", content_indent)?;
    } else {
        writeln!(out, "{}Files:", content_indent)?;
        writeln!(out)?;
        let child_indent = base_indent + CONTENT_OFFSET;
        for (idx, file) in tree.files().iter().enumerate() {
            render_file_entry(out, file, resolver, child_indent)?;
            if idx + 1 < tree.files().len() {
                writeln!(out)?;
            }
        }
    }

    Ok(())
}

fn render_file_entry<W: Write>(
    out: &mut W,
    file: &TemplateFile,
    resolver: &mut LocationResolver,
    base_indent: usize,
) -> io::Result<()> {
    let indent = spaces(base_indent);
    let location = format_location(resolver, &file.location);
    writeln!(out, "{}📄 {} ({})", indent, file.id, location)?;

    let detail_indent = spaces(base_indent + CONTENT_OFFSET + 1);
    let path_display = match file.path.as_deref() {
        Some(path) => format_path(path),
        None => "[string]".to_string(),
    };
    let mut arrow_line = format!("{}-> {}", detail_indent, path_display);
    if let Some(lang) = detect_language(file) {
        arrow_line.push_str(&format!(" ({})", lang));
    }
    writeln!(out, "{}", arrow_line)?;

    let description = clean_description(&file.description);
    let has_description = description
        .as_ref()
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false);

    let has_args = !file.args.is_empty();

    if has_description || has_args {
        writeln!(out)?;
    }

    if let Some(desc) = description {
        if has_description {
            write_description_block(out, desc.as_str(), base_indent + CONTENT_OFFSET + 1)?;
        }
        if has_args {
            writeln!(out)?;
        }
    }

    if has_args {
        write_arguments_block(out, &file.args, base_indent + CONTENT_OFFSET + 1)?;
    }

    Ok(())
}

fn print_tree_structure<W: Write>(
    out: &mut W,
    tree: &CatalogTree,
    indent_width: usize,
    resolver: &mut LocationResolver,
) -> io::Result<bool> {
    let path_tree = build_path_tree(tree);
    if path_tree.is_empty() {
        return Ok(false);
    }

    let prefix = spaces(indent_width);
    write_path_node(out, &path_tree, &prefix, resolver)
}

fn write_path_node<W: Write>(
    out: &mut W,
    node: &PathNode,
    prefix: &str,
    resolver: &mut LocationResolver,
) -> io::Result<bool> {
    let mut has_output = false;
    let total = node.entries.len();

    for (idx, entry) in node.entries.iter().enumerate() {
        let is_last = idx + 1 == total;
        let connector = if is_last { "└──" } else { "├──" };
        match entry {
            PathEntry::Dir { name, node: child } => {
                writeln!(out, "{}{} {}", prefix, connector, name)?;
                let next_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                has_output = true;
                write_path_node(out, child, &next_prefix, resolver)?;
            }
            PathEntry::File { name, location } => {
                let loc = format_location(resolver, location);
                writeln!(out, "{}{} {} ({})", prefix, connector, name, loc)?;
                has_output = true;
            }
        }
    }

    Ok(has_output)
}

fn build_path_tree(tree: &CatalogTree) -> PathNode {
    let mut root = PathNode::default();
    for file in tree.files() {
        if let Some(path) = file.path.as_deref() {
            let normalized = path.replace('\\', "/");
            let trimmed = normalized
                .trim_start_matches("./")
                .trim_start_matches(".\\");
            let segments: Vec<&str> = trimmed
                .split('/')
                .filter(|segment| !segment.is_empty())
                .collect();
            if !segments.is_empty() {
                root.insert(&segments, &file.location);
            }
        }
    }
    root
}

fn write_description_block<W: Write>(
    out: &mut W,
    description: &str,
    indent: usize,
) -> io::Result<()> {
    let prefix = " ".repeat(indent);
    for line in description.split('\n') {
        if line.is_empty() {
            writeln!(out)?;
        } else {
            writeln!(out, "{}{}", prefix, line)?;
        }
    }
    Ok(())
}

fn write_arguments_block<W: Write>(out: &mut W, args: &[Arg], indent: usize) -> io::Result<()> {
    let prefix = " ".repeat(indent);
    writeln!(out, "{}Arguments:", prefix)?;
    writeln!(out)?;
    for arg in args {
        writeln!(out, "{}- {}", prefix, format_argument(arg))?;
    }
    Ok(())
}

fn format_argument(arg: &Arg) -> String {
    let arg_type = match &arg.kind {
        ArgKind::Boolean(_) => "boolean",
        ArgKind::String(_) => "string",
        ArgKind::Number(_) => "number",
        ArgKind::Any(_) => "string",
    };

    if arg.description.trim().is_empty() {
        format!("{} [{}]", arg.name, arg_type)
    } else {
        format!("{} [{}]: {}", arg.name, arg_type, arg.description.trim())
    }
}

fn clean_description(input: &str) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut previous_blank = true;

    for raw_line in input.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            if !previous_blank {
                lines.push(String::new());
                previous_blank = true;
            }
            continue;
        }

        if is_inline_path_marker(trimmed) {
            continue;
        }

        lines.push(trimmed.to_string());
        previous_blank = false;
    }

    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn is_inline_path_marker(line: &str) -> bool {
    if line.starts_with('`') && line.ends_with("`:") {
        return true;
    }

    if line.ends_with(':') {
        let stem = &line[..line.len().saturating_sub(1)];
        if !stem.contains(' ') {
            return true;
        }
    }

    false
}

fn format_path(path: &str) -> String {
    if path.is_empty() {
        "[no path]".to_string()
    } else {
        path.to_string()
    }
}

fn detect_language(file: &TemplateFile) -> Option<String> {
    if let Some(path) = file.path.as_deref() {
        let normalized = path.trim_start_matches("./").trim_start_matches(".\\");
        if let Some(ext) = Path::new(normalized).extension().and_then(|s| s.to_str())
            && !ext.is_empty()
        {
            return Some(ext.to_string());
        }
    }

    file.lang.as_deref().and_then(|lang| {
        let trimmed = lang.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn format_location(resolver: &mut LocationResolver, location: &Location) -> String {
    let path = resolver.display_path(&location.path);

    match resolver.line(location) {
        Some(line) => format!("{}:{}", path, line),
        None => path,
    }
}

#[derive(Default)]
struct PathNode {
    entries: Vec<PathEntry>,
}

enum PathEntry {
    Dir { name: String, node: Box<PathNode> },
    File { name: String, location: Location },
}

impl PathNode {
    fn insert(&mut self, segments: &[&str], location: &Location) {
        if segments.is_empty() {
            return;
        }

        if segments.len() == 1 {
            let name = segments[0];
            if let Some(PathEntry::File { location: existing, .. }) = self
                .entries
                .iter_mut()
                .find(|entry| matches!(entry, PathEntry::File { name: entry_name, .. } if entry_name == name))
            {
                *existing = location.clone();
            } else {
                self.entries.push(PathEntry::File {
                    name: name.to_string(),
                    location: location.clone(),
                });
            }
        } else {
            let (head, tail) = segments.split_first().expect("segments non-empty");
            if let Some(PathEntry::Dir { node, .. }) = self
                .entries
                .iter_mut()
                .find(|entry| matches!(entry, PathEntry::Dir { name, .. } if name == head))
            {
                node.insert(tail, location);
            } else {
                let mut child = PathNode::default();
                child.insert(tail, location);
                self.entries.push(PathEntry::Dir {
                    name: head.to_string(),
                    node: Box::new(child),
                });
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

struct LocationResolver {
    root: PathBuf,
    root_parent: Option<PathBuf>,
    cwd: PathBuf,
    cache: HashMap<String, Option<Vec<usize>>>,
}

impl LocationResolver {
    fn new(root: PathBuf) -> Self {
        let canonical_root = root.canonicalize().unwrap_or(root);
        let cwd = std::env::current_dir().unwrap_or_else(|_| canonical_root.clone());
        let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
        let parent = canonical_root.parent().map(|p| p.to_path_buf());
        Self {
            root: canonical_root,
            root_parent: parent,
            cwd: canonical_cwd,
            cache: HashMap::new(),
        }
    }

    fn line(&mut self, location: &Location) -> Option<usize> {
        if location.path.is_empty() {
            return None;
        }

        let path_key = location.path.clone();
        if !self.cache.contains_key(&path_key) {
            let offsets = self.compute_offsets(&path_key);
            self.cache.insert(path_key.clone(), offsets);
        }

        self.cache
            .get(&path_key)
            .and_then(|offsets| offsets.as_ref())
            .map(|offsets| {
                let position = location.span.start;
                let line_index = offsets.partition_point(|offset| *offset <= position);
                // partition_point returns number of entries <= position, which is already 1-based
                line_index.max(1)
            })
    }

    fn compute_offsets(&self, relative: &str) -> Option<Vec<usize>> {
        let absolute = self.resolve_path(relative)?;

        let content = std::fs::read_to_string(&absolute).ok()?;
        let mut offsets = vec![0usize];
        for (idx, ch) in content.char_indices() {
            if ch == '\n' {
                offsets.push(idx + 1);
            }
        }
        Some(offsets)
    }

    fn display_path(&self, path: &str) -> String {
        if path.is_empty() {
            return "<memory>".to_string();
        }

        let normalized = path.replace('\\', "/");
        if normalized.starts_with('/') {
            let candidate = Path::new(&normalized);
            let canonical = candidate
                .canonicalize()
                .unwrap_or_else(|_| candidate.to_path_buf());
            if let Ok(relative) = canonical.strip_prefix(&self.root) {
                let rel_str = relative.to_string_lossy();
                if rel_str.is_empty() {
                    return "./".to_string();
                }
                return format!("./{}", rel_str);
            }
            if let Some(parent) = &self.root_parent
                && let Ok(relative) = canonical.strip_prefix(parent)
            {
                let rel_str = relative.to_string_lossy();
                if rel_str.is_empty() {
                    return "./".to_string();
                }
                return format!("./{}", rel_str);
            }
            if let Ok(relative) = canonical.strip_prefix(&self.cwd) {
                let rel_str = relative.to_string_lossy();
                if rel_str.is_empty() {
                    "./".to_string()
                } else {
                    format!("./{}", rel_str)
                }
            } else {
                canonical.to_string_lossy().to_string()
            }
        } else if normalized.starts_with("./") || normalized.starts_with("../") {
            normalized
        } else {
            format!("./{}", normalized)
        }
    }

    fn resolve_path(&self, raw: &str) -> Option<PathBuf> {
        if raw.is_empty() {
            return None;
        }

        let normalized = raw.replace('\\', "/");
        let candidate = Path::new(&normalized);
        if candidate.is_absolute() {
            return Some(candidate.to_path_buf());
        }

        let mut segments: Vec<&str> = normalized.split('/').collect();
        if segments.is_empty() {
            return Some(self.root.join(normalized));
        }

        if self.root.join(&normalized).is_file() {
            return Some(self.root.join(&normalized));
        }

        while !segments.is_empty() {
            let attempt = segments.join("/");
            let joined = self.root.join(&attempt);
            if joined.is_file() {
                return Some(joined);
            }
            segments.remove(0);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use std::path::PathBuf;

    #[test]
    fn list_output_snapshot() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = root.join("../../");
        let paths = vec![
            root.join("../../examples/basic/tmpls/rust.md"),
            root.join("../../examples/basic/tmpls/rust-crate.md"),
            root.join("../../examples/basic/tmpls/react.md"),
        ];
        let catalog = TemplateCatalog::load(&paths).expect("catalog loads");
        let mut buffer: Vec<u8> = Vec::new();
        render_catalog_to_writer(&catalog, &workspace_root, &mut buffer).expect("render succeeds");

        let output = String::from_utf8(buffer).expect("utf8 output");
        assert_snapshot!(output, @r###"
📁 rust (./examples/basic/tmpls/rust.md:1)
   ->
   ├── Cargo.toml (./examples/basic/tmpls/rust.md:3)
   └── .gitignore (./examples/basic/tmpls/rust.md:20)

   Files:

   📄 rust_package_cargo_toml (./examples/basic/tmpls/rust.md:3)
      -> Cargo.toml (toml)

      Crate manifest file.

      Arguments:

      - description [string]
      - name [string]

   📄 rust_package_gitignore (./examples/basic/tmpls/rust.md:20)
      -> .gitignore

      Rust crate .gitignore file.


📁 rust_crate_lib (./examples/basic/tmpls/rust-crate.md:3)
   ->
   ├── Cargo.toml (./examples/basic/tmpls/rust-crate.md:7)
   └── src
       └── lib.rs (./examples/basic/tmpls/rust-crate.md:18)

   Rust crate library template.

   Files:

   📄 rust_crate_lib_cargo_toml (./examples/basic/tmpls/rust-crate.md:7)
      -> ./Cargo.toml (toml)

      Arguments:

      - pkg_name [string]

   📄 rust_crate_lib_src_lib_rs (./examples/basic/tmpls/rust-crate.md:18)
      -> ./src/lib.rs (rs)


📄 react_react_component (./examples/basic/tmpls/react.md:5)
   -> [string] (tsx)

   React component file template with optional props.

   Arguments:

   - props [boolean]: Include a props interface scaffold.
   - name [string]: Component name.
"###);
    }
}
