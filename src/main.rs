mod cli;
mod git;
mod health_check;

use crate::{
    cli::Args,
    git::GitSyncPush,
    health_check::{serve_health_endpoints, AppState},
};
use anyhow::Result;
use clap::Parser as _;
use git2::Repository;
use std::sync::Arc;
use tokio::{signal, sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

fn signal_handler(token: CancellationToken) -> Result<JoinHandle<()>> {
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;

    Ok(tokio::task::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => {
                info!("Received SIGINT, terminating...");
                token.cancel();
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, terminating...");
                token.cancel();
            }
        }
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    // configure global log collector based on RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    debug!("{:?}", args);

    let token = CancellationToken::new();

    let shared_state = Arc::new(Mutex::new(AppState {
        is_repo_ready: false,
    }));

    let api_handle = tokio::task::spawn({
        let token = token.clone();
        let shared_state = shared_state.clone();
        async move {
            tokio::select! {
                _ = token.cancelled() => {}
                _ = serve_health_endpoints(args.http_bind, shared_state) => {}
            }
        }
    });

    info!("Cloning repository...");

    let mut repo = match Repository::clone(&args.repo, &args.path) {
        Ok(repo) => repo,
        Err(error) => {
            error!("Failed to clone the repository: {}", error);
            return Err(error.into());
        }
    };

    info!("Repository cloned at {}", args.path.display());
    // mark the repository as ready for use
    shared_state.lock().await.is_repo_ready = true;

    let sync_handle = tokio::task::spawn({
        let token = token.clone();
        async move {
            repo.synchronize(
                token,
                args.period,
                args.author_name,
                args.author_email,
                args.username,
                args.password,
            )
            .await
        }
    });

    let signal_handle = signal_handler(token)?;

    let (api_result, sync_result, signal_result) =
        tokio::join!(api_handle, sync_handle, signal_handle);

    api_result?;
    sync_result??;
    signal_result?;

    Ok(())
}
