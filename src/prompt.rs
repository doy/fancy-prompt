use chrono;
use regex;
use std;
use users;

use std::fmt::Write;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

use colors;
use power;
use vcs;

pub struct Prompt {
    colors: colors::Colors,
    data: PromptData,
}

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
    pub vcs_info: Option<Box<vcs::VcsInfo>>,
}

impl Prompt {
    pub fn new(data: PromptData) -> Prompt {
        Prompt {
            colors: colors::Colors::new(data.shell.clone()),
            data: data,
        }
    }

    pub fn display(&self) {
        let user = self.data.user
            .clone()
            .unwrap_or_else(|| String::from("???"));
        let host = self.data.hostname
            .clone()
            .unwrap_or_else(|| String::from("???"));

        let max_vcs_len = 20; // "g*+?:mybr...nch:+1-1"
        let vcs = self.format_vcs();
        let vcs = vcs.map(|vcs| {
            compress_vcs(&vcs, max_vcs_len)
        });

        let battery_len = 10;
        let cols = self.data.terminal_cols.unwrap_or(80);

        // " (~/a/...cde|g*+?:mybr:+1-1) -- {--<=======} doy@lance [19:40:50] "
        let max_path_len = cols
            - 1                            // " "
            - vcs
                .as_ref()
                .map(|vcs| vcs.len() + 1)  // "|g*+?:mybr:+1-1"
                .unwrap_or(0) - 2          // "()"
            - 1                            // " "
            - 1                            // "-"
            - 1                            // " "
            - battery_len - 2              // "{<=========}"
            - 1                            // " "
            - user.len() - 1 - host.len()  // "doy@lance"
            - 1                            // " "
            - 10                           // "[19:40:50]"
            - 1;                           // " "

        if max_path_len < 10 {             // "~/a/...cde"
            panic!(
                "terminal too small (need at least {} cols)",
                cols + 10 - max_path_len
            );
        }

        let path = compress_path(
            &self.data.pwd,
            &self.data.home,
            max_path_len
        );

        self.colors.pad(1);
        self.display_path(
            &path,
            &path_color(&self.data.pwd),
            &vcs,
            &self.vcs_color(),
        );

        self.colors.pad(1);
        self.display_border(max_path_len - path.len() + 1);
        self.colors.pad(1);

        self.display_battery(battery_len);
        self.colors.pad(1);

        self.display_identity(&user, &host);
        self.colors.pad(1);

        self.display_time();
        self.colors.pad(1);

        self.colors.newline();

        self.display_error_code();
        self.colors.pad(1);

        self.display_prompt();
        self.colors.pad(1);
    }

    fn display_path(
        &self,
        path: &str,
        path_color: &str,
        vcs: &Option<String>,
        vcs_color: &str
    ) {
        self.colors.print_host(&self.data.hostname, "(");
        self.colors.print(path_color, path);
        if let Some(ref vcs) = *vcs {
            self.colors.print_host(&self.data.hostname, "|");
            self.colors.print(vcs_color, vcs);
        }
        self.colors.print_host(&self.data.hostname, ")");
    }

    fn display_border(&self, len: usize) {
        self.colors.print("default", &"-".repeat(len));
    }

    fn display_battery(&self, len: usize) {
        self.colors.print_host(&self.data.hostname, "{");
        if let Some(battery_usage) = self.data.power_info.battery_usage() {
            let charging = self.data.power_info.charging();
            let color = battery_discharge_color(battery_usage, charging);
            let filled = (battery_usage * (len as f64)).ceil() as usize;
            let unfilled = len - filled;
            if unfilled > 0 {
                self.colors.print(color, &"-".repeat(unfilled));
            }
            if charging {
                self.colors.print("battery_charging", "<");
            }
            else {
                self.colors.print(color, ">");
            }
            if filled > 1 {
                self.colors.print("battery_charging", &"=".repeat(filled - 1));
            }
        }
        else {
            self.colors.print("error", &"?".repeat(len));
        }
        self.colors.print_host(&self.data.hostname, "}");
    }

    fn display_identity(&self, user: &str, host: &str) {
        self.colors.print_user(&self.data.user, user);
        self.colors.print("default", "@");
        self.colors.print_host(&self.data.hostname, host);
    }

    fn display_time(&self) {
        self.colors.print_host(&self.data.hostname, "[");
        self.colors.print(
            "default",
            &format!("{}", self.data.time.format("%H:%M:%S"))
        );
        self.colors.print_host(&self.data.hostname, "]");
    }

    fn display_error_code(&self) {
        let error_code_color = if self.data.error_code == 0 {
            "default"
        }
        else {
            "error"
        };
        self.colors.print(
            error_code_color,
            &format!("{:03}", self.data.error_code)
        );
    }

    fn display_prompt(&self) {
        let prompt = if self.data.is_root { "#" } else { "$" };
        self.colors.print_user(&self.data.user, prompt);
    }

    fn format_vcs(&self) -> Option<String> {
        self.data.vcs_info.as_ref().map(|vcs_info| {
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
            }

            let branch = vcs_info.branch().map(|branch| {
                if branch == "master" {
                    String::new()
                }
                else {
                    branch
                }
            }).unwrap_or_else(|| String::from("???"));
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
                vcs::ActiveOperation::None => {},
                op => {
                    write!(vcs, "({})", active_operation_id(op)).unwrap();
                }
            }

            vcs
        })
    }

    fn vcs_color(&self) -> String {
        self.data.vcs_info.as_ref().map(|vcs_info| {
            if vcs_info.is_error() {
                String::from("vcs_error")
            }
            else if vcs_info.is_dirty() {
                String::from("vcs_dirty")
            }
            else {
                String::from("default")
            }
        }).unwrap_or_else(|| String::from("vcs_error"))
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

fn path_color<T>(path: &Option<T>) -> String
    where T: AsRef<std::path::Path>
{
    path.as_ref().and_then(|path| {
        std::fs::metadata(path)
            .map(|stat| {
                // XXX there really has to be a better option here
                let euid = users::get_effective_uid();
                let egid = users::get_effective_gid();
                let file_uid = stat.st_uid();
                let file_gid = stat.st_gid();
                let file_mode = stat.permissions().mode();

                if euid == 0 {
                    String::from("default")
                }
                else if (file_uid == euid) && (file_mode & 0o200 != 0) {
                    String::from("default")
                }
                else if (file_gid == egid) && (file_mode & 0o020 != 0) {
                    String::from("default")
                }
                else if file_mode & 0o002 != 0 {
                    String::from("default")
                }
                else {
                    String::from("path_not_writable")
                }
            }).ok()
    }).unwrap_or_else(|| String::from("path_not_exist"))
}

fn compress_path<T, U>(
    path: &Option<T>,
    home: &Option<U>,
    len: usize
) -> String
    where T: AsRef<std::path::Path>,
          U: AsRef<std::path::Path>
{
    if let Some(ref path) = *path {
        let mut path_str = path.as_ref().to_string_lossy().into_owned();

        if let Some(ref home) = *home {
            let home_str = home.as_ref().to_string_lossy().into_owned();
            let home_re = regex::Regex::new(
                &(String::from(r"^") + &regex::escape(&home_str))
            ).unwrap();

            path_str = home_re.replace(&path_str, "~").into_owned();
        }

        let path_compress_re = regex::Regex::new(
            r"/([^/])[^/]+/"
        ).unwrap();

        while path_str.len() > len {
            let prev_len = path_str.len();
            path_str = path_compress_re.replace(&path_str, "/$1/").into_owned();
            if prev_len == path_str.len() {
                break;
            }
        }

        if path_str.len() > len {
            path_str = String::from(&path_str[..len - 6])
                + "..."
                + &path_str[len - 3..len]
        }

        path_str
    }
    else {
        String::from("???")
    }
}

fn compress_vcs(vcs: &str, len: usize) -> String {
    if vcs.len() > len {
        let vcs_parts_re = regex::Regex::new(
            r"^([^:]+):.*:([^:])$"
        ).unwrap();
        vcs_parts_re
            .captures(vcs)
            .map(|cap| {
                let prefix_len = cap.get(1)
                    .map(|mat| mat.end() - mat.start())
                    .unwrap_or(0);
                let suffix_len = cap.get(2)
                    .map(|mat| mat.end() - mat.start())
                    .unwrap_or(0);
                let branch_len = len - prefix_len - suffix_len - 2;
                let branch_re = regex::Regex::new(
                    &format!(
                        r"(:[^:]{{{}}})[^:]*([^:]{{3}}:)",
                        (branch_len - 6).to_string()
                    )
                ).unwrap();
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
