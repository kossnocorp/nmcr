pub use std::path::{Path, PathBuf};
pub use std::process::exit;
pub use std::sync::LazyLock;

pub use anyhow::{Context, Result, anyhow, bail};
pub use clap::Parser;
pub use console::{StyledObject, style};
pub use dialoguer::{
    Confirm, Input, Password,
    theme::{ColorfulTheme, Theme},
};
pub use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
pub use serde::{Deserialize, Serialize};

pub use crate::cli::*;
pub use crate::command::*;
pub use crate::ui::*;

pub use nmcr_project::prelude::*;
