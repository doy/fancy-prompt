fn main() {
    println!("cargo:rustc-env=FANCY_PROMPT_BUILD_GIT_REV={}", git_describe())
}

fn git_describe() -> String {
    let output = std::process::Command::new("git")
        .args(&["describe", "--tags"])
        .output();
    output.and_then(|output| {
        if output.status.success() {
            Ok(
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string()
            )
        }
        else {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to run git"
                )
            )
        }
    }).unwrap_or_else(|_err| {
        // String::from(format!("{}", _err))
        String::from("???")
    })
}
