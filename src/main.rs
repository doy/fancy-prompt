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

mod args;
mod colors;
mod data;
mod power;
mod prompt;
mod sys;
mod vcs;

fn main() {
    start_talking_about_time!("main");
    let opts = args::parse();
    talk_about_time!("parsing args");
    let data = data::collect(opts);
    talk_about_time!("collecting data");
    let w = std::io::stdout();
    prompt::Prompt::new(data).display(w);
    talk_about_time!("displaying data");
    stop_talking_about_time!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_render() {
        let data = data::PromptData {
            shell: colors::ShellType::Unknown,
            error_code: 0,
            hostname: Some(String::from("hush")),
            terminal_cols: Some(80),
            pwd: Some(std::path::PathBuf::from(
                "/home/doy/coding/fancy-prompt",
            )),
            home: Some(std::path::PathBuf::from("/home/doy")),
            user: Some(String::from("doy")),
            is_root: false,
            time: chrono::Local
                .with_ymd_and_hms(2018, 5, 14, 17, 35, 45)
                .unwrap(),
            power_info: power::PowerInfo::new(),
            vcs_info: None,
        };
        let w = vec![];
        prompt::Prompt::new(data).display(w);
    }
}
