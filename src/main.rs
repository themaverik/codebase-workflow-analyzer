use clap::Parser;
use codebase_analyzer::{Cli, CliRunner};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let runner = CliRunner::new();
    
    // Create a minimal async runtime just for this call
    let rt = tokio::runtime::Runtime::new()?;
    match rt.block_on(runner.run(cli)) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}