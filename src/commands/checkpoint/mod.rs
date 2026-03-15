use super::CommandContext;
use crate::error::Result;
use crate::execute::Execute;
use crate::prov::ProvStoreJson;
use chrono::Utc;
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct CheckpointCommand {
    pub name: String,
}

#[async_trait::async_trait]
impl Execute for CheckpointCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let base = ctx.project_root.path.join(".aglet");
        let store = ProvStoreJson::new(base.clone());
        let commits = store.list_commits()?;
        let mut summary = format!("checkpoint {} - {}\n\n", self.name, Utc::now());
        for c in commits.iter().rev().take(20) {
            summary.push_str(&format!(
                "- {}: {}\n",
                c.id,
                c.output_summary.as_deref().unwrap_or("")
            ));
        }
        let ckpt_path = base.join("checkpoints").join(format!("{}.md", self.name));
        fs::write(ckpt_path, summary)?;
        println!("checkpoint written: .aglet/checkpoints/{}.md", self.name);
        Ok(())
    }
}
