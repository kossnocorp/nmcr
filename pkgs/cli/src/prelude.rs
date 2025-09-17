pub use std::path::PathBuf;
pub use std::process::exit;
pub use std::sync::LazyLock;

pub use anyhow::Result;
pub use clap::Parser;
pub use console::{StyledObject, style};
pub use dialoguer::{
    Input,
    theme::{ColorfulTheme, Theme},
};
pub use indicatif::{ProgressBar, ProgressStyle};

pub use crate::cli::*;
pub use crate::command::*;
pub use crate::ui::*;

pub use nmcr_project::prelude::*;
