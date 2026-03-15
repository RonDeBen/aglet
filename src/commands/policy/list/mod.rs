use crate::commands::CommandContext;
use crate::commands::policy::store::PolicyStore;
use crate::error::Result;
use crate::execute::Execute;
use clap::Args;

#[derive(Args)]
pub struct ListPolicyCommand {}

#[async_trait::async_trait]
impl Execute for ListPolicyCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let store = PolicyStore::new(ctx.project_root.path.join(".aglet"));
        let policies = store.list()?;

        if policies.is_empty() {
            println!("no policies found in .aglet/policies");
            return Ok(());
        }

        for policy in policies {
            println!(
                "{} [{}] {}",
                policy.key, policy.document.mode, policy.document.summary
            );
        }

        Ok(())
    }
}
