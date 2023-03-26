use crate::colors;

pub struct CommandLineOptions {
    pub shell: colors::ShellType,
    pub error_code: u8,
}

pub fn parse() -> CommandLineOptions {
    let matches = clap::App::new("fancy-prompt")
        .about("Prints a fancy prompt")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            clap::Arg::with_name("prompt-escape")
                .long("prompt-escape")
                .value_name("SHELL")
                .help("Produces escape sequence wrappers for the given shell")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("error-code")
                .value_name("ERROR_CODE")
                .help("The error code of the previously run command"),
        )
        .get_matches();

    let shell = matches
        .value_of("prompt-escape")
        .map(colors::ShellType::from_str)
        .unwrap_or(colors::ShellType::Unknown);
    let error_code = matches
        .value_of("error-code")
        .map(|code| code.parse().expect("error code must be a u8"))
        .unwrap_or(0);

    CommandLineOptions { shell, error_code }
}
