mod desktop;
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

const NEXTCLOUD_API_URL: &str = "https://api.github.com/repos/nextcloud/desktop/releases";
const VERSION_LIMIT: usize = 5;

fn assert_additional_arg_limit(count: usize) {
    if count + 2 < std::env::args().count() {
        println!("too many arguments");
        std::process::exit(1);
    }
}

fn assert_no_additional_args() {
    assert_additional_arg_limit(0);
}

fn invalid_argument(arg: String) {
    println!("invalid argument: {}", arg);
    std::process::exit(1);
}

fn get_additonal_arg(pos: usize) -> Option<String> {
    if pos + 1 >= std::env::args().count() {
        return None;
    }
    Some(std::env::args().nth(pos + 1).unwrap())
}

fn header() {
    let line_lenght = NAME.len() + VERSION.len() + 2;
    println!("");
    println!("{} v{}", NAME, VERSION);
    println!("{}", "=".repeat(line_lenght));
}

fn help() {
    assert_no_additional_args();
    header();
    println!("Usage: {} <command>", NAME);
    println!("Commands:");
    println!("  help                Print this help message");
    println!("  version             Print version information");
    println!("  list [all]          List available Nextcloud desktop client versions");
    println!(
        "  status              Displays the status of the currently installed Nextcloud desktop client"
    );
    println!(
        "  install [version]   Installs the Nextcloud desktop client if not already installed"
    );
    println!("  update [allow-rc]   Updates to the latest Nextcloud desktop client");
}

fn version() {
    assert_no_additional_args();
    println!("{}", VERSION.to_string());
}

async fn list() {
    assert_additional_arg_limit(1);
    println!("Fetching available Nextcloud versions...");

    let arg = get_additonal_arg(1);
    let all = arg.is_some();
    if arg.is_some() {
        let arg_name = arg.unwrap();
        if arg_name != "all" {
            invalid_argument(arg_name.to_string());
        }
    }

    match fetch_versions(all).await {
        Ok(versions) => {
            println!("Available Nextcloud versions:");
            for version in versions {
                println!("{}", version);
            }
        }
        Err(e) => {
            println!("Error fetching versions: {}", e);
        }
    }
}

async fn fetch_versions(all: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = NEXTCLOUD_API_URL;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", format!("{}/{}", NAME, VERSION))
        .send()
        .await?;

    if response.status().is_success() {
        let releases: Vec<serde_json::Value> = response.json().await?;
        let all_versions: Vec<String> = releases
            .iter()
            .map(|release| {
                release["tag_name"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            })
            .collect();
        Ok(if all {
            all_versions
        } else {
            all_versions.into_iter().take(VERSION_LIMIT).collect()
        })
    } else {
        Err(format!("HTTP error: {}", response.status()).into())
    }
}

fn applications_dir() -> Option<PathBuf> {
    let home_dir = std::env::home_dir();
    if home_dir.is_none() {
        return None;
    }

    let home = home_dir.unwrap();
    let applications_dir = home
        .as_path()
        .join(".local")
        .join("share")
        .join("applications");

    if !applications_dir.is_dir() {
        return None;
    }

    Some(applications_dir)
}

fn nextcloud_desktop_file() -> Option<PathBuf> {
    match applications_dir() {
        Some(app_dir) => {
            let file = app_dir.join("nextcloud.desktop");
            if file.is_file() { Some(file) } else { None }
        }
        None => None,
    }
}

// fn nextcloud_app_folder() -> Option<PathBuf> {

// }

fn status() {
    let nextcloud_desktop_file = nextcloud_desktop_file();
    if nextcloud_desktop_file.is_none() {
        println!("Did not find existing nextcloud.desktop file.");
        return;
    }

    let desktop_file = nextcloud_desktop_file.unwrap();
    println!(
        "Found existing app configuration: {}",
        desktop_file.display()
    );

    let test = desktop::DesktopFile::from_file(desktop_file).unwrap();
    println!("{}", test.icon);
}

fn install() {}

fn update() {}

#[tokio::main]
async fn main() {
    if std::env::args().count() < 2 {
        println!("no command given");
        help();
        return;
    }
    let command = std::env::args().nth(1).unwrap();
    match command.as_str() {
        "help" => help(),
        "version" => version(),
        "list" => list().await,
        "status" => status(),
        "install" => install(),
        "update" => update(),
        _ => {
            println!("unknown command: {}", command);
            help();
        }
    }
}
