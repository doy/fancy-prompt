use chrono;
use regex;
use std;

use colors;
use power;

pub struct Prompt {
    components_line1: Vec<Box<PromptComponent>>,
    components_line2: Vec<Box<PromptComponent>>,
    data: PromptData,
    colors: colors::Colors,
}

trait PromptComponent {
    fn display(&self, colors: &colors::Colors, data: &PromptData);
    fn length(&self, data: &PromptData) -> usize;
}

struct PromptPath {
    braces: String,
}

struct PromptBorder {
    border: String,
}

struct PromptBattery {
    length: usize,

    braces: String,

    full: String,
    empty: String,
    charging: String,
    discharging: String,
    unknown: String,
}

struct PromptIdentity {}

struct PromptTime {
    braces: String,
}

struct PromptCommandError {}

struct PromptPrompt {
    user: String,
    root: String,
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
        let shell = data.shell.clone();
        let components_line1: Vec<Box<PromptComponent>> = vec![
            Box::new(PromptPath { braces: "()".to_string() }),
            Box::new(PromptBorder { border: "-".to_string() }),
            Box::new(PromptBattery {
                length: 10,
                braces: "{}".to_string(),
                full: "=".to_string(),
                empty: "-".to_string(),
                charging: "<".to_string(),
                discharging: ">".to_string(),
                unknown: "?".to_string(),
            }),
            Box::new(PromptIdentity {}),
            Box::new(PromptTime { braces: "[]".to_string() }),
        ];
        let components_line2: Vec<Box<PromptComponent>> = vec![
            Box::new(PromptCommandError {}),
            Box::new(PromptPrompt {
                user: "$".to_string(),
                root: "#".to_string(),
            }),
        ];

        Prompt {
            components_line1: components_line1,
            components_line2: components_line2,
            data: data,
            colors: colors::Colors::new(shell),
        }
    }

    pub fn display(&self) {
        self.colors.pad(1);
        for component in self.components_line1.iter() {
            component.display(&self.colors, &self.data);
            self.colors.pad(1);
        }

        self.colors.newline();

        for component in self.components_line2.iter() {
            component.display(&self.colors, &self.data);
            self.colors.pad(1);
        }
    }
}

impl PromptComponent for PromptPath {
    fn display(&self, colors: &colors::Colors, data: &PromptData) {
        colors.print_host(
            &data.hostname,
            &self.braces[0..1]
        );
        if let Some(ref pwd) = data.pwd {
            colors.print(
                "default",
                &compress_path(pwd, &data.home, 20)
            );
        }
        else {
            colors.print("error", "???");
        }
        colors.print_host(
            &data.hostname,
            &self.braces[1..2]
        );
    }

    fn length(&self, data: &PromptData) -> usize {
        0
    }
}

impl PromptComponent for PromptBorder {
    fn display(&self, colors: &colors::Colors, data: &PromptData) {
        colors.print("default", &self.border.repeat(20))
    }

    fn length(&self, data: &PromptData) -> usize {
        0
    }
}

impl PromptComponent for PromptBattery {
    fn display(&self, colors: &colors::Colors, data: &PromptData) {
        colors.print_host(
            &data.hostname,
            &self.braces[0..1]
        );

        if let Some(usage) = data.power_info.battery_usage() {
            let color = if usage >= 0.8 {
                "battery_full"
            }
            else if data.power_info.charging() {
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
            };
            let filled = (self.length as f64 * usage).ceil() as usize;
            let unfilled = self.length - filled;

            if unfilled > 0 {
                colors.print(
                    color,
                    &self.empty.repeat(unfilled)
                );
            }

            if data.power_info.charging() {
                colors.print(
                    color,
                    &self.charging
                )
            }
            else {
                colors.print(
                    color,
                    &self.discharging
                )
            }

            if filled > 1 {
                colors.print(
                    color,
                    &self.full.repeat(filled - 1)
                )
            }
        }
        else {
            colors.print(
                "battery_emerg",
                &self.unknown.repeat(self.length)
            )
        }

        colors.print_host(
            &data.hostname,
            &self.braces[1..2]
        );
    }

    fn length(&self, data: &PromptData) -> usize {
        0
    }
}

impl PromptComponent for PromptIdentity {
    fn display(&self, colors: &colors::Colors, data: &PromptData) {
        colors.print_user(
            &data.user,
            &data.user.clone().unwrap_or("???".to_string())
        );
        colors.print("default", "@");
        colors.print_host(
            &data.hostname,
            &data.hostname.clone().unwrap_or("???".to_string())
        );
    }

    fn length(&self, data: &PromptData) -> usize {
        0
    }
}

impl PromptComponent for PromptTime {
    fn display(&self, colors: &colors::Colors, data: &PromptData) {
        colors.print_host(
            &data.hostname,
            &self.braces[0..1]
        );

        colors.print(
            "default",
            &format!("{}", data.time.format("%H:%M:%S"))
        );

        colors.print_host(
            &data.hostname,
            &self.braces[1..2]
        );
    }

    fn length(&self, data: &PromptData) -> usize {
        0
    }
}

impl PromptComponent for PromptCommandError {
    fn display(&self, colors: &colors::Colors, data: &PromptData) {
        let color = if data.error_code == 0 {
            "default"
        }
        else {
            "error"
        };
        colors.print(color, &format!("{:03}", data.error_code));
    }

    fn length(&self, data: &PromptData) -> usize {
        0
    }
}

impl PromptComponent for PromptPrompt {
    fn display(&self, colors: &colors::Colors, data: &PromptData) {
        let prompt = if data.is_root {
            &self.root
        }
        else {
            &self.user
        };
        colors.print_user(&data.user, prompt);
    }

    fn length(&self, data: &PromptData) -> usize {
        0
    }
}

fn compress_path<T: AsRef<std::path::Path>, U: AsRef<std::path::Path>>(path: T, home: &Option<U>, max_len: u16) -> String {
    let path_str = path.as_ref().to_string_lossy().into_owned();
    if let &Some(ref home) = home {
        let home_str = home.as_ref().to_string_lossy().into_owned();
        let home_re = regex::Regex::new(&(r"^".to_string() + &regex::escape(&home_str))).unwrap();
        home_re.replace(&path_str, "~").into_owned()
    }
    else {
        path_str
    }
}
