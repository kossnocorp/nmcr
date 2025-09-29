use crate::prelude::*;
use anyhow::{Context, anyhow, bail};
use nmcr_md_parser::prelude::*;
use nmcr_template::TemplateRenderer;
use nmcr_types::{Location, OutputFile, OutputTree, Template, TemplateFile, TemplateTree};
use nmcr_types_internal::FormattedLocation;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf as FsPathBuf;

#[derive(Args, Debug)]
pub struct GenArgs {
    /// Template id to generate (file or tree)
    #[arg(index = 1)]
    pub id: String,

    /// Output directory to write into; omit to use --print or supply as the first positional argument
    #[arg(long, value_name = "path")]
    pub out: Option<FsPathBuf>,

    /// Print the rendered output instead of writing files (trees are emitted as JSON)
    #[arg(long)]
    pub print: bool,

    /// Template arguments in key=value form
    #[arg(index = 2, value_name = "key=value", num_args = 0.., allow_hyphen_values = true)]
    pub pairs: Vec<String>,
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
        let print = args.local.print;

        let mut positional_pairs = args.local.pairs.clone();
        let positional_out = extract_positional_out(&mut positional_pairs);

        if args.local.out.is_some() && positional_out.is_some() {
            bail!(
                "Output path provided both with --out and as a positional argument; use one or the other."
            );
        }

        let out_dir = args.local.out.clone().or(positional_out);
        let args_map = build_context_map(&positional_pairs)?;
        let renderer = TemplateRenderer::new();

        // Match id and execute
        if let Some(t) = files.get(id) {
            let rendered = render_template_file(&renderer, t, &args_map)?;
            if print {
                print!("{}", rendered.content);
                io::stdout().flush()?;
                return Ok(());
            }

            let root = out_dir.clone().ok_or_else(|| {
                anyhow!(
                    "--print not set and no output path provided. Pass --out <path> or supply a positional path."
                )
            })?;

            // Require path for file writes
            let rel = rendered.path.as_ref().ok_or_else(|| {
                anyhow!(
                    "Template '{}' has no path; supply --print to inspect output or select a tree template.",
                    t.id
                )
            })?;
            let target = root.join(rel);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, rendered.content)?;
            println!("Wrote {}", target.display());
            return Ok(());
        }

        if let Some(tree) = trees.get(id) {
            if print {
                let files_out: Vec<OutputFile> = tree
                    .files
                    .iter()
                    .filter_map(|t| match t {
                        Template::TemplateFile(f) => Some(f),
                        _ => None,
                    })
                    .map(|f| render_template_file(&renderer, f, &args_map))
                    .collect::<Result<_>>()?;
                let out = OutputTree { files: files_out };
                println!("{}", serde_json::to_string_pretty(&out)?);
                return Ok(());
            }

            let root = out_dir.clone().ok_or_else(|| {
                anyhow!(
                    "--print not set and no output path provided. Pass --out <path> or supply a positional path."
                )
            })?;
            if !root.exists() {
                fs::create_dir_all(&root)?;
            }
            if !root.is_dir() {
                bail!("Output path '{}' is not a directory.", root.display());
            }
            // Validate all files have paths
            for t in &tree.files {
                if let Template::TemplateFile(f) = t
                    && f.path.is_none()
                {
                    bail!(
                        "Tree '{}' contains a file without a path (file id '{}').",
                        tree.id,
                        f.id
                    );
                }
            }
            for t in &tree.files {
                let f = match t {
                    Template::TemplateFile(f) => f,
                    _ => continue,
                };
                let rendered = render_template_file(&renderer, f, &args_map)?;
                let rel = rendered.path.as_ref().ok_or_else(|| {
                    anyhow!(
                        "Tree '{}' file '{}' is missing a path after rendering.",
                        tree.id,
                        f.id
                    )
                })?;
                let target = root.join(rel);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&target, rendered.content)?;
                println!("Wrote {}", target.display());
            }
            return Ok(());
        }

        let mut all: Vec<String> = files.keys().cloned().collect();
        all.extend(trees.keys().cloned());
        all.sort();
        bail!(
            "Template id '{}' not found. Available: {}",
            id,
            all.join(", ")
        )
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

fn render_template_file(
    renderer: &TemplateRenderer,
    template: &TemplateFile,
    context: &JsonMap<String, JsonValue>,
) -> Result<OutputFile> {
    let missing: Vec<String> = template
        .args
        .iter()
        .filter(|arg| arg.required && !context.contains_key(&arg.name))
        .map(|arg| arg.name.clone())
        .collect();
    if !missing.is_empty() {
        bail!(
            "Missing required argument(s) {} for template '{}'.",
            missing.join(", "),
            template.id
        );
    }

    let content = renderer
        .render_map(&template.id, &template.content, context)
        .with_context(|| format!("Failed to render content for template '{}'", template.id))?;
    let path = match &template.path {
        Some(path_tpl) => Some(
            renderer
                .render_map(&format!("{}::path", template.id), path_tpl, context)
                .with_context(|| format!("Failed to render path for template '{}'", template.id))?,
        ),
        None => None,
    };

    Ok(OutputFile {
        path,
        lang: template.lang.clone(),
        content,
    })
}

fn build_context_map(pairs: &[String]) -> Result<JsonMap<String, JsonValue>> {
    let mut map = JsonMap::new();
    for raw in pairs {
        let (key, value) = parse_arg_pair(raw)?;
        if map.insert(key.clone(), value).is_some() {
            bail!("Duplicate argument '{}'.", key);
        }
    }
    Ok(map)
}

fn extract_positional_out(pairs: &mut Vec<String>) -> Option<FsPathBuf> {
    if let Some(candidate) = pairs.first()
        && !candidate.contains('=')
    {
        let raw = pairs.remove(0).trim().to_string();
        if raw.is_empty() {
            return None;
        }
        return Some(FsPathBuf::from(raw));
    }
    None
}

fn parse_arg_pair(raw: &str) -> Result<(String, JsonValue)> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid argument '{}', expected key=value", raw))?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() {
        bail!("argument name cannot be empty in '{}'", raw);
    }

    Ok((key.to_string(), parse_value(value)))
}

fn parse_value(raw: &str) -> JsonValue {
    if raw.eq_ignore_ascii_case("true") {
        JsonValue::Bool(true)
    } else if raw.eq_ignore_ascii_case("false") {
        JsonValue::Bool(false)
    } else if raw.eq_ignore_ascii_case("null") {
        JsonValue::Null
    } else if let Ok(int) = raw.parse::<i64>() {
        JsonValue::Number(JsonNumber::from(int))
    } else if let Ok(float) = raw.parse::<f64>() {
        JsonNumber::from_f64(float)
            .map(JsonValue::Number)
            .unwrap_or_else(|| JsonValue::String(raw.to_string()))
    } else {
        JsonValue::String(raw.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nmcr_types::Span;
    fn make_loc(path: &str, start: usize, end: usize) -> Location {
        Location {
            path: path.to_string(),
            span: Span { start, end },
        }
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

    #[test]
    fn parse_arg_pair_supports_scalars() {
        let (key, value) = parse_arg_pair("count=42").expect("parse integer");
        assert_eq!(key, "count");
        assert_eq!(value, JsonValue::Number(JsonNumber::from(42)));

        let (_, bool_value) = parse_arg_pair("enabled=true").expect("parse bool");
        assert_eq!(bool_value, JsonValue::Bool(true));

        let (_, string_value) = parse_arg_pair("name=component").expect("parse string");
        assert_eq!(string_value, JsonValue::String("component".into()));

        let (_, null_value) = parse_arg_pair("optional=null").expect("parse null");
        assert_eq!(null_value, JsonValue::Null);
    }

    #[test]
    fn parse_arg_pair_requires_equals() {
        let err = parse_arg_pair("invalid").expect_err("should error");
        assert!(err.to_string().contains("expected key=value"));
    }

    #[test]
    fn extract_positional_out_consumes_first_value() {
        let mut pairs = vec!["./out".to_string(), "name=app".to_string()];
        let out = extract_positional_out(&mut pairs).expect("out present");
        assert_eq!(out, FsPathBuf::from("./out"));
        assert_eq!(pairs, vec!["name=app".to_string()]);
    }
}
