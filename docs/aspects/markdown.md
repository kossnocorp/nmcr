# Markdown Template Aspect

## Argument notation

Document template arguments under an "Args" or "Arguments" heading using list items with inline code for the variable name. Append a `?` to the inline code (for example, `` `suffix?` ``) to mark an argument as optional. You can declare the expected type by placing `[boolean]`, `[string]`, `[number]`, or `[any]` immediately after the argument name, and follow it with an optional description introduced by a colon:

```
- `name` [string]: Display name for the generated export.
- `withTests?` [boolean]
```

The parser automatically merges these declarations with Handlebars placeholders discovered in the template content and any relative path strings. Newly discovered placeholders are treated as required arguments unless they already appear in the documentation.
