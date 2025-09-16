use crate::prelude::*;

pub struct UiConfig {}

impl UiConfig {
    pub fn inquire_templates_glob() -> Result<String> {
        let templates_glob = Input::with_theme(UiTheme::for_dialoguer())
            .with_prompt("Templates files pattern")
            .default(Config::default_templates_glob())
            .interact_text()?;
        Ok(templates_glob)
    }
}
