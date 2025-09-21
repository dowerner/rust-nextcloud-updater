use std::{
    fs::File,
    io::{self, BufRead, Read},
    path::{Path, PathBuf},
};

use tokio::fs::read;

#[derive(Debug)]
pub struct DesktopFile {
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub terminal: bool,
    pub app_type: String,
    pub categories: String,
}

impl DesktopFile {
    pub fn from_file(path: PathBuf) -> Option<Self> {
        let mut name = "".to_string();
        let mut exec = "".to_string();
        let mut icon = "".to_string();
        let mut terminal = false;
        let mut app_type = "".to_string();
        let mut categories = "".to_string();
        if let Ok(lines) = read_lines(path) {
            for line in lines.map_while(Result::ok) {
                let lcase_line = line.clone().to_lowercase();
                if lcase_line.starts_with("name") {
                    match read_entry_value_as_str(line) {
                        Some(value) => name = value,
                        None => {}
                    }
                } else if lcase_line.starts_with("exec") {
                    match read_entry_value_as_str(line) {
                        Some(value) => exec = value,
                        None => {}
                    }
                } else if lcase_line.starts_with("icon") {
                    match read_entry_value_as_str(line) {
                        Some(value) => icon = value,
                        None => {}
                    }
                } else if lcase_line.starts_with("terminal") {
                    match read_entry_value_as_bool(line) {
                        Some(value) => terminal = value,
                        None => {}
                    }
                } else if lcase_line.starts_with("type") {
                    match read_entry_value_as_str(line) {
                        Some(value) => app_type = value,
                        None => {}
                    }
                } else if lcase_line.starts_with("categories") {
                    match read_entry_value_as_str(line) {
                        Some(value) => categories = value,
                        None => {}
                    }
                }
            }

            return Some(Self {
                name: name,
                exec: exec.to_string(),
                icon: icon.to_string(),
                terminal: terminal,
                app_type: app_type.to_string(),
                categories: categories.to_string(),
            });
        } else {
            None
        }
    }

    pub fn to_string(&self) -> &str {
        self.name.as_str()
    }
}

fn read_entry_value_as_str(line: String) -> Option<String> {
    if !line.contains("=") {
        return None;
    }
    match line.split("=").last() {
        Some(value) => Some(value.to_string()),
        None => None,
    }
}

fn read_entry_value_as_bool(line: String) -> Option<bool> {
    match read_entry_value_as_str(line) {
        Some(value) => Some(value.to_lowercase().trim() == "true"),
        None => None,
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
