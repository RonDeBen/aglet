use super::CommandContext;
use crate::commands::policy::store::PolicyStore;
use crate::error::Result;
use crate::execute::Execute;
use crate::prov::ProvStoreJson;
use clap::Args;
use std::fs;
use walkdir::WalkDir;

#[derive(Args)]
pub struct InitCommand {
    /// skip codebase mapping during init
    #[arg(long, default_value_t = false)]
    pub no_map: bool,
}

#[async_trait::async_trait]
impl Execute for InitCommand {
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        let root = ctx.project_root.path;
        let agents_dir = root.join(".aglet");
        fs::create_dir_all(&agents_dir)?;
        fs::create_dir_all(agents_dir.join("commits"))?;
        fs::create_dir_all(agents_dir.join("artifacts"))?;
        fs::create_dir_all(agents_dir.join("checkpoints"))?;
        fs::create_dir_all(agents_dir.join("worktrees"))?;
        fs::create_dir_all(agents_dir.join("memory"))?;
        fs::create_dir_all(agents_dir.join("policies"))?;
        PolicyStore::new(agents_dir.clone()).ensure_default_policies()?;

        // update .gitignore
        let gitignore = root.join(".gitignore");
        if gitignore.exists() {
            let s = fs::read_to_string(&gitignore)?;
            if !s.contains(".aglet/") {
                use std::io::Write;
                let mut f = fs::OpenOptions::new().append(true).open(&gitignore)?;
                writeln!(f, "\n# aglet state\n.aglet/")?;
            }
        } else {
            fs::write(&gitignore, ".aglet/\n")?;
        }

        if !self.no_map {
            // simple map: top-level directories
            let mut modules = vec![];
            for e in WalkDir::new(&root)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if e.depth() == 1 && e.file_type().is_dir() {
                    modules.push(e.file_name().to_string_lossy().to_string());
                }
            }
            let map_summary = format!("modules: {:?}", modules);
            let store = ProvStoreJson::new(agents_dir.clone());
            store.write_map_initial(&map_summary)?;
            log::info!("agent: mapped codebase and wrote map:v1");
        } else {
            log::info!("agent: init completed (mapping skipped)");
        }

        Ok(())
    }
}
