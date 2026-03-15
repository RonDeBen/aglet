use super::CommandContext;
use crate::commands::policy::store::PolicyStore;
use crate::error::Result;
use crate::execute::Execute;
use crate::workspace::store::{WorkspaceDocument, WorkspaceStore, workspace_doc_id};
use chrono::Utc;
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
        fs::create_dir_all(agents_dir.join("objects"))?;
        fs::create_dir_all(agents_dir.join("policies"))?;
        fs::create_dir_all(agents_dir.join("refs"))?;
        fs::create_dir_all(agents_dir.join("runs"))?;
        fs::create_dir_all(agents_dir.join("steps"))?;
        fs::create_dir_all(agents_dir.join("workspace"))?;
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
            let now = Utc::now();
            let map_contents = format!(
                "# Codebase Map\n\nRoot: {}\n\nTop-level directories:\n{}\n",
                root.display(),
                modules
                    .iter()
                    .map(|module| format!("- {}", module))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            let workspace_store = WorkspaceStore::new(agents_dir.clone());
            let doc_id = workspace_doc_id("codebase-map", ".", now);
            let artifact_ref = workspace_store.write_object(
                &format!("{}-artifact", doc_id),
                "md",
                &map_contents,
            )?;
            let document = WorkspaceDocument {
                id: doc_id.clone(),
                scope_path: ".".into(),
                kind: "codebase-map".into(),
                summary: format!("Top-level codebase map for {} modules.", modules.len()),
                artifact_ref,
                updated_at: now,
            };
            workspace_store.write_document(&document)?;
            workspace_store.write_ref("workspace-map", &doc_id)?;
            log::info!("agent: mapped codebase and wrote map:v1");
        } else {
            log::info!("agent: init completed (mapping skipped)");
        }

        Ok(())
    }
}
