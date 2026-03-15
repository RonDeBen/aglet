use crate::commands::CommandContext;
use crate::commands::policy::store::{PolicyMode, PolicyStore, policy_key, scaffold_policy};
use crate::error::{CliError, Result};
use crate::execute::Execute;
use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct EditPolicyCommand {
    pub name: Option<String>,
}

#[async_trait::async_trait]
impl Execute for EditPolicyCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let store = PolicyStore::new(ctx.project_root.path.join(".aglet"));
        store.ensure_dir()?;

        let target = if let Some(name) = &self.name {
            let key = policy_key(name);
            if key.is_empty() {
                return Err(CliError::ConfigError(
                    "policy name must include at least one letter or number".into(),
                ));
            }

            let path = store.path_for(&key);
            if !path.exists() {
                let document = scaffold_policy(key.clone(), PolicyMode::Optional);
                store.create(&key, &document)?;
            }
            path
        } else {
            store.dir()
        };

        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .map_err(|_| {
                CliError::ConfigError("set $VISUAL or $EDITOR before using policy edit".into())
            })?;

        let mut parts = shlex::split(&editor).ok_or_else(|| {
            CliError::ConfigError("could not parse $VISUAL or $EDITOR command".into())
        })?;
        if parts.is_empty() {
            return Err(CliError::ConfigError(
                "$VISUAL or $EDITOR cannot be empty".into(),
            ));
        }

        let program = parts.remove(0);
        let status = Command::new(program).args(parts).arg(&target).status()?;
        if !status.success() {
            return Err(CliError::Other(anyhow::anyhow!(
                "editor exited with status {status}"
            )));
        }

        Ok(())
    }
}
