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
        start_talking_about_time!("git");

        let mut modified_statuses = git2::Status::empty();
        modified_statuses.insert(git2::Status::WT_DELETED);
        modified_statuses.insert(git2::Status::WT_MODIFIED);
        modified_statuses.insert(git2::Status::WT_RENAMED);
        modified_statuses.insert(git2::Status::WT_TYPECHANGE);
        modified_statuses.insert(git2::Status::CONFLICTED);
        let mut staged_statuses = git2::Status::empty();
        staged_statuses.insert(git2::Status::INDEX_DELETED);
        staged_statuses.insert(git2::Status::INDEX_MODIFIED);
        staged_statuses.insert(git2::Status::INDEX_NEW);
        staged_statuses.insert(git2::Status::INDEX_RENAMED);
        staged_statuses.insert(git2::Status::INDEX_TYPECHANGE);
        let mut new_statuses = git2::Status::empty();
        new_statuses.insert(git2::Status::WT_NEW);
        talk_about_time!("status bitsets");

        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        if true {
            // XXX
            status_options.update_index(true);
        } else {
            status_options.update_index(false);
            status_options.no_refresh(true);
        }
        talk_about_time!("status options");

        let status = git.statuses(Some(&mut status_options));
        talk_about_time!("statuses");

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
        talk_about_time!("status iteration");

        let head = git.head();
        talk_about_time!("head");

        let commits = head.is_ok();
        let branch = head.ok().and_then(|head| {
            if head.is_branch() {
                head.shorthand().map(|s| s.to_string())
            } else {
                head.resolve().ok().and_then(|head| head.target()).map(
                    |oid| {
                        let mut sha = String::new();
                        for b in oid.as_bytes().iter() {
                            write!(sha, "{:02x}", b).unwrap();
                        }
                        sha.truncate(7);
                        sha
                    },
                )
            }
        });
        talk_about_time!("branch");

        let active_operation = match git.state() {
            git2::RepositoryState::Merge => super::ActiveOperation::Merge,
            git2::RepositoryState::Revert
            | git2::RepositoryState::RevertSequence => {
                super::ActiveOperation::Revert
            }
            git2::RepositoryState::CherryPick
            | git2::RepositoryState::CherryPickSequence => {
                super::ActiveOperation::CherryPick
            }
            git2::RepositoryState::Bisect => super::ActiveOperation::Bisect,
            git2::RepositoryState::Rebase
            | git2::RepositoryState::RebaseInteractive
            | git2::RepositoryState::RebaseMerge => {
                super::ActiveOperation::Rebase
            }
            _ => super::ActiveOperation::None,
        };
        talk_about_time!("active operation");

        let remote_branch_diff = git
            .head()
            .ok()
            .and_then(|head| if head.is_branch() { Some(head) } else { None })
            .and_then(|head| head.resolve().ok())
            .map(|head| {
                (head.target(), head.shorthand().map(|s| s.to_string()))
            })
            .and_then(|(head_id, name)| {
                head_id.and_then(|head_id| {
                    name.and_then(|name| {
                        git.refname_to_id(
                            &(String::from("refs/remotes/origin/") + &name),
                        )
                        .ok()
                        .and_then(|remote_id| {
                            git.graph_ahead_behind(head_id, remote_id).ok()
                        })
                    })
                })
            });
        talk_about_time!("remote branch diff");
        stop_talking_about_time!();

        GitInfo {
            modified_files,
            staged_files,
            new_files,
            commits,
            active_operation,
            branch,
            remote_branch_diff,
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

pub fn detect() -> Option<Box<dyn super::VcsInfo>> {
    start_talking_about_time!("git::detect");

    let git = git2::Repository::open_from_env().ok();
    talk_about_time!("discover");

    stop_talking_about_time!();

    if let Some(git) = git {
        Some(Box::new(GitInfo::new(&git)))
    } else {
        None
    }
}
