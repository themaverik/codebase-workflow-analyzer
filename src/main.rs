use clap::Parser;
use codebase_analyzer::{Cli, CliRunner};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let runner = CliRunner::new();
    
    match runner.run(cli).await {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}