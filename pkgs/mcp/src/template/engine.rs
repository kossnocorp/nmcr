use crate::prelude::*;
use nmcr_template::TemplateRenderer;

pub(crate) fn render_template(
    template_id: &str,
    template: &str,
    args: &JsonMap<String, JsonValue>,
) -> Result<String> {
    let renderer = TemplateRenderer::new();
    renderer
        .render_map(template_id, template, args)
        .with_context(|| format!("Failed to render template '{}'", template_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_template_content_unmodified() {
        let mut ctx = JsonMap::new();
        ctx.insert("name".into(), JsonValue::String("world".into()));
        let rendered = render_template("greeting", "Hello, {{name}}!", &ctx).expect("render");
        assert_eq!(rendered, "Hello, world!");
    }

    #[test]
    fn missing_variable_errors() {
        let err = render_template("greeting", "Hello, {{name}}!", &JsonMap::new())
            .expect_err("missing args should fail");
        assert!(err.to_string().contains("greeting"));
    }
}
