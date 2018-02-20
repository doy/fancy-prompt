use chrono;
use regex;
use std;

use colors;
use power;

pub struct Prompt {
    colors: colors::Colors,
    data: PromptData,
}

#[derive(Debug)]
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
}

impl Prompt {
    pub fn new(data: PromptData) -> Prompt {
        Prompt {
            colors: colors::Colors::new(data.shell.clone()),
            data: data,
        }
    }

    pub fn display(&self) {
        let user = self.data.user.clone().unwrap_or(String::from("???"));
        let host = self.data.hostname.clone().unwrap_or(String::from("???"));

        let battery_len = 10;
        let cols = self.data.terminal_cols.unwrap_or(80);

        // " (~/a/...cde) ---- {--<=======} doy@lance [19:40:50] "
        let max_path_len = cols
            - 1                            // " "
            - 2                            // "()"
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
        self.display_path(&path);

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

    fn display_path(&self, path: &str) {
        self.colors.print_host(&self.data.hostname, "(");
        self.colors.print("default", path);
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
        self.colors.print_user(&self.data.user, &user);
        self.colors.print("default", "@");
        self.colors.print_host(&self.data.hostname, &host);
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

fn compress_path<T, U>(
    path: &Option<T>,
    home: &Option<U>,
    len: usize
) -> String
    where T: AsRef<std::path::Path>,
          U: AsRef<std::path::Path>
{
    if let &Some(ref path) = path {
        let mut path_str = path.as_ref().to_string_lossy().into_owned();

        if let &Some(ref home) = home {
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
