use crate::colors;

pub struct CommandLineOptions {
    pub shell: colors::ShellType,
    pub error_code: u8,
}

pub fn parse() -> CommandLineOptions {
    let matches = clap::Command::new("fancy-prompt")
        .about("Prints a fancy prompt")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .arg(
            clap::Arg::new("prompt-escape")
                .long("prompt-escape")
                .value_name("SHELL")
                .help(
                    "Produces escape sequence wrappers for the given shell",
                ),
        )
        .arg(
            clap::Arg::new("error-code")
                .value_name("ERROR_CODE")
                .value_parser(clap::value_parser!(u8))
                .help("The error code of the previously run command"),
        )
        .get_matches();

    let shell = matches
        .get_one::<String>("prompt-escape")
        .map(|s| colors::ShellType::from_str(s))
        .unwrap_or(colors::ShellType::Unknown);
    let error_code =
        matches.get_one::<u8>("error-code").copied().unwrap_or(0);

    CommandLineOptions { shell, error_code }
}
