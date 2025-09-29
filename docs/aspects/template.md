# Template Aspect

## CLI usage

Use the CLI to inspect output with `nmcr gen template_id --print` or to materialize files with positional arguments and `--out` for the destination root, for example:

```
nmcr gen component name=Button --out ./generated
```

```mermaid
classDiagram
direction TB
    class TemplateFile {
    }

    class Template {
	    File(TemplateFile)
	    Tree(TemplateTree)
    }

    class TemplateTree {
	    +Vec~TemplateFile~ files
    }

	<<struct>> TemplateFile
	<<enumeration>> Template
	<<struct>> TemplateTree

    Template --> TemplateFile : File
    Template --> TemplateTree : Tree
    TemplateTree o-- TemplateFile : files
```
