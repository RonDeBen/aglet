pub mod add;
pub mod edit;
pub mod list;
pub mod show;
pub mod store;

use super::CommandContext;
use crate::error::Result;
use crate::execute::Execute;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct PolicyCommand {
    #[command(subcommand)]
    pub sub_command: PolicySubcommand,
}

#[derive(Subcommand)]
pub enum PolicySubcommand {
    /// List available policies
    List(list::ListPolicyCommand),
    /// Show a policy definition
    Show(show::ShowPolicyCommand),
    /// Create a new policy file
    Add(add::AddPolicyCommand),
    /// Open a policy in your editor
    Edit(edit::EditPolicyCommand),
}

#[async_trait::async_trait]
impl Execute for PolicyCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        match &self.sub_command {
            PolicySubcommand::List(cmd) => cmd.execute(ctx).await,
            PolicySubcommand::Show(cmd) => cmd.execute(ctx).await,
            PolicySubcommand::Add(cmd) => cmd.execute(ctx).await,
            PolicySubcommand::Edit(cmd) => cmd.execute(ctx).await,
        }
    }
}
