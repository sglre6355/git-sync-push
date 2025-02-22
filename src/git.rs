use crate::signal_handlers::{sigint, sigterm};
use anyhow::Result;
use chrono::Utc;
use git2::{Cred, IndexAddOption, Oid, PushOptions, RemoteCallbacks, Repository, Signature};
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

trait GitSyncPush {
    fn has_staged_changes(&self) -> Result<bool>;
    fn add_changes(&mut self) -> Result<()>;
    fn commit_staged_changes(&mut self, author_name: &str, author_email: &str) -> Result<Oid>;
    fn push_commits(&mut self, username: &str, password: &str) -> Result<()>;
}

impl GitSyncPush for Repository {
    fn has_staged_changes(&self) -> Result<bool> {
        let head_tree = self.head()?.peel_to_commit()?.tree().ok();
        let diff = self.diff_tree_to_index(head_tree.as_ref(), None, None)?;

        Ok(diff.deltas().len() != 0)
    }

    fn add_changes(&mut self) -> Result<()> {
        let mut index = self.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;

        Ok(())
    }

    fn commit_staged_changes(&mut self, author_name: &str, author_email: &str) -> Result<Oid> {
        let mut index = self.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.find_tree(tree_oid)?;

        let signature = Signature::now(author_name, author_email)?;
        let commit_message = format!(
            "Auto-sync: snapshot at {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        let parent_commits = match self.head() {
            Ok(head_ref) => {
                let head_commit = head_ref.peel_to_commit()?;
                vec![head_commit]
            }
            Err(_) => Vec::new(), // No parent commit yet (e.g. initial commit)
        };

        let commit_id = self.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &commit_message,
            &tree,
            &parent_commits.iter().collect::<Vec<_>>(),
        )?;

        Ok(commit_id)
    }

    fn push_commits(&mut self, username: &str, password: &str) -> Result<()> {
        // Configure authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(username, password)
        });
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let mut remote = self.find_remote("origin")?;
        remote.push(
            &["refs/heads/main:refs/heads/main"],
            Some(&mut push_options),
        )?;

        Ok(())
    }
}

pub async fn synchronize_repo(
    mut repo: Repository,
    sync_period: Duration,
    author_name: String,
    author_email: String,
    username: String,
    password: String,
) -> Result<()> {
    let mut periodic_timer = interval(sync_period);

    loop {
        tokio::select! {
            _ = periodic_timer.tick() => {
                repo.add_changes()?;

                if !repo.has_staged_changes()? {
                    info!("No changes detected, skipping");
                    continue;
                }

                match repo.commit_staged_changes(&author_name, &author_email) {
                    Ok(commit_id) => info!("Changes have been commited: {}", commit_id),
                    Err(error) => {
                        error!("Failed to commit changes: {}", error);
                        // Skip push upon commit errors
                        continue
                    }
                }

                match repo.push_commits(&username, &password) {
                    Ok(_) => info!("Changes have been pushed to the remote"),
                    Err(error) => error!("Failed to push changes to the remote: {}", error),
                }
            }

            // Exit the loop upon receiving a termination signal
            _ = sigint() => {
                break;
            }
            _ = sigterm() => {
                break;
            }
        }
    }

    info!("Signal received, finishing up...");

    // Commit and push any changes before terminating
    repo.add_changes()?;
    if repo.has_staged_changes()? {
        match repo.commit_staged_changes(&author_name, &author_email) {
            Ok(commit_id) => {
                info!("Changes have been commited: {}", commit_id);

                // Only push when changes are committed
                match repo.push_commits(&username, &password) {
                    Ok(_) => info!("Changes have been pushed to the remote"),
                    Err(error) => error!("Failed to push changes to the remote: {}", error),
                }
            }
            Err(error) => error!("Failed to commit changes: {}", error),
        }
    }

    Ok(())
}
