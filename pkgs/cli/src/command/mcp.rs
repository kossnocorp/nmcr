use crate::prelude::*;
use nmcr_mcp::prelude::*;

#[derive(Args, Debug, Default)]
pub struct McpArgs {}

#[derive(Args)]
pub struct McpCmd;

impl McpCmd {
    pub async fn run(args: &CliCommandProject<McpArgs>) -> Result<()> {
        let project = args.load_project()?;
        McpProtocolStdio.run(&project).await
    }
}
