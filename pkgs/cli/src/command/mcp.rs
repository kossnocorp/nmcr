use crate::prelude::*;

#[derive(clap::Args, Debug, Default)]
pub struct McpArgs {}

pub struct McpCmd;

impl McpCmd {
    pub async fn run(cli: &Cli, _args: &McpArgs) -> Result<()> {
        let path = cli.resolve_project_path();
        nmcr_mcp::run_stdio(Some(path)).await
    }
}
