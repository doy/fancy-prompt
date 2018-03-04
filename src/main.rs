extern crate chrono;
#[macro_use]
extern crate clap;
extern crate git2;
extern crate hostname;
extern crate regex;
extern crate term;
extern crate term_size;
extern crate users;
extern crate walkdir;

#[macro_use]
mod verbose;

mod colors;
mod power;
mod prompt;
mod system_info;
mod vcs;

fn collect_data() -> prompt::PromptData {
    start_talking_about_time!("collecting data");

    let matches = clap::App::new("fancy-prompt")
        .about("Prints a fancy prompt")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(clap::Arg::with_name("prompt-escape")
             .long("prompt-escape")
             .value_name("SHELL")
             .help("Produces escape sequence wrappers for the given shell")
             .takes_value(true))
        .arg(clap::Arg::with_name("error-code")
             .value_name("ERROR_CODE")
             .help("The error code of the previously run command")
             )
        .get_matches();

    let shell = matches
        .value_of("prompt-escape")
        .map(|shell| colors::ShellType::from_str(shell))
        .unwrap_or(colors::ShellType::Unknown);
    let error_code = matches
        .value_of("error-code")
        .map(|code| code.parse().expect("error code must be a u8"))
        .unwrap_or(0);
    talk_about_time!("command line arg parsing");

    let hostname = system_info::hostname();
    talk_about_time!("hostname");
    let terminal_cols = system_info::terminal_cols();
    talk_about_time!("terminal_cols");
    let pwd = system_info::pwd();
    talk_about_time!("pwd");
    let home = system_info::home();
    talk_about_time!("home");
    let user = system_info::user();
    talk_about_time!("user");
    let is_root = system_info::is_root();
    talk_about_time!("is_root");
    let time = system_info::time();
    talk_about_time!("time");
    let power_info = system_info::power_info();
    talk_about_time!("power_info");
    let vcs_info = system_info::vcs_info();
    talk_about_time!("vcs_info");

    stop_talking_about_time!();

    prompt::PromptData {
        shell,
        error_code,
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

fn main() {
    start_talking_about_time!("main");
    let data = collect_data();
    talk_about_time!("collecting data");
    prompt::Prompt::new(data).display();
    talk_about_time!("displaying data");
    stop_talking_about_time!();
}
