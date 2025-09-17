use anyhow::{anyhow, bail, Context, Result};
use std::path::Path;

use crate::ParsedMarkdown;
use nmcr_types::{Template, TemplateArgs, TemplateCollection};

// The markdown crate exposes mdast structures and a conversion entry point.
use markdown::{mdast as ast, to_mdast, ParseOptions};

#[derive(Clone, Debug)]
struct Section {
    level: u8,
    title: String,
    nodes: Vec<ast::Node>,
}

const ALLOWED_SUBHEADS: &[&str] = &["args", "arguments", "template"];

pub fn parse_file(path: &Path) -> Result<ParsedMarkdown> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read markdown file: {}", path.display()))?;
    let stem = path.file_stem().and_then(|s| s.to_str());
    parse_str(stem, &content)
}

pub fn parse_str(file_stem: Option<&str>, input: &str) -> Result<ParsedMarkdown> {
    let root = to_mdast(input, &ParseOptions::default())
        .map_err(|e| anyhow!("Failed to parse markdown: {e}"))?;

    let root = match root {
        ast::Node::Root(root) => root,
        _ => bail!("Markdown root node is not a Root"),
    };

    let sections = make_sections(&root.children);

    // First pass: find min level of allowed subheads anywhere
    let min_allowed_level = sections
        .iter()
        .filter(|s| is_allowed(&s.title))
        .map(|s| s.level)
        .min();

    if let Some(min_level) = min_allowed_level {
        let base_level = min_level.saturating_sub(1).max(1);

        // Parent sections at base level that contain at least one allowed subheading
        let mut templates = Vec::new();
        for parent in sections.iter().filter(|s| s.level == base_level) {
            if section_contains_allowed(parent) {
                if let Some(t) = parse_template_from_section(parent)? {
                    templates.push(t);
                }
            }
        }

        match templates.len() {
            0 => bail!("No templates found in markdown"),
            1 => return Ok(ParsedMarkdown::Template(templates.remove(0))),
            _ => {
                // Collection: choose collection meta from nearest header above base_level (usually H1)
                let collection_meta = sections
                    .iter()
                    .filter(|s| s.level < base_level)
                    .next();
                let (name, description) = if let Some(meta) = collection_meta {
                    (meta.title.clone(), collect_description(meta))
                } else {
                    (
                        file_stem.unwrap_or("Untitled").to_string(),
                        String::new(),
                    )
                };
                return Ok(ParsedMarkdown::Collection(TemplateCollection {
                    name,
                    description,
                    templates,
                }));
            }
        }
    }

    // Second pass: no allowed subheads — use code-block heuristic
    let mut standalones = Vec::new();
    for sec in &sections {
        let subsections = make_sections(&sec.nodes);
        let has_subheads = subsections.iter().any(|s| s.level > sec.level);
        if !has_subheads {
            let mut codes = collect_code_blocks(&sec.nodes);
            if codes.len() == 1 {
                let (lang, content) = codes.remove(0);
                let mut tmpl = Template::default();
                tmpl.name = sec.title.clone();
                tmpl.description = collect_description(sec);
                tmpl.lang = lang;
                tmpl.content = content;
                standalones.push(tmpl);
            }
        }
    }

    match standalones.len() {
        0 => bail!("No templates found in markdown"),
        1 => Ok(ParsedMarkdown::Template(standalones.remove(0))),
        _ => Ok(ParsedMarkdown::Collection(TemplateCollection {
            name: file_stem.unwrap_or("Untitled").to_string(),
            description: String::new(),
            templates: standalones,
        })),
    }
}

fn make_sections(nodes: &[ast::Node]) -> Vec<Section> {
    // Collect indices of headings
    let mut heads: Vec<(usize, u8, String)> = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        if let ast::Node::Heading(h) = node {
            let title = inline_text(&h.children);
            heads.push((i, h.depth as u8, title));
        }
    }
    let mut sections = Vec::new();
    for (idx, (start, level, title)) in heads.iter().enumerate() {
        let end = heads
            .iter()
            .skip(idx + 1)
            .find(|(_, next_level, _)| *next_level <= *level)
            .map(|(i, _, _)| *i)
            .unwrap_or(nodes.len());
        // Nodes within this section excluding the heading line itself
        let inner = nodes[(start + 1)..end].to_vec();
        sections.push(Section {
            level: *level,
            title: title.clone(),
            nodes: inner,
        });
    }
    sections
}

fn section_contains_allowed(section: &Section) -> bool {
    let subs = make_sections(&section.nodes);
    subs.iter().any(|s| is_allowed(&s.title))
}

fn is_allowed(title: &str) -> bool {
    let t = title.trim().to_lowercase();
    ALLOWED_SUBHEADS.contains(&t.as_str())
}

fn parse_template_from_section(section: &Section) -> Result<Option<Template>> {
    let subsections = make_sections(&section.nodes);

    // Name/description from section itself
    let mut tmpl = Template::default();
    tmpl.name = section.title.clone();
    tmpl.description = collect_description(section);

    // Args
    if let Some(args_sec) = subsections
        .iter()
        .find(|s| matches_subhead(&s.title, &["args", "arguments"]))
    {
        tmpl.args = parse_args(args_sec);
    }

    // Template content
    if let Some(tpl_sec) = subsections
        .iter()
        .find(|s| matches_subhead(&s.title, &["template"]))
    {
        if let Some((lang, content)) = collect_code_blocks(&tpl_sec.nodes).into_iter().next() {
            tmpl.lang = lang;
            tmpl.content = content;
        }
    }

    // Fallback: single code block anywhere in the section
    if tmpl.content.is_empty() {
        let mut codes = collect_code_blocks(&section.nodes);
        if codes.len() == 1 {
            let (lang, content) = codes.remove(0);
            tmpl.lang = lang;
            tmpl.content = content;
        }
    }

    if tmpl.content.is_empty() {
        // No content yet — this section is not a complete template
        return Ok(None);
    }

    Ok(Some(tmpl))
}

fn matches_subhead(title: &str, names: &[&str]) -> bool {
    let t = title.trim().to_lowercase();
    names.contains(&t.as_str())
}

fn collect_description(section: &Section) -> String {
    let mut out = String::new();
    for node in &section.nodes {
        match node {
            ast::Node::Paragraph(p) => {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }
                out.push_str(&inline_text(&p.children));
            }
            ast::Node::Heading(_) | ast::Node::Code(_) => break,
            _ => {}
        }
    }
    out.trim().to_string()
}

fn collect_code_blocks(nodes: &[ast::Node]) -> Vec<(Option<String>, String)> {
    let mut acc: Vec<(Option<String>, String)> = Vec::new();
    for node in nodes {
        match node {
            ast::Node::Code(code) => acc.push((code.lang.clone(), code.value.clone())),
            // Recurse into list and list items
            ast::Node::List(list) => {
                acc.extend(collect_code_blocks(&list.children));
            }
            ast::Node::ListItem(item) => acc.extend(collect_code_blocks(&item.children)),
            ast::Node::Blockquote(bq) => acc.extend(collect_code_blocks(&bq.children)),
            _ => {}
        }
    }
    acc
}

fn parse_args(section: &Section) -> TemplateArgs {
    let mut args = TemplateArgs::new();
    for node in &section.nodes {
        match node {
            ast::Node::List(list) => {
                for item in &list.children {
                    if let ast::Node::ListItem(li) = item {
                        if let Some((name, desc)) = parse_arg_item(li) {
                            args.push(name, desc);
                        }
                    }
                }
            }
            ast::Node::ListItem(item) => {
                if let Some((name, desc)) = parse_arg_item(item) {
                    args.push(name, desc);
                }
            }
            _ => {}
        }
    }
    args
}

fn parse_arg_item(item: &ast::ListItem) -> Option<(String, String)> {
    // Strategy: find first inline code as name, then collect text AFTER it.
    // On the very first text fragment after the name, strip leading separators like ':' or '-'.
    let mut name: Option<String> = None;
    let mut desc = String::new();
    let mut started_desc = false;

    for node in &item.children {
        if let ast::Node::Paragraph(p) = node {
            for child in &p.children {
                match child {
                    ast::Node::InlineCode(ic) if name.is_none() => {
                        name = Some(ic.value.clone());
                    }
                    _ if name.is_none() => {
                        // Ignore any content before the first inline code (name)
                    }
                    ast::Node::Text(t) => {
                        let mut s = t.value.as_str();
                        if !started_desc {
                            // Trim leading whitespace and common separators once
                            s = s.trim_start();
                            s = s.trim_start_matches([':', '-', '–', '—']);
                            s = s.trim_start();
                            started_desc = true;
                        }
                        if !s.is_empty() {
                            if !desc.is_empty() && !desc.ends_with(' ') {
                                desc.push(' ');
                            }
                            desc.push_str(s);
                        }
                    }
                    other => {
                        let s = inline_text(&[other.clone()]);
                        if !s.is_empty() {
                            if !desc.is_empty() && !desc.ends_with(' ') && !s.starts_with(char::is_whitespace) {
                                desc.push(' ');
                            }
                            if !started_desc && !s.is_empty() {
                                // First fragment: strip leading separators if it starts with them
                                let trimmed = s.trim_start().trim_start_matches([':', '-', '–', '—']).trim_start().to_string();
                                desc.push_str(&trimmed);
                                started_desc = true;
                            } else {
                                desc.push_str(&s);
                            }
                        }
                    }
                }
            }
        }
    }

    let name = name?;
    let desc = desc.trim().to_string();
    Some((name, desc))
}

fn inline_text(nodes: &[ast::Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            ast::Node::Text(t) => out.push_str(&t.value),
            ast::Node::InlineCode(c) => out.push_str(&c.value),
            ast::Node::Emphasis(e) => out.push_str(&inline_text(&e.children)),
            ast::Node::Strong(e) => out.push_str(&inline_text(&e.children)),
            ast::Node::Link(e) => out.push_str(&inline_text(&e.children)),
            ast::Node::Paragraph(p) => out.push_str(&inline_text(&p.children)),
            ast::Node::Heading(h) => out.push_str(&inline_text(&h.children)),
            _ => {}
        }
    }
    out
}
