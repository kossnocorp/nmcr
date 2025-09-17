use crate::prelude::*;

pub struct UiMessage {}

impl UiMessage {
    #[allow(dead_code)]
    pub fn info(message: &str) {
        println!("{}", UiTheme::format_info(message));
    }

    #[allow(dead_code)]
    pub fn warn(message: &str) {
        println!("{}", UiTheme::format_warn(message));
    }

    #[allow(dead_code)]
    pub fn success(message: &str) {
        println!("{}", UiTheme::format_success(message));
    }

    pub fn error(err: anyhow::Error) {
        eprintln!("{}", UiTheme::format_error(&format!("{:#}", err)));
    }
}
