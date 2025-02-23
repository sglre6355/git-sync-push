use clap::Parser;
use derive_more::Debug;
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, env = "GITSYNCPUSH_REPO")]
    pub repo: String,
    #[arg(long, env = "GITSYNCPUSH_PATH")]
    pub path: PathBuf,
    #[arg(long, env = "GITSYNCPUSH_PERIOD", value_parser = humantime::parse_duration)]
    pub period: Duration,
    #[arg(long, env = "GITSYNCPUSH_AUTHOR_NAME")]
    pub author_name: String,
    #[arg(long, env = "GITSYNCPUSH_AUTHOR_EMAIL")]
    pub author_email: String,
    #[arg(long, env = "GITSYNCPUSH_USERNAME")]
    pub username: String,
    #[arg(env = "GITSYNCPUSH_PASSWORD")]
    #[debug("omitted")]
    pub password: String,
    #[arg(long, env = "GITSYNCPUSH_HTTP_BIND")]
    pub http_bind: String,
}
