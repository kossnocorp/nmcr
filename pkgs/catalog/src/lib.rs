use anyhow::{Context, Result, anyhow};
use nmcr_md_parser::ParsedMarkdown;
use nmcr_md_parser::prelude::parse_file;
use nmcr_types::{Location, Template, TemplateFile, TemplateTree};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CatalogTree {
    id: String,
    name: String,
    description: String,
    files: Vec<TemplateFile>,
    location: Location,
}

impl CatalogTree {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn files(&self) -> &[TemplateFile] {
        &self.files
    }

    pub fn location(&self) -> &Location {
        &self.location
    }
}

#[derive(Debug)]
pub struct TemplateCatalog {
    files: Vec<TemplateFile>,
    trees: Vec<CatalogTree>,
    index: HashMap<String, TemplateRef>,
}

#[derive(Debug)]
enum TemplateRef {
    File(usize),
    Tree(usize),
    TreeFile { tree: usize, file: usize },
}

impl TemplateCatalog {
    pub fn load(paths: &[PathBuf]) -> Result<Self> {
        let mut builder = CatalogBuilder::default();
        for path in paths {
            builder.ingest(path)?;
        }
        Ok(builder.finish())
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.trees.is_empty()
    }

    pub fn standalone_files(&self) -> &[TemplateFile] {
        &self.files
    }

    pub fn tree_templates(&self) -> &[CatalogTree] {
        &self.trees
    }

    pub fn instructions(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some("Templates available via tools: refer to the tool list in your client".to_string())
        }
    }

    pub fn get_tree(&self, id: &str) -> Option<&CatalogTree> {
        match self.index.get(id) {
            Some(TemplateRef::Tree(idx)) => self.trees.get(*idx),
            _ => None,
        }
    }

    pub fn get_file(&self, id: &str) -> Option<FileRef<'_>> {
        match self.index.get(id) {
            Some(TemplateRef::File(idx)) => self.files.get(*idx).map(FileRef::Standalone),
            Some(TemplateRef::TreeFile { tree, file }) => self.trees.get(*tree).and_then(|tree| {
                tree.files()
                    .get(*file)
                    .map(|f| FileRef::TreeMember { tree, file: f })
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum FileRef<'a> {
    Standalone(&'a TemplateFile),
    TreeMember {
        tree: &'a CatalogTree,
        file: &'a TemplateFile,
    },
}

#[derive(Default)]
struct CatalogBuilder {
    files: Vec<TemplateFile>,
    trees: Vec<CatalogTree>,
    index: HashMap<String, TemplateRef>,
    ids: IdRegistry,
}

impl CatalogBuilder {
    fn ingest(&mut self, path: &Path) -> Result<()> {
        let parsed = parse_file(path)
            .with_context(|| format!("Failed to parse template file: {}", path.display()))?;
        match parsed {
            ParsedMarkdown::Template(t) => match t {
                Template::TemplateFile(file) => self.add_file(file)?,
                Template::TemplateTree(tree) => {
                    self.add_tree(tree)?;
                }
            },
            ParsedMarkdown::Tree(tree) => {
                self.add_tree(tree)?;
            }
            ParsedMarkdown::Collection(collection) => {
                let mut trees = Vec::new();
                let mut files = Vec::new();
                for template in collection.templates {
                    match template {
                        Template::TemplateFile(file) => files.push(file),
                        Template::TemplateTree(tree) => trees.push(tree),
                    }
                }

                let mut tree_member_ids: HashSet<String> = HashSet::new();
                for tree in trees {
                    let member_ids = self.add_tree(tree)?;
                    tree_member_ids.extend(member_ids);
                }

                for file in files {
                    if tree_member_ids.contains(&file.id) {
                        continue;
                    }
                    self.add_file(file)?;
                }
            }
        }
        Ok(())
    }

    fn add_file(&mut self, file: TemplateFile) -> Result<()> {
        self.ids.claim(&file.id, &file.location)?;
        let idx = self.files.len();
        self.files.push(file.clone());
        self.index.insert(file.id.clone(), TemplateRef::File(idx));
        Ok(())
    }

    fn add_tree(&mut self, tree: TemplateTree) -> Result<Vec<String>> {
        self.ids.claim(&tree.id, &tree.location)?;
        let tree_index = self.trees.len();
        let mut files: Vec<TemplateFile> = Vec::new();
        let mut member_ids: Vec<String> = Vec::new();
        for template in tree.files.into_iter() {
            match template {
                Template::TemplateFile(file) => {
                    self.ids.claim(&file.id, &file.location)?;
                    member_ids.push(file.id.clone());
                    let file_index = files.len();
                    files.push(file.clone());
                    self.index.insert(
                        file.id.clone(),
                        TemplateRef::TreeFile {
                            tree: tree_index,
                            file: file_index,
                        },
                    );
                }
                Template::TemplateTree(nested) => {
                    return Err(anyhow!(
                        "Nested template trees are not supported (tree '{}', nested '{}')",
                        tree.id,
                        nested.id
                    ));
                }
            }
        }

        let catalog_tree = CatalogTree {
            id: tree.id.clone(),
            name: tree.name,
            description: tree.description,
            files,
            location: tree.location,
        };
        self.index
            .insert(tree.id.clone(), TemplateRef::Tree(tree_index));
        self.trees.push(catalog_tree);
        Ok(member_ids)
    }

    fn finish(self) -> TemplateCatalog {
        TemplateCatalog {
            files: self.files,
            trees: self.trees,
            index: self.index,
        }
    }
}

#[derive(Default)]
struct IdRegistry {
    seen: HashMap<String, Location>,
}

impl IdRegistry {
    fn claim(&mut self, id: &str, location: &Location) -> Result<()> {
        if let Some(existing) = self.seen.get(id) {
            let first = nmcr_types_internal::FormattedLocation(existing).to_string();
            let duplicate = nmcr_types_internal::FormattedLocation(location).to_string();
            return Err(anyhow!("Duplicate template id: {}", id))
                .with_context(|| format!("duplicate occurrence at {duplicate}"))
                .with_context(|| format!("first occurrence at {first}"));
        }
        self.seen.insert(id.to_string(), location.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../examples/basic/tmpls/{name}"))
    }

    #[test]
    fn loads_tree_and_nested_files_without_duplication() {
        let path = fixture("rust-crate.md");
        let catalog = TemplateCatalog::load(&[path]).expect("catalog loads");
        assert!(
            catalog.tree_templates().len() >= 1,
            "expected at least one tree template"
        );
        let tree = &catalog.tree_templates()[0];
        assert!(
            !tree.files().is_empty(),
            "expected tree to carry member files"
        );

        for file in tree.files() {
            let file_id = &file.id;
            let is_standalone = catalog
                .standalone_files()
                .iter()
                .any(|standalone| &standalone.id == file_id);
            assert!(
                !is_standalone,
                "tree member {file_id} should not appear as standalone"
            );
            match catalog.get_file(file_id) {
                Some(FileRef::TreeMember { file: nested, .. }) => {
                    assert_eq!(nested.id, *file_id);
                }
                other => panic!("expected tree member lookup, got {:?}", other),
            }
        }
    }

    #[test]
    fn duplicate_ids_error() {
        let path = fixture("npm.md");
        let err = TemplateCatalog::load(&[path.clone(), path])
            .expect_err("loading same file twice should surface duplicates");
        let rendered = format!("{err:?}");
        assert!(rendered.contains("Duplicate template id"));
    }

    #[test]
    fn unmatched_ids_do_not_resolve() {
        let path = fixture("rust-crate.md");
        let catalog = TemplateCatalog::load(&[path]).expect("catalog loads");
        let tree = &catalog.tree_templates()[0];
        let bogus_id = format!("{}__missing", tree.id());
        assert!(
            catalog.get_file(&bogus_id).is_none(),
            "unexpected match for {bogus_id}"
        );
        assert!(
            catalog.get_tree("not_a_tree").is_none(),
            "unexpected tree match"
        );
    }
}
