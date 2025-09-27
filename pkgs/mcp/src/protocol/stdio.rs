use crate::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct McpProtocolStdio;

impl McpProtocolStdio {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, project: &Project) -> Result<()> {
        let template_paths = project
            .template_paths()
            .with_context(|| "Failed to collect template files from project")?;

        let catalog = TemplateCatalog::load(&template_paths)?;
        if catalog.is_empty() {
            return Err(anyhow!("No templates found in the project"));
        }

        let mut tool_router = ToolRouter::new();
        for tool in catalog.file_tools() {
            tool_router.add_route(tool.route());
        }
        for tree in catalog.tree_tools() {
            tool_router.add_route(tree.route());
        }

        let instructions = catalog.instructions();
        let server = TemplateServer::new(tool_router, instructions);

        self.serve(server).await
    }
}

impl McpProtocol for McpProtocolStdio {
    fn protocol_name(&self) -> &'static str {
        "stdio"
    }

    fn serve<'a>(
        &'a self,
        server: TemplateServer,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let service = server
                .serve(stdio())
                .await
                .with_context(|| "Failed to start MCP stdio transport")?;

            service
                .waiting()
                .await
                .context("MCP server task terminated unexpectedly")?;

            Ok(())
        })
    }
}
