use super::CommandContext;
use crate::error::Result;
use crate::execute::Execute;
use crate::prov::ProvStoreJson;
use clap::Args;

#[derive(Args)]
pub struct ShowCommand {
    pub commit_id: String,
}

#[async_trait::async_trait]
impl Execute for ShowCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let store = ProvStoreJson::new(ctx.project_root.path.join(".aglet"));
        let c = store.read_commit(&self.commit_id)?;
        println!("{}", serde_json::to_string_pretty(&c)?);
        Ok(())
    }
}
