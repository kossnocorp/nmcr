use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Project {
    pub config: Config,
}

impl Project {
    pub fn from_config(config: Config) -> Self {
        Self { config }
    }
}
