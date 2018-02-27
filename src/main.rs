extern crate chrono;
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
    let matches = clap::App::new("fancy-prompt")
        .about("Prints a fancy prompt")
        // XXX author, version (extract from cargo)
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

    prompt::PromptData {
        shell: matches
            .value_of("prompt-escape")
            .map(|shell| colors::ShellType::from_str(shell))
            .unwrap_or(colors::ShellType::Unknown),
        error_code: matches
            .value_of("error-code")
            .map(|code| code.parse().expect("error code must be a u8"))
            .unwrap_or(0),

        hostname: system_info::hostname(),
        terminal_cols: system_info::terminal_cols(),
        pwd: system_info::pwd(),
        home: system_info::home(),
        user: system_info::user(),
        is_root: system_info::is_root(),
        time: system_info::time(),
        power_info: system_info::power_info(),
        vcs_info: system_info::vcs_info(),
    }
}

fn main() {
    let data = collect_data();
    prompt::Prompt::new(data).display();
}
