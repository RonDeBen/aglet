use crate::commands::CommandContext;
use crate::error::Result;

#[async_trait::async_trait]
pub trait Execute {
    async fn execute(&self, ctx: CommandContext) -> Result<()>;
}
