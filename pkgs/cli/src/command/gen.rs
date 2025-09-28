use crate::prelude::*;
use anyhow::{anyhow, bail, Context};
use nmcr_md_parser::prelude::*;
use nmcr_types::{Location, OutputFile, OutputTree, Template, TemplateFile, TemplateTree};
use nmcr_types_internal::FormattedLocation;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf as FsPathBuf;

#[derive(Args, Debug)]
pub struct GenArgs {
    /// Template id to generate (file or tree)
    #[arg(index = 1)]
    pub id: String,

    /// Destination directory to write into; omit to use --print
    #[arg(index = 2)]
    pub dest: Option<FsPathBuf>,

    /// Print structured JSON instead of writing files
    #[arg(long)]
    pub print: bool,

    /// Template arguments in key=value form
    #[arg(long = "arg", value_parser = parse_kv, num_args=0.., value_name="key=value")]
    pub args: Vec<(String, String)>,
}

#[derive(Args)]
pub struct GenCmd {}

impl GenCmd {
    pub async fn run(args: &CliCommandProject<GenArgs>) -> Result<()> {
        let project = args.load_project()?;
        let paths = project.template_paths()?;

        // Collect templates and trees
        let mut files: HashMap<String, TemplateFile> = HashMap::new();
        let mut trees: HashMap<String, TemplateTree> = HashMap::new();
        let mut ids_seen: HashMap<String, Location> = HashMap::new();

        for p in &paths {
            match parse_file(p) {
                Ok(ParsedMarkdown::Template(t)) => match t {
                    Template::TemplateFile(f) => {
                        register_id(&f.id, &f.location, &mut ids_seen)?;
                        files.insert(f.id.clone(), f);
                    }
                    Template::TemplateTree(tr) => {
                        for tf in &tr.files {
                            if let Template::TemplateFile(f) = tf {
                                register_id(&f.id, &f.location, &mut ids_seen)?;
                            }
                        }
                        trees.insert(tr.id.clone(), tr);
                    }
                },
                Ok(ParsedMarkdown::Tree(tree)) => {
                    for t in &tree.files {
                        if let Template::TemplateFile(f) = t {
                            register_id(&f.id, &f.location, &mut ids_seen)?;
                        }
                    }
                    trees.insert(tree.id.clone(), tree);
                }
                Ok(ParsedMarkdown::Collection(c)) => {
                    for t in c.templates {
                        match t {
                            Template::TemplateFile(f) => {
                                register_id(&f.id, &f.location, &mut ids_seen)?;
                                files.insert(f.id.clone(), f);
                            }
                            Template::TemplateTree(tr) => {
                                // Do not re-register files inside the tree; they appear above as TemplateFile entries.
                                trees.insert(tr.id.clone(), tr);
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("! Failed to parse {}: {err}", p.display());
                }
            }
        }

        let id = &args.local.id;
        let dest = args.local.dest.clone();
        let print = args.local.print;
        let args_map: serde_json::Map<String, serde_json::Value> = args
            .local
            .args
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        // Match id and execute
        if let Some(t) = files.get(id) {
            if print {
                let out = OutputFile {
                    path: t.path.clone(),
                    lang: t.lang.clone(),
                    content: render(&t.content, &args_map),
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
                return Ok(());
            }

            let root = match dest {
                Some(d) => d,
                None => bail!("--print not set and no destination provided. Pass a directory path or use --print."),
            };

            // Require path for file writes
            let rel = match &t.path {
                Some(p) => p,
                None => bail!("Template '{}' has no path; supply --print to inspect output or select a tree template.", t.id),
            };
            let target = root.join(rel);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, render(&t.content, &args_map))?;
            println!("Wrote {}", target.display());
            return Ok(());
        }

        if let Some(tree) = trees.get(id) {
            if print {
                let files_out: Vec<OutputFile> = tree
                    .files
                    .iter()
                    .filter_map(|t| match t { Template::TemplateFile(f) => Some(f), _ => None })
                    .map(|f| OutputFile { path: f.path.clone(), lang: f.lang.clone(), content: render(&f.content, &args_map) })
                    .collect();
                let out = OutputTree { files: files_out };
                println!("{}", serde_json::to_string_pretty(&out)?);
                return Ok(());
            }

            let root = match dest {
                Some(d) => d,
                None => bail!("--print not set and no destination provided. Pass a directory path or use --print."),
            };
            if !root.exists() {
                fs::create_dir_all(&root)?;
            }
            if !root.is_dir() {
                bail!("Destination '{}' is not a directory.", root.display());
            }
            // Validate all files have paths
            for t in &tree.files {
                if let Template::TemplateFile(f) = t
                    && f.path.is_none() {
                        bail!("Tree '{}' contains a file without a path (file id '{}').", tree.id, f.id);
                    }
            }
            for t in &tree.files {
                let f = match t { Template::TemplateFile(f) => f, _ => continue };
                let rel = f.path.as_ref().unwrap();
                let target = root.join(rel);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&target, render(&f.content, &args_map))?;
                println!("Wrote {}", target.display());
            }
            return Ok(());
        }

        let mut all: Vec<String> = files.keys().cloned().collect();
        all.extend(trees.keys().cloned());
        all.sort();
        bail!("Template id '{}' not found. Available: {}", id, all.join(", "))
    }
}

fn register_id(id: &str, location: &Location, seen: &mut HashMap<String, Location>) -> Result<()> {
    if let Some(existing) = seen.get(id) {
        let first = FormattedLocation(existing).to_string();
        let duplicate = FormattedLocation(location).to_string();
        return Err(anyhow!("Duplicate template id: {}", id))
            .with_context(|| format!("duplicate occurrence at {duplicate}"))
            .with_context(|| format!("first occurrence at {first}"));
    }

    seen.insert(id.to_string(), location.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nmcr_types::Span;
    fn make_loc(path: &str, start: usize, end: usize) -> Location {
        Location { path: path.to_string(), span: Span { start, end } }
    }

    #[test]
    fn register_template_detects_duplicates() {
        let mut seen = HashMap::new();
        let first = make_loc("templates/a.md", 0, 10);
        register_id("duplicate", &first, &mut seen).expect("first registration succeeds");

        let second = make_loc("templates/b.md", 5, 15);
        let err = register_id("duplicate", &second, &mut seen).expect_err("duplicate should fail");

        let rendered = format!("{err:?}");
        assert!(rendered.contains("Duplicate template id: duplicate"));
        assert!(rendered.contains("duplicate occurrence at templates/b.md:5-15"));
        assert!(rendered.contains("first occurrence at templates/a.md:0-10"));
    }
}

fn parse_kv(s: &str) -> Result<(String, String)> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid arg, expected key=value: {}", s))?;
    Ok((k.to_string(), v.to_string()))
}

fn render(content: &str, _args: &serde_json::Map<String, serde_json::Value>) -> String {
    // Placeholder: passthrough until a real engine is wired
    content.to_string()
}
