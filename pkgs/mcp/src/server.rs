use crate::{prelude::*, template::TemplateCatalog};

pub async fn run_stdio(project_path: Option<PathBuf>) -> Result<()> {
    let project = Project::load(project_path)?;
    let template_paths = project
        .template_paths()
        .with_context(|| "Failed to collect template files from project")?;

    let catalog = TemplateCatalog::load(&template_paths)?;
    if catalog.is_empty() {
        return Err(anyhow!("No templates found in the project"));
    }

    let mut tool_router = ToolRouter::new();
    for tool in catalog.tools() {
        tool_router.add_route(tool.route());
    }

    let instructions = catalog.instructions();
    let server = TemplateServer::new(tool_router, instructions);

    let service = server
        .serve(stdio())
        .await
        .with_context(|| "Failed to start MCP stdio transport")?;

    service
        .waiting()
        .await
        .context("MCP server task terminated unexpectedly")?;

    Ok(())
}

#[derive(Clone)]
struct TemplateServer {
    tool_router: ToolRouter<Self>,
    instructions: Option<String>,
}

impl TemplateServer {
    fn new(tool_router: ToolRouter<Self>, instructions: Option<String>) -> Self {
        Self {
            tool_router,
            instructions,
        }
    }
}

impl ServerHandler for TemplateServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: self.instructions.clone(),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let tools = self.tool_router.list_all();
        async move {
            Ok(ListToolsResult {
                tools,
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let ctx = ToolCallContext::new(self, request, context);
            self.tool_router.call(ctx).await
        }
    }
}
