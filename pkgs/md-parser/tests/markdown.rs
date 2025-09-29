use indoc::indoc;
use nmcr_md_parser::ParsedMarkdown;
use nmcr_md_parser::markdown::parse_str;
use nmcr_types::{ArgKind, Template};

#[test]
fn tree_detection() {
    let input = indoc! {r#"
        # Package

        ## Lib

        ### `./Cargo.toml`

        ```toml
        [package]
        name = "example"
        version = "0.1.0"
        ```

        ### `./src/lib.rs`

        ```rust
        pub fn add(a: u8, b: u8) -> u8 { a + b }
        ```
    "#};

    let parsed = parse_str(Some("pkg"), input).expect("parse markdown");

    insta::assert_debug_snapshot!(parsed, @r#"
    Collection(
        TemplateCollection {
            name: "pkg",
            description: "",
            templates: [
                TemplateFile(
                    TemplateFile {
                        kind: "file",
                        id: "package_lib_cargo_toml",
                        name: "./Cargo.toml",
                        description: "",
                        args: [],
                        lang: Some(
                            "toml",
                        ),
                        content: "[package]\nname = \"example\"\nversion = \"0.1.0\"",
                        path: Some(
                            "./Cargo.toml",
                        ),
                        location: Location {
                            path: "",
                            span: Span {
                                start: 19,
                                end: 95,
                            },
                        },
                    },
                ),
                TemplateFile(
                    TemplateFile {
                        kind: "file",
                        id: "package_lib_src_lib_rs",
                        name: "./src/lib.rs",
                        description: "",
                        args: [],
                        lang: Some(
                            "rust",
                        ),
                        content: "pub fn add(a: u8, b: u8) -> u8 { a + b }",
                        path: Some(
                            "./src/lib.rs",
                        ),
                        location: Location {
                            path: "",
                            span: Span {
                                start: 97,
                                end: 169,
                            },
                        },
                    },
                ),
                TemplateTree(
                    TemplateTree {
                        kind: "tree",
                        id: "package_lib",
                        name: "Lib",
                        description: "",
                        files: [
                            TemplateFile(
                                TemplateFile {
                                    kind: "file",
                                    id: "package_lib_cargo_toml",
                                    name: "./Cargo.toml",
                                    description: "",
                                    args: [],
                                    lang: Some(
                                        "toml",
                                    ),
                                    content: "[package]\nname = \"example\"\nversion = \"0.1.0\"",
                                    path: Some(
                                        "./Cargo.toml",
                                    ),
                                    location: Location {
                                        path: "",
                                        span: Span {
                                            start: 19,
                                            end: 95,
                                        },
                                    },
                                },
                            ),
                            TemplateFile(
                                TemplateFile {
                                    kind: "file",
                                    id: "package_lib_src_lib_rs",
                                    name: "./src/lib.rs",
                                    description: "",
                                    args: [],
                                    lang: Some(
                                        "rust",
                                    ),
                                    content: "pub fn add(a: u8, b: u8) -> u8 { a + b }",
                                    path: Some(
                                        "./src/lib.rs",
                                    ),
                                    location: Location {
                                        path: "",
                                        span: Span {
                                            start: 97,
                                            end: 169,
                                        },
                                    },
                                },
                            ),
                        ],
                        location: Location {
                            path: "",
                            span: Span {
                                start: 11,
                                end: 169,
                            },
                        },
                    },
                ),
            ],
            location: Location {
                path: "",
                span: Span {
                    start: 0,
                    end: 170,
                },
            },
        },
    )
    "#);
}
#[test]
fn single() {
    let input = indoc! {r#"
        # Hello World

        This template prints a greeting.

        ```python
        print("Hello, world!")
        ```
    "#};

    let parsed = parse_str(Some("hello_world"), input).expect("parse markdown");

    insta::assert_debug_snapshot!(parsed, @r#"
    Template(
        TemplateFile(
            TemplateFile {
                kind: "file",
                id: "hello_world",
                name: "Hello World",
                description: "This template prints a greeting.",
                args: [],
                lang: Some(
                    "python",
                ),
                content: "print(\"Hello, world!\")",
                path: None,
                location: Location {
                    path: "",
                    span: Span {
                        start: 0,
                        end: 85,
                    },
                },
            },
        ),
    )
    "#);
}

#[test]
fn single_details() {
    let input = indoc! {r#"
        # Personalized Greeting

        Generates a personalized greeting.

        ## Args

        - `name`: Name to greet.

        ## Template

        ```handlebars
        Hello, {{ name }}!
        ```
    "#};

    let parsed = parse_str(Some("personalized_greeting"), input).expect("parse markdown");

    insta::assert_debug_snapshot!(parsed, @r#"
    Template(
        TemplateFile(
            TemplateFile {
                kind: "file",
                id: "personalized_greeting",
                name: "Personalized Greeting",
                description: "Generates a personalized greeting.",
                args: [
                    Arg {
                        name: "name",
                        description: "Name to greet.",
                        kind: Any(
                            "any",
                        ),
                        required: true,
                    },
                ],
                lang: Some(
                    "handlebars",
                ),
                content: "Hello, {{ name }}!",
                path: None,
                location: Location {
                    path: "",
                    span: Span {
                        start: 0,
                        end: 145,
                    },
                },
            },
        ),
    )
    "#);
}

#[test]
fn argument_types_and_descriptions() {
    let input = indoc! {r#"
        # Component

        ## Arguments

        - `name` [string]: Display name.
        - `includeProps?` [boolean]

        ## Template

        ```handlebars
        export const Component = () => "hello";
        ```
    "#};

    let parsed = parse_str(Some("component"), input).expect("parse markdown");

    let file = match parsed {
        ParsedMarkdown::Template(Template::TemplateFile(file)) => file,
        other => panic!("unexpected parser result: {other:?}"),
    };

    assert_eq!(file.args.len(), 2, "expected two arguments");

    let first = &file.args[0];
    assert_eq!(first.name, "name");
    assert_eq!(first.description, "Display name.");
    assert!(matches!(first.kind, ArgKind::String(_)));
    assert!(first.required);

    let second = &file.args[1];
    assert_eq!(second.name, "includeProps");
    assert!(second.description.is_empty());
    assert!(matches!(second.kind, ArgKind::Boolean(_)));
    assert!(!second.required);
}

#[test]
fn collection() {
    let input = indoc! {r#"
        # Greeting Templates

        A collection of greeting snippets.

        ## Quick Hello

        ```text
        Hello!
        ```

        ## Friendly Greeting

        ### Args

        - `name`: Name to greet.

        ### Template

        ```handlebars
        Hey, {{ name }}!
        ```

        ## Formal Greeting

        ```text
        Good day.
        ```
    "#};

    let parsed = parse_str(Some("greetings"), input).expect("parse markdown");

    insta::assert_debug_snapshot!(parsed, @r#"
    Template(
        TemplateFile(
            TemplateFile {
                kind: "file",
                id: "greeting_templates_friendly_greeting",
                name: "Friendly Greeting",
                description: "",
                args: [
                    Arg {
                        name: "name",
                        description: "Name to greet.",
                        kind: Any(
                            "any",
                        ),
                        required: true,
                    },
                ],
                lang: Some(
                    "handlebars",
                ),
                content: "Hey, {{ name }}!",
                path: None,
                location: Location {
                    path: "",
                    span: Span {
                        start: 94,
                        end: 200,
                    },
                },
            },
        ),
    )
    "#);
}
