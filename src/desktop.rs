use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use std::io::Write;

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
    pub fn new(
        name: String,
        exec: String,
        icon: String,
        terminal: bool,
        app_type: String,
        categories: String,
    ) -> Self {
        return Self {
            name: name,
            exec: exec,
            icon: icon,
            terminal: terminal,
            app_type: app_type,
            categories: categories,
        };
    }

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

    pub fn save(&self, path: PathBuf) -> std::io::Result<()> {
        // Build the desktop entry as a single String
        let content = format!(
            "[Desktop Entry]\nName={}\nExec={}\nIcon={}\nTerminal={}\nType={}\nCategories={}\n",
            self.name,
            self.exec,
            self.icon,
            if self.terminal { "true" } else { "false" },
            self.app_type,
            self.categories
        );

        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
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
