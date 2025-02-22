use anyhow::{Context as _, Result};
use tokio::signal;

pub async fn sigint() -> Result<()> {
    signal::ctrl_c()
        .await
        .context("Failed to install SIGINT handler")?;

    Ok(())
}

pub async fn sigterm() -> Result<()> {
    signal::unix::signal(signal::unix::SignalKind::terminate())
        .context("Failed to install SIGTERM handler")?
        .recv()
        .await;

    Ok(())
}
