mod adapters;
mod commands;
mod error;
mod execute;
mod prov;
mod utils;

use clap::Parser;
use env_logger::Env;
use execute::Execute;
use std::io::Write;

use crate::commands::AgentCli;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = AgentCli::parse();

    // Determine log level based on verbosity flags
    let log_level = if cli.quiet {
        "error".to_string()
    } else {
        match cli.verbose {
            0 => "info".to_string(),
            1 => "debug".to_string(),
            _ => "trace".to_string(),
        }
    };

    env_logger::Builder::from_env(Env::default().default_filter_or(log_level))
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let ctx = cli.resolve_context();
    if let Err(e) = cli.execute(ctx).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
