pub mod context;
pub mod init;
pub mod log;
pub mod merge;
pub mod policy;
pub mod run;
pub mod show;

use crate::error::Result;
use crate::execute::Execute;
use crate::utils::fs::ProjectRoot;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aglet")]
#[command(version)]
pub struct AgentCli {
    #[command(subcommand)]
    pub sub_command: AgentSubcommand,

    /// Suppress all output
    #[arg(short, long, global = true)]
    pub quiet: bool,
    /// Increase verbosity (can be used multiple times)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

pub struct CommandContext {
    pub project_root: ProjectRoot,
}

#[derive(Subcommand)]
pub enum AgentSubcommand {
    /// Initialize agent metadata and map codebase
    Init(init::InitCommand),
    /// Run an agent task
    Run(run::RunCommand),
    /// Merge a completed run's branch into the current branch and clean up its worktree
    Merge(merge::MergeCommand),
    /// Show audit log of all agent runs
    Log(log::LogCommand),
    /// Show full details for a run or step by ID
    Show(show::ShowCommand),
    /// Assemble and print agent context for a task
    Context(context::ContextCommand),
    /// Manage modular policies
    Policy(policy::PolicyCommand),
}

impl AgentCli {
    pub fn resolve_context(&self) -> CommandContext {
        CommandContext {
            project_root: ProjectRoot::discover()
                .unwrap_or_else(|_| ProjectRoot::new(std::env::current_dir().unwrap())),
        }
    }
}

#[async_trait::async_trait]
impl Execute for AgentCli {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        match &self.sub_command {
            AgentSubcommand::Init(cmd) => cmd.execute(ctx).await,
            AgentSubcommand::Run(cmd) => cmd.execute(ctx).await,
            AgentSubcommand::Merge(cmd) => cmd.execute(ctx).await,
            AgentSubcommand::Log(cmd) => cmd.execute(ctx).await,
            AgentSubcommand::Show(cmd) => cmd.execute(ctx).await,
            AgentSubcommand::Context(cmd) => cmd.execute(ctx).await,
            AgentSubcommand::Policy(cmd) => cmd.execute(ctx).await,
        }
    }
}
