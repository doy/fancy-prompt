mod git;

pub enum VcsType {
    Git,
}

#[derive(Debug,Copy,Clone)]
pub enum ActiveOperation {
    None,
    Merge,
    Revert,
    CherryPick,
    Bisect,
    Rebase,
}

pub trait VcsInfo {
    fn vcs(&self) -> VcsType;
    fn has_modified_files(&self) -> bool;
    fn has_staged_files(&self) -> bool;
    fn has_new_files(&self) -> bool;
    fn has_commits(&self) -> bool;
    fn active_operation(&self) -> ActiveOperation;
    fn branch(&self) -> Option<String>;
    fn remote_branch_diff(&self) -> Option<(usize, usize)>;
}

pub fn detect() -> Option<Box<VcsInfo>> {
    if let Some(git) = git::detect() {
        Some(git)
    }
    else {
        None
    }
}
