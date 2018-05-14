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

mod args;
mod colors;
mod data;
mod power;
mod prompt;
mod vcs;

fn main() {
    start_talking_about_time!("main");
    let opts = args::parse();
    talk_about_time!("parsing args");
    let data = data::collect(opts);
    talk_about_time!("collecting data");
    prompt::Prompt::new(data).display();
    talk_about_time!("displaying data");
    stop_talking_about_time!();
}
