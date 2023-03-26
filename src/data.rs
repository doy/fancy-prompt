use crate::args;
use crate::colors;
use crate::power;
use crate::vcs;

pub struct PromptData {
    pub shell: colors::ShellType,
    pub error_code: u8,
    pub hostname: Option<String>,
    pub terminal_cols: Option<usize>,
    pub pwd: Option<std::path::PathBuf>,
    pub home: Option<std::path::PathBuf>,
    pub user: Option<String>,
    pub is_root: bool,
    pub time: chrono::DateTime<chrono::Local>,
    pub power_info: power::PowerInfo,
    pub vcs_info: Option<Box<dyn vcs::VcsInfo>>,
}

pub fn collect(opts: args::CommandLineOptions) -> PromptData {
    start_talking_about_time!("collecting data");

    let hostname = hostname();
    talk_about_time!("hostname");
    let terminal_cols = terminal_cols();
    talk_about_time!("terminal_cols");
    let pwd = pwd();
    talk_about_time!("pwd");
    let home = home();
    talk_about_time!("home");
    let user = user();
    talk_about_time!("user");
    let is_root = is_root();
    talk_about_time!("is_root");
    let time = time();
    talk_about_time!("time");
    let power_info = power_info();
    talk_about_time!("power_info");
    let vcs_info = vcs_info();
    talk_about_time!("vcs_info");

    stop_talking_about_time!();

    PromptData {
        shell: opts.shell,
        error_code: opts.error_code,
        hostname,
        terminal_cols,
        pwd,
        home,
        user,
        is_root,
        time,
        power_info,
        vcs_info,
    }
}

fn hostname() -> Option<String> {
    if let Ok(name) = hostname::get() {
        let mut name = name.into_string().unwrap();
        if let Some(idx) = name.find('.') {
            name.truncate(idx);
        }
        Some(name)
    } else {
        None
    }
}

fn terminal_cols() -> Option<usize> {
    if let Some((w, _h)) = terminal_size::terminal_size() {
        Some(usize::from(w.0))
    } else {
        None
    }
}

fn pwd() -> Option<std::path::PathBuf> {
    std::env::var("PWD").map(std::path::PathBuf::from).ok()
}

fn home() -> Option<std::path::PathBuf> {
    std::env::var("HOME").map(std::path::PathBuf::from).ok()
}

fn user() -> Option<String> {
    users::get_current_username().map(|s| s.into_string().unwrap())
}

fn is_root() -> bool {
    users::get_current_uid() == 0
}

fn time() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

fn power_info() -> power::PowerInfo {
    power::PowerInfo::new()
}

fn vcs_info() -> Option<Box<dyn vcs::VcsInfo>> {
    vcs::detect()
}
