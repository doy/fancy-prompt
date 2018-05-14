use regex;
use std;
use term;

use std::fmt::Write;

use colors;
use data;
use sys;
use vcs;

pub struct Prompt {
    colors: colors::Colors,
    data: data::PromptData,
}

impl Prompt {
    pub fn new(data: data::PromptData) -> Prompt {
        let colors = colors::Colors::new(data.shell.clone());
        Prompt {
            colors,
            data,
        }
    }

    pub fn display<W: std::io::Write>(&self, w: W) {
        let mut t = term::TerminfoTerminal::new(w).unwrap();

        let user =
            self.data.user
                .as_ref()
                .map(String::as_ref)
                .unwrap_or_else(|| "???");
        let host =
            self.data.hostname
                .as_ref()
                .map(String::as_ref)
                .unwrap_or_else(|| "???");

        let max_vcs_len = 20; // "g*+?:mybr...nch:+1-1"
        let vcs = self.format_vcs();
        let vcs = vcs.map(|vcs| compress_vcs(&vcs, max_vcs_len));

        let battery_len = 10;
        let cols = self.data.terminal_cols.unwrap_or(80);

        // " (~/a/...cde|g*+?:mybr:+1-1) -- {--<=======} doy@lance [19:40:50] "
        let mut max_path_len = cols
            - 1                               // " "
            - vcs
                .as_ref()
                .map(|vcs| vcs.len() + 1)     // "|g*+?:mybr:+1-1"
                .unwrap_or(0) - 2             // "()"
            - 1                               // " "
            - 1                               // "-"
            - 1                               // " "
            - user.len() - 1 - host.len()     // "doy@lance"
            - 1                               // " "
            - 10                              // "[19:40:50]"
            - 1;                              // " "
        if self.data.power_info.has_batteries() {
            max_path_len -= battery_len + 2   // "{<=========}"
                + 1;                          // " "
        }

        if max_path_len < 10 {                // "~/a/...cde"
            panic!(
                "terminal too small (need at least {} cols)",
                cols + 10 - max_path_len
            );
        }

        let path =
            compress_path(&self.data.pwd, &self.data.home, max_path_len);

        self.colors.pad(&mut t, 1);
        self.display_path(
            &mut t,
            &path,
            &path_color(
                self.data.pwd.as_ref().map(std::path::PathBuf::as_ref),
            ),
            vcs.as_ref().map(String::as_ref),
            &self.vcs_color(),
        );

        self.colors.pad(&mut t, 1);
        self.display_border(&mut t, max_path_len - path.len() + 1);
        self.colors.pad(&mut t, 1);

        if self.data.power_info.has_batteries() {
            self.display_battery(&mut t, battery_len);
            self.colors.pad(&mut t, 1);
        }

        self.display_identity(&mut t, user, host);
        self.colors.pad(&mut t, 1);

        self.display_time(&mut t);
        self.colors.pad(&mut t, 1);

        self.colors.newline(&mut t);

        self.display_error_code(&mut t);
        self.colors.pad(&mut t, 1);

        self.display_prompt(&mut t);
        self.colors.pad(&mut t, 1);

        #[cfg(feature = "verbose")]
        self.colors.newline(&mut t);
    }

    fn display_path<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
        path: &str,
        path_color: &str,
        vcs: Option<&str>,
        vcs_color: &str,
    ) {
        self.print_host(t, "(");
        self.colors.print(t, path_color, path);
        if let Some(vcs) = vcs {
            self.print_host(t, "|");
            self.colors.print(t, vcs_color, vcs);
        }
        self.print_host(t, ")");
    }

    fn display_border<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
        len: usize
    ) {
        self.colors.print(t, "default", &"-".repeat(len));
    }

    fn display_battery<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
        len: usize
    ) {
        self.print_host(t, "{");
        if let Some(battery_usage) = self.data.power_info.battery_usage() {
            let charging = self.data.power_info.charging();
            let color = battery_discharge_color(battery_usage, charging);
            let filled = (battery_usage * (len as f64)).ceil() as usize;
            if len > filled {
                let unfilled = len - filled;
                self.colors.print(t, color, &"-".repeat(unfilled));
            }
            if len >= filled {
                if charging {
                    self.colors.print(t, "battery_charging", "<");
                }
                else {
                    self.colors.print(t, color, ">");
                }
            }
            if filled > 1 {
                self.colors
                    .print(t, "battery_charging", &"=".repeat(filled - 1));
            }
        }
        else {
            self.colors.print(t, "error", &"?".repeat(len));
        }
        self.print_host(t, "}");
    }

    fn display_identity<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
        user: &str,
        host: &str,
    ) {
        self.print_user(t, user);
        self.colors.print(t, "default", "@");
        self.print_host(t, host);
    }

    fn display_time<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
    ) {
        self.print_host(t, "[");
        self.colors.print(
            t,
            "default",
            &format!("{}", self.data.time.format("%H:%M:%S")),
        );
        self.print_host(t, "]");
    }

    fn display_error_code<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
    ) {
        let error_code_color = if self.data.error_code == 0 {
            "default"
        }
        else {
            "error"
        };
        self.colors.print(
            t,
            error_code_color,
            &format!("{:03}", self.data.error_code)
        );
    }

    fn display_prompt<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
    ) {
        let prompt = if self.data.is_root {
            "#"
        }
        else {
            "$"
        };
        self.print_user(t, prompt);
    }

    fn format_vcs(&self) -> Option<String> {
        format_vcs(self.data.vcs_info.as_ref().map(|v| &**v))
    }

    fn vcs_color(&self) -> String {
        vcs_color(self.data.vcs_info.as_ref().map(|v| &**v))
    }

    fn print_host<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
        text: &str
    ) {
        self.colors.print_host(t, self.hostname(), text);
    }

    fn print_user<W: std::io::Write>(
        &self,
        t: &mut term::Terminal<Output=W>,
        text: &str
    ) {
        self.colors.print_user(t, self.user(), text);
    }

    fn hostname(&self) -> Option<&str> {
        self.data.hostname.as_ref().map(String::as_ref)
    }

    fn user(&self) -> Option<&str> {
        self.data.user.as_ref().map(String::as_ref)
    }
}

fn battery_discharge_color(usage: f64, charging: bool) -> &'static str {
    if usage >= 0.8 {
        "battery_full"
    }
    else if charging {
        "default"
    }
    else if usage >= 0.4 {
        "default"
    }
    else if usage >= 0.15 {
        "battery_warn"
    }
    else if usage >= 0.05 {
        "battery_crit"
    }
    else {
        "battery_emerg"
    }
}

fn path_color(path: Option<&std::path::Path>) -> String {
    path.as_ref()
        .map(|path| {
            match sys::path_writable(path) {
                sys::PathWritability::Writable
                    => String::from("default"),
                sys::PathWritability::NotWritable
                    => String::from("path_not_writable"),
                sys::PathWritability::NotExist
                    => String::from("path_not_exist"),
            }
        })
        .unwrap_or_else(|| String::from("path_not_exist"))
}

fn format_vcs(vcs_info: Option<&vcs::VcsInfo>) -> Option<String> {
    vcs_info.as_ref().map(|vcs_info| {
        let mut vcs = String::new();

        write!(vcs, "{}", vcs_id(vcs_info.vcs())).unwrap();

        if vcs_info.has_modified_files() {
            write!(vcs, "*").unwrap();
        }
        if vcs_info.has_staged_files() {
            write!(vcs, "+").unwrap();
        }
        if vcs_info.has_new_files() {
            write!(vcs, "?").unwrap();
        }
        if !vcs_info.has_commits() {
            write!(vcs, "!").unwrap();
            return vcs;
        }

        let branch = vcs_info
            .branch()
            .map(|branch| {
                if branch == "master" {
                    String::new()
                }
                else {
                    branch
                }
            })
            .unwrap_or_else(|| String::from("???"));
        if branch != "" {
            write!(vcs, ":").unwrap();
        }
        write!(vcs, "{}", branch).unwrap();

        if let Some((local, remote)) = vcs_info.remote_branch_diff() {
            if local > 0 || remote > 0 {
                write!(vcs, ":").unwrap();
            }
            if local > 0 {
                write!(vcs, "+{}", local).unwrap();
            }
            if remote > 0 {
                write!(vcs, "-{}", remote).unwrap();
            }
        }
        else {
            write!(vcs, ":-").unwrap();
        }

        match vcs_info.active_operation() {
            vcs::ActiveOperation::None => {}
            op => {
                write!(vcs, "({})", active_operation_id(op)).unwrap();
            }
        }

        vcs
    })
}

fn vcs_color(vcs_info: Option<&vcs::VcsInfo>) -> String {
    vcs_info
        .as_ref()
        .map(|vcs_info| {
            if vcs_info.is_error() {
                String::from("vcs_error")
            }
            else if vcs_info.is_dirty() {
                String::from("vcs_dirty")
            }
            else {
                String::from("default")
            }
        })
        .unwrap_or_else(|| String::from("vcs_error"))
}

fn compress_path<T, U>(
    path: &Option<T>,
    home: &Option<U>,
    len: usize,
) -> String
where
    T: AsRef<std::path::Path>,
    U: AsRef<std::path::Path>,
{
    if let Some(ref path) = *path {
        let mut path_str = path.as_ref().to_string_lossy().into_owned();

        if let Some(ref home) = *home {
            let home_str = home.as_ref().to_string_lossy().into_owned();
            let home_re = regex::Regex::new(
                &(String::from(r"^") + &regex::escape(&home_str)),
            ).unwrap();

            path_str = home_re.replace(&path_str, "~").into_owned();
        }

        let path_compress_re = regex::Regex::new(r"/([^/])[^/]+/").unwrap();

        while path_str.len() > len {
            let prev_len = path_str.len();
            path_str =
                path_compress_re.replace(&path_str, "/$1/").into_owned();
            if prev_len == path_str.len() {
                break;
            }
        }

        if path_str.len() > len {
            path_str = String::from(&path_str[..len - 6]) + "..."
                + &path_str[path_str.len() - 3..]
        }

        path_str
    }
    else {
        String::from("???")
    }
}

fn compress_vcs(vcs: &str, len: usize) -> String {
    if vcs.len() > len {
        let vcs_parts_re =
            regex::Regex::new(r"^([^:]+):.*?(?::([^:]+))?$").unwrap();
        vcs_parts_re
            .captures(vcs)
            .map(|cap| {
                let prefix_len = cap.get(1)
                    .map(|mat| mat.end() - mat.start() + 1)
                    .unwrap_or(0);
                let suffix_len = cap.get(2)
                    .map(|mat| mat.end() - mat.start() + 1)
                    .unwrap_or(0);
                let branch_len = len - prefix_len - suffix_len;
                let branch_re = regex::Regex::new(&format!(
                    r"(:[^:]{{{}}})[^:]*([^:]{{3}}:?)",
                    (branch_len - 6).to_string()
                )).unwrap();
                branch_re.replace(vcs, "$1...$2").into_owned()
            })
            .unwrap_or_else(|| vcs.to_string())
    }
    else {
        vcs.to_string()
    }
}

fn vcs_id(vcs: vcs::VcsType) -> String {
    match vcs {
        vcs::VcsType::Git => String::from("g"),
    }
}

fn active_operation_id(op: vcs::ActiveOperation) -> String {
    match op {
        vcs::ActiveOperation::None => String::new(),
        vcs::ActiveOperation::Merge => String::from("m"),
        vcs::ActiveOperation::Revert => String::from("v"),
        vcs::ActiveOperation::CherryPick => String::from("c"),
        vcs::ActiveOperation::Bisect => String::from("b"),
        vcs::ActiveOperation::Rebase => String::from("r"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone)]
    struct TestVcs {
        vcs: vcs::VcsType,
        has_modified_files: bool,
        has_staged_files: bool,
        has_new_files: bool,
        has_commits: bool,
        active_operation: vcs::ActiveOperation,
        branch: Option<String>,
        remote_branch_diff: Option<(usize, usize)>,
    }

    impl vcs::VcsInfo for TestVcs {
        fn vcs(&self) -> vcs::VcsType {
            self.vcs
        }
        fn has_modified_files(&self) -> bool {
            self.has_modified_files
        }
        fn has_staged_files(&self) -> bool {
            self.has_staged_files
        }
        fn has_new_files(&self) -> bool {
            self.has_new_files
        }
        fn has_commits(&self) -> bool {
            self.has_commits
        }
        fn active_operation(&self) -> vcs::ActiveOperation {
            self.active_operation
        }
        fn branch(&self) -> Option<String> {
            self.branch.clone()
        }
        fn remote_branch_diff(&self) -> Option<(usize, usize)> {
            self.remote_branch_diff
        }
    }

    #[test]
    fn test_compress_path() {
        {
            let home = &Some("/home/doy");
            let path = &Some("/home/doy/coding/fancy-prompt");
            let expecteds = vec![
                "~/coding/fancy-prompt", // 25
                "~/coding/fancy-prompt",
                "~/coding/fancy-prompt",
                "~/coding/fancy-prompt",
                "~/coding/fancy-prompt",
                "~/c/fancy-prompt", // 20
                "~/c/fancy-prompt",
                "~/c/fancy-prompt",
                "~/c/fancy-prompt",
                "~/c/fancy-prompt",
                "~/c/fancy...mpt", // 15
                "~/c/fanc...mpt",
                "~/c/fan...mpt",
                "~/c/fa...mpt",
                "~/c/f...mpt",
                "~/c/...mpt",
            ];
            for (i, &expected) in expecteds.iter().enumerate() {
                assert_eq!(compress_path(path, home, 25 - i), expected);
            }
        }
    }

    #[test]
    fn test_compress_vcs() {
        {
            let vcs = "g:this-is-a-branch:-";
            let expecteds = vec![
                "g:this-is-a-branch:-", // 25
                "g:this-is-a-branch:-",
                "g:this-is-a-branch:-",
                "g:this-is-a-branch:-",
                "g:this-is-a-branch:-",
                "g:this-is-a-branch:-",
                "g:this-is-a...nch:-", // 19
                "g:this-is-...nch:-",
                "g:this-is...nch:-",
                "g:this-i...nch:-",
                "g:this-...nch:-",
                "g:this...nch:-",
                "g:thi...nch:-",
                "g:th...nch:-",
                "g:t...nch:-",
                "g:...nch:-",
            ];
            for (i, &expected) in expecteds.iter().enumerate() {
                assert_eq!(compress_vcs(vcs, 25 - i), expected);
            }
        }
        {
            let vcs = "g:this-is-a-branch";
            let expecteds = vec![
                "g:this-is-a-branch", // 23
                "g:this-is-a-branch",
                "g:this-is-a-branch",
                "g:this-is-a-branch",
                "g:this-is-a-branch",
                "g:this-is-a-branch",
                "g:this-is-a...nch", // 17
                "g:this-is-...nch",
                "g:this-is...nch",
                "g:this-i...nch",
                "g:this-...nch",
                "g:this...nch",
                "g:thi...nch",
                "g:th...nch",
                "g:t...nch",
                "g:...nch",
            ];
            for (i, &expected) in expecteds.iter().enumerate() {
                assert_eq!(compress_vcs(vcs, 23 - i), expected);
            }
        }
        {
            let vcs = "g*:this-is-a-branch:+1-14(m)";
            let expecteds = vec![
                "g*:this-is-a-branch:+1-14(m)", // 33
                "g*:this-is-a-branch:+1-14(m)",
                "g*:this-is-a-branch:+1-14(m)",
                "g*:this-is-a-branch:+1-14(m)",
                "g*:this-is-a-branch:+1-14(m)",
                "g*:this-is-a-branch:+1-14(m)",
                "g*:this-is-a...nch:+1-14(m)", // 27
                "g*:this-is-...nch:+1-14(m)",
                "g*:this-is...nch:+1-14(m)",
                "g*:this-i...nch:+1-14(m)",
                "g*:this-...nch:+1-14(m)",
                "g*:this...nch:+1-14(m)",
                "g*:thi...nch:+1-14(m)",
                "g*:th...nch:+1-14(m)",
                "g*:t...nch:+1-14(m)",
                "g*:...nch:+1-14(m)",
            ];
            for (i, &expected) in expecteds.iter().enumerate() {
                assert_eq!(compress_vcs(vcs, 33 - i), expected);
            }
        }
    }

    #[test]
    fn test_format_vcs() {
        {
            assert_eq!(format_vcs(None), None)
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: false,
                has_staged_files: false,
                has_new_files: false,
                has_commits: true,
                active_operation: vcs::ActiveOperation::None,
                branch: Some(String::from("master")),
                remote_branch_diff: Some((0, 0)),
            };

            assert_eq!(format_vcs(Some(&test_vcs)), Some(String::from("g")));
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("default"));
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: false,
                has_staged_files: false,
                has_new_files: false,
                has_commits: true,
                active_operation: vcs::ActiveOperation::None,
                branch: Some(String::from("dev")),
                remote_branch_diff: Some((0, 0)),
            };

            assert_eq!(
                format_vcs(Some(&test_vcs)),
                Some(String::from("g:dev"))
            );
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("default"));
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: false,
                has_staged_files: false,
                has_new_files: false,
                has_commits: true,
                active_operation: vcs::ActiveOperation::None,
                branch: Some(String::from("master")),
                remote_branch_diff: None,
            };

            assert_eq!(
                format_vcs(Some(&test_vcs)),
                Some(String::from("g:-"))
            );
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("vcs_dirty"));
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: false,
                has_staged_files: false,
                has_new_files: false,
                has_commits: true,
                active_operation: vcs::ActiveOperation::None,
                branch: Some(String::from("dev")),
                remote_branch_diff: None,
            };

            assert_eq!(
                format_vcs(Some(&test_vcs)),
                Some(String::from("g:dev:-"))
            );
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("vcs_dirty"));
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: true,
                has_staged_files: true,
                has_new_files: true,
                has_commits: true,
                active_operation: vcs::ActiveOperation::None,
                branch: Some(String::from("master")),
                remote_branch_diff: None,
            };

            assert_eq!(
                format_vcs(Some(&test_vcs)),
                Some(String::from("g*+?:-"))
            );
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("vcs_dirty"));
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: true,
                has_staged_files: true,
                has_new_files: true,
                has_commits: true,
                active_operation: vcs::ActiveOperation::None,
                branch: Some(String::from("dev")),
                remote_branch_diff: None,
            };

            assert_eq!(
                format_vcs(Some(&test_vcs)),
                Some(String::from("g*+?:dev:-"))
            );
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("vcs_dirty"));
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: false,
                has_staged_files: false,
                has_new_files: false,
                has_commits: false,
                active_operation: vcs::ActiveOperation::None,
                branch: None,
                remote_branch_diff: None,
            };

            assert_eq!(format_vcs(Some(&test_vcs)), Some(String::from("g!")));
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("vcs_error"));
        }
        {
            let test_vcs = TestVcs {
                vcs: vcs::VcsType::Git,
                has_modified_files: false,
                has_staged_files: false,
                has_new_files: false,
                has_commits: true,
                active_operation: vcs::ActiveOperation::None,
                branch: Some(String::from("master")),
                remote_branch_diff: Some((2, 3)),
            };

            assert_eq!(
                format_vcs(Some(&test_vcs)),
                Some(String::from("g:+2-3"))
            );
            assert_eq!(vcs_color(Some(&test_vcs)), String::from("vcs_dirty"));
        }
    }
}
