use std;
use term;

#[derive(Debug,Clone)]
pub enum ShellType {
    Unknown,
    Bash,
    Zsh,
}

type ColorMap = std::collections::HashMap<String, term::color::Color>;

pub struct Colors {
    color_map: ColorMap,
    unknown_color: term::color::Color,
    shell_type: ShellType,
}

impl ShellType {
    pub fn from_str(shell: &str) -> Self {
        match shell {
            "bash" => ShellType::Bash,
            "zsh" => ShellType::Zsh,
            _ => panic!("unknown shell {}", shell)
        }
    }
}

impl Colors {
    pub fn new(shell_type: ShellType) -> Colors {
        let mut color_map = std::collections::HashMap::new();

        color_map.insert(String::from("path_not_writable"), term::color::YELLOW);
        color_map.insert(String::from("path_not_exist"), term::color::RED);

        color_map.insert(String::from("battery_warn"), term::color::YELLOW);
        color_map.insert(String::from("battery_crit"), term::color::RED);
        color_map.insert(String::from("battery_emerg"), term::color::BRIGHT_RED);
        color_map.insert(String::from("battery_full"), term::color::GREEN);
        color_map.insert(String::from("battery_charging"), term::color::GREEN);

        color_map.insert(String::from("default"), term::color::BRIGHT_BLACK);
        color_map.insert(String::from("error"), term::color::RED);

        let unknown_color = term::color::YELLOW;

        Self::read_colors_from_env(&mut color_map);

        Colors {
            color_map: color_map,
            unknown_color: unknown_color,
            shell_type: shell_type,
        }
    }

    fn read_colors_from_env(color_map: &mut ColorMap) {
        if let Ok(val) = std::env::var("FANCY_PROMPT_COLORS") {
            for mapping in val.split(",") {
                let parts: Vec<_> = mapping.split("=").collect();
                let (name, color) = (parts[0], parts[1]);
                color_map.insert(
                    String::from(name),
                    Self::color_from_string(color)
                );
            }
        }
    }

    fn color_from_string(color_name: &str) -> term::color::Color {
        match color_name {
            "black" => term::color::BLACK,
            "blue" => term::color::BLUE,
            "bright_black" => term::color::BRIGHT_BLACK,
            "bright_blue" => term::color::BRIGHT_BLUE,
            "bright_cyan" => term::color::BRIGHT_CYAN,
            "bright_green" => term::color::BRIGHT_GREEN,
            "bright_magenta" => term::color::BRIGHT_MAGENTA,
            "bright_red" => term::color::BRIGHT_RED,
            "bright_white" => term::color::BRIGHT_WHITE,
            "bright_yellow" => term::color::BRIGHT_YELLOW,
            "cyan" => term::color::CYAN,
            "green" => term::color::GREEN,
            "magenta" => term::color::MAGENTA,
            "red" => term::color::RED,
            "white" => term::color::WHITE,
            "yellow" => term::color::YELLOW,
            _ => panic!("unknown color {}", color_name),
        }
    }

    pub fn print(&self, color: &str, text: &str) {
        let color = self.color_map.get(color);
        self.print_with_color(color, text);
    }

    pub fn pad(&self, len: usize) {
        print!("{}", " ".repeat(len));
    }

    pub fn newline(&self) {
        self.print_wrapped(|| {
            print!("{}", "\n");
        });
    }

    pub fn print_host(&self, host: &Option<String>, text: &str) {
        let color = host
            .clone()
            .and_then(|hostname| {
                self.color_map.get(&format!("host_{}", hostname))
            });
        self.print_with_color(color, text);
    }

    pub fn print_user(&self, user: &Option<String>, text: &str) {
        let color = user
            .clone()
            .and_then(|username| {
                self.color_map.get(&format!("user_{}", username))
            });
        self.print_with_color(color, text);
    }

    fn print_with_color(&self, color: Option<&term::color::Color>, text: &str) {
        let mut t = term::stdout().unwrap();
        self.print_color(&mut *t, color);
        write!(t, "{}", text).unwrap();
        t.reset().unwrap();
    }

    fn print_color(&self, t: &mut term::StdoutTerminal, color: Option<&term::color::Color>) {
        self.print_wrapped(|| {
            let real_color = *color.unwrap_or(&self.unknown_color);
            t.fg(real_color).unwrap();
            match real_color {
                term::color::BRIGHT_BLACK
            | term::color::BRIGHT_BLUE
            | term::color::BRIGHT_CYAN
            | term::color::BRIGHT_GREEN
            | term::color::BRIGHT_MAGENTA
            | term::color::BRIGHT_RED
            | term::color::BRIGHT_WHITE
            | term::color::BRIGHT_YELLOW => {
                    t.attr(term::Attr::Bold).unwrap()
                },
                _ => {},
            }
        })
    }

    fn print_wrapped<T>(&self, printer: T)
        where T: FnOnce()
    {
        match self.shell_type {
            ShellType::Bash => { print!("{}", "\\["); },
            ShellType::Zsh => { print!("{}", "%{"); },
            _ => {},
        }

        printer();

        match self.shell_type {
            ShellType::Bash => { print!("{}", "\\]"); },
            ShellType::Zsh => { print!("{}", "%}"); },
            _ => {},
        }
    }
}
