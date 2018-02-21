use git2;
use std;

use std::fmt::Write;

#[derive(Debug)]
pub struct GitInfo {
    modified_files: bool,
    staged_files: bool,
    new_files: bool,
    commits: bool,
    active_operation: super::ActiveOperation,
    branch: Option<String>,
    remote_branch_diff: Option<(usize, usize)>,
}

impl GitInfo {
    pub fn new(git: &git2::Repository) -> GitInfo {
        let mut modified_statuses = git2::Status::empty();
        modified_statuses.insert(git2::STATUS_WT_DELETED);
        modified_statuses.insert(git2::STATUS_WT_MODIFIED);
        modified_statuses.insert(git2::STATUS_WT_RENAMED);
        modified_statuses.insert(git2::STATUS_WT_TYPECHANGE);
        let mut staged_statuses = git2::Status::empty();
        staged_statuses.insert(git2::STATUS_INDEX_DELETED);
        staged_statuses.insert(git2::STATUS_INDEX_MODIFIED);
        staged_statuses.insert(git2::STATUS_INDEX_NEW);
        staged_statuses.insert(git2::STATUS_INDEX_RENAMED);
        staged_statuses.insert(git2::STATUS_INDEX_TYPECHANGE);
        let mut new_statuses = git2::Status::empty();
        new_statuses.insert(git2::STATUS_WT_NEW);

        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        if true { // XXX
            status_options.update_index(true);
        }
        else {
            status_options.update_index(false);
            status_options.no_refresh(true);
        }
        let status = git.statuses(Some(&mut status_options));
        let mut modified_files = false;
        let mut staged_files = false;
        let mut new_files = false;
        for status in status.iter() {
            for file in status.iter() {
                if file.status().intersects(modified_statuses) {
                    modified_files = true;
                }
                if file.status().intersects(staged_statuses) {
                    staged_files = true;
                }
                if file.status().intersects(new_statuses) {
                    new_files = true;
                }
            }
        }

        let head = git.head();
        let commits = head.is_ok();
        let branch = head.ok()
            .and_then(|head| {
                if head.is_branch() {
                    head.shorthand().map(|s| s.to_string())
                }
                else {
                    head.resolve().ok()
                        .and_then(|head| head.target())
                        .map(|oid| {
                            let mut sha = String::new();
                            for b in oid.as_bytes().iter() {
                                write!(sha, "{:02x}", b).unwrap();
                            }
                            sha.truncate(7);
                            sha
                        })
                }
            });

        let active_operation = match git.state() {
            git2::RepositoryState::Merge
                => super::ActiveOperation::Merge,
            git2::RepositoryState::Revert
                | git2::RepositoryState::RevertSequence
                => super::ActiveOperation::Revert,
            git2::RepositoryState::CherryPick
                | git2::RepositoryState::CherryPickSequence
                => super::ActiveOperation::CherryPick,
            git2::RepositoryState::Bisect
                => super::ActiveOperation::Bisect,
            git2::RepositoryState::Rebase
                | git2::RepositoryState::RebaseInteractive
                | git2::RepositoryState::RebaseMerge
                => super::ActiveOperation::Rebase,
            _ => super::ActiveOperation::None,
        };

        let remote_branch_diff = git.head().ok()
            .and_then(|head| if head.is_branch() { Some(head) } else { None })
            .and_then(|head| {
                head.resolve().ok()
            })
            .map(|head| {
                (head.target(), head.shorthand().map(|s| s.to_string()))
            })
            .and_then(|(head_id, name)| {
                head_id.and_then(|head_id| {
                    name.and_then(|name| {
                        git.refname_to_id(
                            &(String::from("refs/remotes/origin/") + &name)
                        ).ok().and_then(|remote_id| {
                            git.graph_ahead_behind(head_id, remote_id).ok()
                        })
                    })
                })
            });

        GitInfo {
            modified_files: modified_files,
            staged_files: staged_files,
            new_files: new_files,
            commits: commits,
            active_operation: active_operation,
            branch: branch,
            remote_branch_diff: remote_branch_diff,
        }
    }
}

impl super::VcsInfo for GitInfo {
    fn vcs(&self) -> super::VcsType {
        super::VcsType::Git
    }

    fn has_modified_files(&self) -> bool {
        self.modified_files
    }

    fn has_staged_files(&self) -> bool {
        self.staged_files
    }

    fn has_new_files(&self) -> bool {
        self.new_files
    }

    fn has_commits(&self) -> bool {
        self.commits
    }

    fn active_operation(&self) -> super::ActiveOperation {
        self.active_operation
    }

    fn branch(&self) -> Option<String> {
        self.branch.clone()
    }

    fn remote_branch_diff(&self) -> Option<(usize, usize)> {
        self.remote_branch_diff
    }
}

pub fn detect() -> Option<Box<super::VcsInfo>> {
    let git = std::env::current_dir().ok().and_then(|pwd| {
        git2::Repository::discover(pwd).ok()
    });

    if let Some(git) = git {
        Some(Box::new(GitInfo::new(&git)))
    }
    else {
        None
    }
}
