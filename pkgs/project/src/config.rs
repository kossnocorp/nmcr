use crate::prelude::*;
use std::fs;

pub const CONFIG_FILENAME: &str = "ncmr.toml";

pub const DEFAULT_TEMPLATES_GLOB: &str = "./tmpls/**/*.md";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    /// Config path.
    pub path: PathBuf,
    /// User config.
    pub user: ConfigUser,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigUser {
    /// Glob pattern to find template files
    #[serde(default = "Config::default_templates_glob")]
    pub templates: String,
}

impl Default for ConfigUser {
    fn default() -> Self {
        Self {
            templates: Config::default_templates_glob(),
        }
    }
}

impl Config {
    pub fn new(path: PathBuf, user: ConfigUser) -> Self {
        Self { path, user }
    }

    pub fn resolve_path<P: AsRef<Path>>(path: P) -> PathBuf {
        if path.as_ref().ends_with(CONFIG_FILENAME) {
            return path.as_ref().to_path_buf();
        }
        Self::join_path(path)
    }

    pub fn join_path<P: AsRef<Path>>(path: P) -> PathBuf {
        path.as_ref().join(CONFIG_FILENAME)
    }

    pub fn init<P: AsRef<Path>>(path: P, force: bool) -> Result<Self> {
        let path = Self::resolve_path(path);
        if path.exists() && !force {
            bail!("Config file already exists at {}", path.display());
        }

        Ok(Self::new(path, Default::default()))
    }

    pub fn find(path: &Option<PathBuf>) -> Result<Config> {
        match path {
            // If there's a path, locate the config file there
            Some(path) => {
                // If that's a directory, join the config filename
                let path = if path.is_dir() {
                    Self::join_path(path)
                } else {
                    path.clone()
                };
                Self::read(path.clone())
            }

            // Locate the config file in the current or parent directories
            None => {
                let initial: PathBuf = ".".into();
                let mut cur_path: Option<&Path> = Some(&initial);
                while let Some(path) = cur_path {
                    let file = Self::join_path(path);
                    if file.is_file() {
                        return Self::read(file);
                    }
                    cur_path = path.parent();
                }

                Err(anyhow!(
                    "No config file found in current or parent directories"
                ))
            }
        }
    }

    fn read(path: PathBuf) -> Result<Config> {
        let settings = config::Config::builder()
            .add_source(config::File::from(path.clone()))
            .add_source(config::Environment::with_prefix("NMCR"))
            .build()?;

        let user = settings.try_deserialize::<ConfigUser>()?;

        Ok(Self::new(path, user))
    }

    pub fn write(&self) -> Result<()> {
        let parent = self
            .path
            .parent()
            .map(|p| p.to_path_buf())
            .with_context(|| {
                format!("Failed to get parent directory of {}", self.path.display())
            })?;
        if !parent.exists() {
            fs::create_dir_all(&parent).with_context(|| {
                format!("Failed to create directories for {}", parent.display())
            })?;
        }

        let toml = toml::to_string_pretty(&self.user)
            .with_context(|| format!("Failed to serialize config"))?;
        fs::write(&self.path, toml)
            .with_context(|| format!("Failed to write config at {}", self.path.display()))?;

        Ok(())
    }

    pub fn default_templates_glob() -> String {
        DEFAULT_TEMPLATES_GLOB.to_string()
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from(CONFIG_FILENAME)
    }
}
