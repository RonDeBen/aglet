use crate::commands::CommandContext;
use crate::commands::policy::store::PolicyStore;
use crate::error::Result;
use crate::execute::Execute;
use clap::Args;

#[derive(Args)]
pub struct ShowPolicyCommand {
    pub name: String,
}

#[async_trait::async_trait]
impl Execute for ShowPolicyCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let store = PolicyStore::new(ctx.project_root.path.join(".aglet"));
        let policy = store.read(&self.name)?;

        println!("name: {}", policy.document.name);
        println!("key: {}", policy.key);
        println!("mode: {}", policy.document.mode);
        println!("path: {}", policy.path.display());
        println!();
        println!("summary: {}", policy.document.summary);

        if !policy.document.applies_when.is_empty() {
            println!();
            println!("applies_when:");
            for item in policy.document.applies_when {
                println!("- {}", item);
            }
        }

        if !policy.document.skip_when.is_empty() {
            println!();
            println!("skip_when:");
            for item in policy.document.skip_when {
                println!("- {}", item);
            }
        }

        if !policy.document.rules.is_empty() {
            println!();
            println!("rules:");
            for item in policy.document.rules {
                println!("- {}", item);
            }
        }

        if !policy.document.examples_good.is_empty() {
            println!();
            println!("examples_good:");
            for item in policy.document.examples_good {
                println!("- {}", item);
            }
        }

        if !policy.document.examples_bad.is_empty() {
            println!();
            println!("examples_bad:");
            for item in policy.document.examples_bad {
                println!("- {}", item);
            }
        }

        if let Some(rationale) = policy.document.rationale {
            println!();
            println!("rationale: {}", rationale);
        }

        Ok(())
    }
}
