use indoc::indoc;
use nmcr_md_parser::markdown::parse_str;

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
        Template {
            name: "Hello World",
            description: "This template prints a greeting.",
            collection: None,
            args: TemplateArgs {
                items: [],
            },
            lang: Some(
                "python",
            ),
            content: "print(\"Hello, world!\")",
            location: Location {
                path: "",
                span: Span {
                    start: 0,
                    end: 85,
                },
            },
        },
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
        Template {
            name: "Personalized Greeting",
            description: "Generates a personalized greeting.",
            collection: None,
            args: TemplateArgs {
                items: [
                    TemplateArg {
                        name: "name",
                        description: "Name to greet.",
                        kind: Any,
                    },
                ],
            },
            lang: Some(
                "handlebars",
            ),
            content: "Hello, {{ name }}!",
            location: Location {
                path: "",
                span: Span {
                    start: 0,
                    end: 145,
                },
            },
        },
    )
    "#);
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
        Template {
            name: "Friendly Greeting",
            description: "",
            collection: Some(
                "Greeting Templates",
            ),
            args: TemplateArgs {
                items: [
                    TemplateArg {
                        name: "name",
                        description: "Name to greet.",
                        kind: Any,
                    },
                ],
            },
            lang: Some(
                "handlebars",
            ),
            content: "Hey, {{ name }}!",
            location: Location {
                path: "",
                span: Span {
                    start: 94,
                    end: 200,
                },
            },
        },
    )
    "#);
}
