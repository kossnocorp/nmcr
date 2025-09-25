use crate::prelude::*;

#[derive(Clone)]
pub struct TemplateServer {
    tool_router: ToolRouter<Self>,
    instructions: Option<String>,
}

impl TemplateServer {
    pub(crate) fn new(tool_router: ToolRouter<Self>, instructions: Option<String>) -> Self {
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

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = ToolCallContext::new(self, request, context);
        self.tool_router.call(ctx).await
    }
}
