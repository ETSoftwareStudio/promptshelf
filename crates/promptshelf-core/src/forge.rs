//! The only host-specific parts of team libraries: open a pull request and read
//! its state. Everything else is plain git, so any git host works.

use std::path::Path;
use std::process::Stdio;

use serde::{Deserialize, Serialize};

use crate::config::LibraryConfig;
use crate::error::Result;
use crate::git;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Forge {
    Github,
    Gitlab,
    /// Gitea and Forgejo: pull requests through AGit push options.
    Gitea,
    /// Any other host: push the branch and link to it.
    Plain,
}

impl Forge {
    /// The library's `forge` setting, or a guess from the remote URL.
    pub fn of(lib: &LibraryConfig, remote_url: &str) -> Forge {
        match lib.forge.as_deref() {
            Some("github") => return Forge::Github,
            Some("gitlab") => return Forge::Gitlab,
            Some("gitea") | Some("forgejo") => return Forge::Gitea,
            Some(_) => return Forge::Plain,
            None => {}
        }
        if remote_url.contains("github.com") {
            Forge::Github
        } else if remote_url.contains("gitlab") {
            Forge::Gitlab
        } else if remote_url.contains("gitea") || remote_url.contains("codeberg") {
            Forge::Gitea
        } else {
            Forge::Plain
        }
    }
}

/// `owner/repo` from an SSH or HTTPS remote URL.
pub fn repo_path(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("git@")
        .and_then(|r| r.split_once(':').map(|(_, p)| p))
        .or_else(|| {
            url.split_once("://")
                .and_then(|(_, r)| r.split_once('/').map(|(_, p)| p))
        })?;
    let path = rest.trim_end_matches('/').trim_end_matches(".git");
    (path.contains('/')).then(|| path.to_string())
}

/// The web address of the host, for links: `https://github.com/owner/repo`.
pub fn web_url(url: &str) -> Option<String> {
    let host = if let Some(r) = url.strip_prefix("git@") {
        r.split_once(':')?.0.to_string()
    } else {
        let after = url.split_once("://")?.1;
        let host = after.split_once('/')?.0;
        host.rsplit_once('@').map_or(host, |(_, h)| h).to_string()
    };
    Some(format!("https://{host}/{}", repo_path(url)?))
}

/// What happened when a proposal was sent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sent {
    /// The pull request number, when the host returned one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u64>,
    /// The pull request page, or a page where you can open one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

pub struct Request<'a> {
    pub branch: &'a str,
    pub base: &'a str,
    pub title: &'a str,
    pub body: &'a str,
}

fn has_tool(name: &str) -> bool {
    crate::tools::command(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Pushes the proposal and opens its pull request where the host allows it.
pub fn open(forge: Forge, worktree: &Path, remote_url: &str, req: &Request) -> Result<Sent> {
    match forge {
        Forge::Gitlab => {
            let title = format!("merge_request.title={}", req.title);
            let target = format!("merge_request.target={}", req.base);
            git::run(
                worktree,
                &[
                    "push",
                    "-q",
                    "-u",
                    "origin",
                    req.branch,
                    "-o",
                    "merge_request.create",
                    "-o",
                    &target,
                    "-o",
                    &title,
                ],
            )?;
            Ok(Sent {
                number: None,
                url: web_url(remote_url).map(|u| format!("{u}/-/merge_requests")),
            })
        }
        Forge::Gitea => {
            let refspec = format!(
                "HEAD:refs/for/{}/{}",
                req.base,
                req.branch.replace('/', "-")
            );
            let title = format!("title={}", req.title);
            let desc = format!("description={}", req.body);
            git::run(
                worktree,
                &["push", "-q", "origin", &refspec, "-o", &title, "-o", &desc],
            )?;
            // AGit keeps no branch on the host, so keep one for merge checks.
            let _ = git::try_run(worktree, &["push", "-q", "-u", "origin", req.branch]);
            Ok(Sent {
                number: None,
                url: web_url(remote_url).map(|u| format!("{u}/pulls")),
            })
        }
        Forge::Github => {
            git::run(worktree, &["push", "-q", "-u", "origin", req.branch])?;
            let repo = repo_path(remote_url);
            if has_tool("gh") {
                if let Some(repo) = &repo {
                    let out = crate::tools::command("gh")
                        .args(["pr", "create", "--repo", repo, "--head", req.branch])
                        .args(["--base", req.base, "--title", req.title, "--body", req.body])
                        .current_dir(worktree)
                        .stdin(Stdio::null())
                        .output();
                    if let Some(out) = out.ok().filter(|o| o.status.success()) {
                        let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        let number = url.rsplit('/').next().and_then(|n| n.parse().ok());
                        return Ok(Sent {
                            number,
                            url: Some(url),
                        });
                    }
                }
            }
            // No gh login: link to GitHub's "open a pull request" page.
            Ok(Sent {
                number: None,
                url: web_url(remote_url)
                    .map(|u| format!("{u}/compare/{}...{}?expand=1", req.base, req.branch)),
            })
        }
        Forge::Plain => {
            git::run(worktree, &["push", "-q", "-u", "origin", req.branch])?;
            Ok(Sent::default())
        }
    }
}

/// The host's answer about a pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostState {
    Open,
    ChangesRequested,
    Merged,
    Closed,
}

/// Reads a pull request's state through the host's CLI. `None` when there is
/// no CLI, no login or no number: the caller then checks with plain git.
pub fn state(forge: Forge, remote_url: &str, number: Option<u64>) -> Option<HostState> {
    let number = number?;
    if forge != Forge::Github || !has_tool("gh") {
        return None;
    }
    let repo = repo_path(remote_url)?;
    let out = crate::tools::command("gh")
        .args(["pr", "view", &number.to_string(), "--repo", &repo])
        .args(["--json", "state,reviewDecision"])
        .stdin(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct View {
        state: String,
        review_decision: Option<String>,
    }
    let v: View = serde_json::from_slice(&out.stdout).ok()?;
    Some(match (v.state.as_str(), v.review_decision.as_deref()) {
        ("MERGED", _) => HostState::Merged,
        ("CLOSED", _) => HostState::Closed,
        (_, Some("CHANGES_REQUESTED")) => HostState::ChangesRequested,
        _ => HostState::Open,
    })
}

#[cfg(test)]
#[path = "tests/forge.rs"]
mod tests;
