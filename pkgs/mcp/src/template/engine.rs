use crate::prelude::*;

pub fn render_template(content: &str, _args: &JsonMap<String, JsonValue>) -> String {
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_template_content_unmodified() {
        let rendered = render_template("Hello, {{name}}!", &JsonMap::new());
        assert_eq!(rendered, "Hello, {{name}}!");
    }
}
