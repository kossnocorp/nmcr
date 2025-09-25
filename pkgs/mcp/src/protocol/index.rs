use crate::prelude::*;

pub trait McpProtocol {
    fn protocol_name(&self) -> &'static str;

    fn serve<'a>(
        &'a self,
        server: TemplateServer,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}
