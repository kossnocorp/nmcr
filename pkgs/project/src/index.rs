use wax::Glob;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Project {
    pub config: Config,
}

impl Project {
    pub fn from_config(config: Config) -> Self {
        Self { config }
    }

    pub fn load<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        let config = Config::find(path)?;
        Ok(Self::from_config(config))
    }

    pub fn path(&self) -> PathBuf {
        self.config
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    }

    pub fn template_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        let pattern = self.config.user.normalized_templates();
        let glob = Glob::new(&pattern)
            .with_context(|| format!("Failed to build glob with pattern: {}", &pattern))?;

        for entry in glob.walk(self.path()) {
            let entry = entry?;
            if entry.file_type().is_file() {
                paths.push(entry.path().to_path_buf());
            }
        }

        Ok(paths)
    }
}
