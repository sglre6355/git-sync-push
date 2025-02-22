mod cli;
mod git;
mod signal_handlers;

use crate::{cli::Args, git::synchronize_repo};
use anyhow::{bail, Result};
use clap::Parser as _;
use git2::Repository;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    // configure global log collector based on RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    debug!("{:?}", args);

    let repo = match Repository::clone(&args.repo, &args.path) {
        Ok(repo) => {
            info!("Repository cloned at {}", args.path.display());
            repo
        }
        Err(error) => bail!("Failed to clone the repository: {}", error),
    };

    synchronize_repo(
        repo,
        args.period,
        args.author_name,
        args.author_email,
        args.username,
        args.password,
    )
    .await?;

    Ok(())
}
