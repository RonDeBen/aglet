use crate::commands::CommandContext;
use crate::commands::policy::store::{PolicyMode, PolicyStore, policy_key, scaffold_policy};
use crate::error::{CliError, Result};
use crate::execute::Execute;
use clap::{Args, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum PolicyModeArg {
    Always,
    Optional,
}

impl From<PolicyModeArg> for PolicyMode {
    fn from(value: PolicyModeArg) -> Self {
        match value {
            PolicyModeArg::Always => PolicyMode::Always,
            PolicyModeArg::Optional => PolicyMode::Optional,
        }
    }
}

#[derive(Args)]
pub struct AddPolicyCommand {
    pub name: String,

    #[arg(long)]
    pub summary: Option<String>,

    #[arg(long, value_enum, default_value_t = PolicyModeArg::Optional)]
    pub mode: PolicyModeArg,

    #[arg(long = "applies-when")]
    pub applies_when: Vec<String>,

    #[arg(long = "skip-when")]
    pub skip_when: Vec<String>,

    #[arg(long = "rule")]
    pub rules: Vec<String>,

    #[arg(long = "example-good")]
    pub examples_good: Vec<String>,

    #[arg(long = "example-bad")]
    pub examples_bad: Vec<String>,

    #[arg(long)]
    pub rationale: Option<String>,
}

#[async_trait::async_trait]
impl Execute for AddPolicyCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let store = PolicyStore::new(ctx.project_root.path.join(".aglet"));
        let key = policy_key(&self.name);
        if key.is_empty() {
            return Err(CliError::ConfigError(
                "policy name must include at least one letter or number".into(),
            ));
        }

        let mut document = scaffold_policy(key.clone(), self.mode.into());
        if let Some(summary) = &self.summary {
            document.summary = summary.clone();
        }
        if !self.applies_when.is_empty() {
            document.applies_when = self.applies_when.clone();
        }
        if !self.skip_when.is_empty() {
            document.skip_when = self.skip_when.clone();
        }
        if !self.rules.is_empty() {
            document.rules = self.rules.clone();
        }
        if !self.examples_good.is_empty() {
            document.examples_good = self.examples_good.clone();
        }
        if !self.examples_bad.is_empty() {
            document.examples_bad = self.examples_bad.clone();
        }
        if let Some(rationale) = &self.rationale {
            document.rationale = Some(rationale.clone());
        }

        let path = store.create(&key, &document)?;
        println!("created policy: {}", path.display());
        Ok(())
    }
}
