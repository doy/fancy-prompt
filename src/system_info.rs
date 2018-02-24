use chrono;
use hostname;
use term_size;
use std;
use users;

use power;
use vcs;

pub fn hostname() -> Option<String> {
    hostname::get_hostname()
}

pub fn terminal_cols() -> Option<usize> {
    if let Some((w, _h)) = term_size::dimensions() {
        Some(w)
    }
    else {
        None
    }
}

pub fn pwd() -> Option<std::path::PathBuf> {
    std::env::var("PWD").map(std::path::PathBuf::from).ok()
}

pub fn home() -> Option<std::path::PathBuf> {
    std::env::var("HOME").map(std::path::PathBuf::from).ok()
}

pub fn user() -> Option<String> {
    users::get_current_username()
}

pub fn is_root() -> bool {
    users::get_current_uid() == 0
}

pub fn time() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

pub fn power_info() -> power::PowerInfo {
    power::PowerInfo::new()
}

pub fn vcs_info() -> Option<Box<vcs::VcsInfo>> {
    vcs::detect()
}
