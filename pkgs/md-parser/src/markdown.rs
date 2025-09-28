use crate::prelude::*;
use nmcr_id::EntityId;
use relative_path::RelativePathBuf;

#[derive(Clone, Debug)]
struct Section {
    level: u8,
    title: String,
    heading_nodes: Vec<mdast::Node>,
    nodes: Vec<mdast::Node>,
    heading_span: Option<Span>,
    path: Vec<String>,
}

const ALLOWED_SUBHEADS: &[&str] = &["args", "arguments", "template"];

pub fn parse_file(path: &Path) -> Result<ParsedMarkdown> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read markdown file: {}", path.display()))?;
    let stem = path.file_stem().and_then(|s| s.to_str());
    parse_str_with_path(Some(path), stem, &content)
}

pub fn parse_str(file_stem: Option<&str>, input: &str) -> Result<ParsedMarkdown> {
    parse_str_with_path(None, file_stem, input)
}

fn parse_str_with_path(
    path: Option<&Path>,
    file_stem: Option<&str>,
    input: &str,
) -> Result<ParsedMarkdown> {
    let root = to_mdast(input, &ParseOptions::default())
        .map_err(|e| anyhow!("Failed to parse markdown: {e}"))?;

    let root = match root {
        mdast::Node::Root(root) => root,
        _ => bail!("Markdown root node is not a Root"),
    };

    let root_span = root.position.as_ref().map(position_to_span);

    let sections = make_sections(&root.children);

    // First pass: find min level of allowed subheads anywhere
    let min_allowed_level = sections
        .iter()
        .filter(|s| is_allowed(&s.title))
        .map(|s| s.level)
        .min();

    if let Some(min_level) = min_allowed_level {
        let base_level = min_level.saturating_sub(1).max(1);

        // Gather file templates from sections at base level that contain allowed subheadings
        let mut templates: Vec<Template> = Vec::new();
        // Track mapping from parent section index to its direct child file templates
        let mut parent_children: Vec<(usize, Vec<TemplateFile>)> = Vec::new();

        // Build an index of sections by their original order
        for (idx, parent) in sections
            .iter()
            .enumerate()
            .filter(|(_, s)| s.level == base_level)
        {
            if section_contains_allowed(parent) {
                if let Some(t) = parse_template_from_section(parent, path)? {
                    templates.push(Template::TemplateFile(t.clone()));
                }

                // Direct children of this parent are sub-sections with greater level but not exceeding base_level+1
                let children: Vec<Section> = make_sections(&parent.nodes)
                    .into_iter()
                    .filter(|s| s.level == base_level + 1)
                    .collect();

                let mut child_templates: Vec<TemplateFile> = Vec::new();
                for child in &children {
                    if section_contains_allowed(child) {
                        if let Some(t) = parse_template_from_section(child, path)? {
                            child_templates.push(t);
                        }
                    } else if let Some((lang, content)) =
                        collect_code_blocks(&child.nodes).into_iter().next()
                    {
                        // Support simple child with a single code block (no subheads)
                        let id =
                            EntityId::new().from_segments(child.path.iter().map(|s| s.as_str()));
                        if !id.is_empty() {
                            let mut t = TemplateFile {
                                kind: TemplateFileKindFile,
                                id,
                                name: child.title.clone(),
                                description: collect_description(child),
                                args: Vec::new(),
                                lang,
                                content,
                                location: section_location(child, path),
                                path: None,
                            };
                            // Attempt inline path capture
                            t.path = extract_inline_path_before_code(&child.nodes);
                            child_templates.push(t);
                        }
                    }
                }

                if !child_templates.is_empty() {
                    parent_children.push((idx, child_templates));
                }
            }
        }

        // Build trees: a section becomes a tree if all of its direct child templates have a path
        let mut trees: Vec<TemplateTree> = Vec::new();
        for (idx, files) in parent_children.into_iter() {
            if files.iter().all(|t| t.path.is_some()) {
                let parent = &sections[idx];
                let id = EntityId::new().from_segments(parent.path.iter().map(|s| s.as_str()));
                if id.is_empty() {
                    continue;
                }
                let tree = TemplateTree {
                    kind: TemplateTreeKindTree,
                    id,
                    name: parent.title.clone(),
                    description: collect_tree_description(parent),
                    files: files.into_iter().map(Template::TemplateFile).collect(),
                    location: section_location(parent, path),
                };
                trees.push(tree);
            }
        }

        // Decide which top-level variant to return
        let collection_meta = sections.iter().find(|s| s.level < base_level);
        match (templates.len(), trees.len()) {
            (0, 0) => bail!("No templates found in markdown"),
            (0, 1) => return Ok(ParsedMarkdown::Tree(trees.remove(0))),
            (1, 0) => return Ok(ParsedMarkdown::Template(templates.remove(0))),
            _ => {
                // Collection: include both file templates and any detected trees
                let (name, description) = if let Some(meta) = collection_meta {
                    (meta.title.clone(), collect_description(meta))
                } else {
                    (file_stem.unwrap_or("Untitled").to_string(), String::new())
                };
                let location = collection_meta
                    .map(|meta| section_location(meta, path))
                    .unwrap_or_else(|| make_location(path, root_span.clone()));
                // Wrap trees into union templates
                let mut all_templates = templates;
                for tr in trees {
                    all_templates.push(Template::TemplateTree(tr));
                }
                return Ok(ParsedMarkdown::Collection(TemplateCollection {
                    name,
                    description,
                    templates: all_templates,
                    location,
                }));
            }
        }
    }

    // Second pass: no allowed subheads — use code-block heuristic
    let mut standalones: Vec<(TemplateFile, Vec<String>)> = Vec::new();
    for sec in &sections {
        let subsections = make_sections(&sec.nodes);
        let has_subheads = subsections.iter().any(|s| s.level > sec.level);
        if !has_subheads {
            let mut codes = collect_code_blocks(&sec.nodes);
            if codes.len() == 1 {
                let (lang, content) = codes.remove(0);
                let id = EntityId::new().from_segments(sec.path.iter().map(|s| s.as_str()));
                if id.is_empty() {
                    bail!(
                        "Unable to derive template id from headings: {}",
                        sec.path.join(" > ")
                    );
                }

                let mut tmpl = TemplateFile {
                    kind: TemplateFileKindFile,
                    id,
                    name: sec.title.clone(),
                    description: collect_description(sec),
                    args: Vec::new(),
                    lang,
                    content,
                    location: section_location(sec, path),
                    path: None,
                };
                tmpl.path = extract_inline_path_before_code(&sec.nodes)
                    .or_else(|| extract_inline_path_from_heading(sec));
                // Parent path (all but last segment)
                let parent_path = if sec.path.len() > 1 {
                    sec.path[..(sec.path.len() - 1)].to_vec()
                } else {
                    Vec::new()
                };
                standalones.push((tmpl, parent_path));
            }
        }
    }

    match standalones.len() {
        0 => bail!("No templates found in markdown"),
        1 => Ok(ParsedMarkdown::Template(Template::TemplateFile(
            standalones.remove(0).0,
        ))),
        _ => {
            use std::collections::BTreeMap;
            let mut groups: BTreeMap<Vec<String>, Vec<TemplateFile>> = BTreeMap::new();
            for (t, parent) in standalones.into_iter() {
                groups.entry(parent).or_default().push(t);
            }

            let mut templates: Vec<Template> = Vec::new();
            let mut trees: Vec<TemplateTree> = Vec::new();

            // Helper to find a section by its full path
            let find_section =
                |p: &Vec<String>| -> Option<&Section> { sections.iter().find(|s| &s.path == p) };

            for (parent_path, files) in groups.into_iter() {
                if !files.is_empty() && files.iter().all(|t| t.path.is_some())
                    && let Some(parent_sec) = find_section(&parent_path) {
                        let id = EntityId::new()
                            .from_segments(parent_sec.path.iter().map(|s| s.as_str()));
                        if !id.is_empty() {
                            let tree = TemplateTree {
                                kind: TemplateTreeKindTree,
                                id,
                                name: parent_sec.title.clone(),
                                description: collect_tree_description(parent_sec),
                                files: files
                                    .clone()
                                    .into_iter()
                                    .map(Template::TemplateFile)
                                    .collect(),
                                location: section_location(parent_sec, path),
                            };
                            trees.push(tree);
                        }
                    }
                templates.extend(files.into_iter().map(Template::TemplateFile));
            }

            for tr in trees {
                templates.push(Template::TemplateTree(tr));
            }
            Ok(ParsedMarkdown::Collection(TemplateCollection {
                name: file_stem.unwrap_or("Untitled").to_string(),
                description: String::new(),
                templates,
                location: make_location(path, root_span),
            }))
        }
    }
}

fn make_sections(nodes: &[mdast::Node]) -> Vec<Section> {
    // Collect indices of headings
    let mut heads: Vec<(usize, u8, String, Option<Span>)> = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        if let mdast::Node::Heading(h) = node {
            let title = inline_text(&h.children);
            let span = node.position().map(position_to_span);
            heads.push((i, h.depth, title, span));
        }
    }
    let mut sections = Vec::new();
    let mut stack: Vec<(u8, String)> = Vec::new();
    for (idx, (start, level, title, span)) in heads.iter().enumerate() {
        while let Some((stack_level, _)) = stack.last() {
            if *stack_level >= *level {
                stack.pop();
            } else {
                break;
            }
        }

        let mut path: Vec<String> = stack.iter().map(|(_, t)| t.clone()).collect();
        path.push(title.clone());
        stack.push((*level, title.clone()));

        let end = heads
            .iter()
            .skip(idx + 1)
            .find(|(_, next_level, _, _)| *next_level <= *level)
            .map(|(i, _, _, _)| *i)
            .unwrap_or(nodes.len());
        // Nodes within this section excluding the heading line itself
        let inner = nodes[(start + 1)..end].to_vec();
        let heading_nodes = if let mdast::Node::Heading(h) = &nodes[*start] {
            h.children.clone()
        } else {
            Vec::new()
        };
        sections.push(Section {
            level: *level,
            title: title.clone(),
            heading_nodes,
            nodes: inner,
            heading_span: span.clone(),
            path,
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

fn parse_template_from_section(
    section: &Section,
    path: Option<&Path>,
) -> Result<Option<TemplateFile>> {
    let subsections = make_sections(&section.nodes);

    // Name/description from section itself
    let mut tmpl = TemplateFile {
        kind: TemplateFileKindFile,
        id: EntityId::new().from_segments(section.path.iter().map(|s| s.as_str())),
        name: section.title.clone(),
        description: collect_description(section),
        args: Vec::new(),
        lang: None,
        content: String::new(),
        location: section_location(section, path),
        path: None,
    };

    if tmpl.id.is_empty() {
        bail!(
            "Unable to derive template id from headings: {}",
            section.path.join(" > ")
        );
    }

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
        && let Some((lang, content)) = collect_code_blocks(&tpl_sec.nodes).into_iter().next()
    {
        tmpl.lang = lang;
        tmpl.content = content;
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

    // Inline path capture (before the first code block)
    tmpl.path = extract_inline_path_before_code(&section.nodes)
        .or_else(|| extract_inline_path_from_heading(section));

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
            mdast::Node::Paragraph(p) => {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }
                out.push_str(&inline_text(&p.children));
            }
            mdast::Node::Heading(_) | mdast::Node::Code(_) => break,
            _ => {}
        }
    }
    out.trim().to_string()
}

fn collect_tree_description(section: &Section) -> String {
    // Description for a tree: prose before the first child heading
    let mut out = String::new();
    for node in &section.nodes {
        match node {
            mdast::Node::Heading(_) => break,
            mdast::Node::Paragraph(p) => {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }
                out.push_str(&inline_text(&p.children));
            }
            _ => {}
        }
    }
    out.trim().to_string()
}

fn collect_code_blocks(nodes: &[mdast::Node]) -> Vec<(Option<String>, String)> {
    let mut acc: Vec<(Option<String>, String)> = Vec::new();
    for node in nodes {
        match node {
            mdast::Node::Code(code) => acc.push((code.lang.clone(), code.value.clone())),
            // Recurse into list and list items
            mdast::Node::List(list) => {
                acc.extend(collect_code_blocks(&list.children));
            }
            mdast::Node::ListItem(item) => acc.extend(collect_code_blocks(&item.children)),
            mdast::Node::Blockquote(bq) => acc.extend(collect_code_blocks(&bq.children)),
            _ => {}
        }
    }
    acc
}

fn extract_inline_path_before_code(nodes: &[mdast::Node]) -> Option<String> {
    for node in nodes {
        match node {
            mdast::Node::Code(_) => break,
            mdast::Node::Paragraph(p) => {
                // Heuristic: paragraph text ends with ':' and contains at least one inline code
                let txt = inline_text(&p.children).trim().to_string();
                if txt.ends_with(':') {
                    // Use the first inline code as the path
                    for child in &p.children {
                        if let mdast::Node::InlineCode(ic) = child {
                            let val = ic.value.trim();
                            if !val.is_empty() {
                                return Some(val.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_inline_path_from_heading(section: &Section) -> Option<String> {
    for node in &section.heading_nodes {
        if let mdast::Node::InlineCode(ic) = node {
            let val = ic.value.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn parse_args(section: &Section) -> Vec<Arg> {
    let mut args = Vec::new();
    for node in &section.nodes {
        match node {
            mdast::Node::List(list) => {
                for item in &list.children {
                    if let mdast::Node::ListItem(li) = item
                        && let Some((name, desc)) = parse_arg_item(li)
                    {
                        args.push(Arg {
                            name,
                            description: desc,
                            kind: ArgKind::Any(ArgKindAny),
                        });
                    }
                }
            }
            mdast::Node::ListItem(item) => {
                if let Some((name, desc)) = parse_arg_item(item) {
                    args.push(Arg {
                        name,
                        description: desc,
                        kind: ArgKind::Any(ArgKindAny),
                    });
                }
            }
            _ => {}
        }
    }
    args
}

fn parse_arg_item(item: &mdast::ListItem) -> Option<(String, String)> {
    // Strategy: find first inline code as name, then collect text AFTER it.
    // On the very first text fragment after the name, strip leading separators like ':' or '-'.
    let mut name: Option<String> = None;
    let mut desc = String::new();
    let mut started_desc = false;

    for node in &item.children {
        if let mdast::Node::Paragraph(p) = node {
            for child in &p.children {
                match child {
                    mdast::Node::InlineCode(ic) if name.is_none() => {
                        name = Some(ic.value.clone());
                    }
                    _ if name.is_none() => {
                        // Ignore any content before the first inline code (name)
                    }
                    mdast::Node::Text(t) => {
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
                        let s = inline_text(std::slice::from_ref(other));
                        if !s.is_empty() {
                            if !desc.is_empty()
                                && !desc.ends_with(' ')
                                && !s.starts_with(char::is_whitespace)
                            {
                                desc.push(' ');
                            }
                            if !started_desc && !s.is_empty() {
                                // First fragment: strip leading separators if it starts with them
                                let trimmed = s
                                    .trim_start()
                                    .trim_start_matches([':', '-', '–', '—'])
                                    .trim_start()
                                    .to_string();
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

fn position_to_span(position: &markdown::unist::Position) -> Span {
    Span {
        start: position.start.offset,
        end: position.end.offset,
    }
}

fn merge_span(base: &mut Option<Span>, candidate: Span) {
    match base {
        Some(existing) => {
            if candidate.start < existing.start {
                existing.start = candidate.start;
            }
            if candidate.end > existing.end {
                existing.end = candidate.end;
            }
        }
        None => *base = Some(candidate),
    }
}

fn extend_span_with_node(out: &mut Option<Span>, node: &mdast::Node) {
    if let Some(position) = node.position() {
        merge_span(out, position_to_span(position));
    }

    if let Some(children) = node.children() {
        for child in children {
            extend_span_with_node(out, child);
        }
    }
}

fn extend_span_with_nodes(out: &mut Option<Span>, nodes: &[mdast::Node]) {
    for node in nodes {
        extend_span_with_node(out, node);
    }
}

fn section_span(section: &Section) -> Option<Span> {
    let mut span = section.heading_span.clone();
    extend_span_with_nodes(&mut span, &section.nodes);
    span
}

fn section_location(section: &Section, path: Option<&Path>) -> Location {
    make_location(path, section_span(section))
}

fn make_location(path: Option<&Path>, span: Option<Span>) -> Location {
    Location {
        path: path.map(normalize_relative_path).unwrap_or_default(),
        span: span.unwrap_or(Span { start: 0, end: 0 }),
    }
}

fn normalize_relative_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    RelativePathBuf::from(normalized).into_string()
}

fn inline_text(nodes: &[mdast::Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            mdast::Node::Text(t) => out.push_str(&t.value),
            mdast::Node::InlineCode(c) => out.push_str(&c.value),
            mdast::Node::Emphasis(e) => out.push_str(&inline_text(&e.children)),
            mdast::Node::Strong(e) => out.push_str(&inline_text(&e.children)),
            mdast::Node::Link(e) => out.push_str(&inline_text(&e.children)),
            mdast::Node::Paragraph(p) => out.push_str(&inline_text(&p.children)),
            mdast::Node::Heading(h) => out.push_str(&inline_text(&h.children)),
            _ => {}
        }
    }
    out
}
