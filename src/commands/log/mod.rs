use super::CommandContext;
use crate::error::Result;
use crate::execute::Execute;
use crate::prov::ProvStoreJson;
use clap::Args;

#[derive(Args)]
pub struct LogCommand {
    #[arg(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl Execute for LogCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let store = ProvStoreJson::new(ctx.project_root.path.join(".aglet"));
        let commits = store.list_commits()?;
        if self.json {
            println!("{}", serde_json::to_string_pretty(&commits)?);
        } else {
            for c in commits {
                println!(
                    "{} {} {}",
                    c.id,
                    c.agent_id,
                    c.output_summary.unwrap_or_default()
                );
            }
        }
        Ok(())
    }
}
