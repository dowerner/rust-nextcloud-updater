mod desktop;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::desktop::DesktopFile;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

const NEXTCLOUD_API_URL: &str = "https://api.github.com/repos/nextcloud/desktop/releases";
const NEXTCLOUD_DOWNLOAD_URL_TEMPLATE: &str = "https://github.com/nextcloud-releases/desktop/releases/download/v{version}/Nextcloud-{version}-x86_64.AppImage";
const VERSION_LIMIT: usize = 5;

const NEXTCLOUD_VERSIN_DELIM: &str = "Nextcloud version";

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
    println!("  update [version]    Updates to the latest Nextcloud desktop client or the version specified");
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

async fn fetch_latest_version() -> Option<String> {
    let versions = fetch_versions(false).await;
    if versions.is_err() {
        return None;
    }
    let versions = versions.unwrap();
    match versions.into_iter().next() {
        Some(version) => Some(version),
        None => None,
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

fn nextcloud_app_dir() -> Option<PathBuf> {
    let home_dir = std::env::home_dir();
    if home_dir.is_none() {
        return None;
    }

    let home = home_dir.unwrap();
    let app_dir = home.as_path().join(".local").join("bin").join("nextcloud");

    Some(app_dir)
}

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

    let desktop_file = desktop::DesktopFile::from_file(desktop_file);
    if desktop_file.is_none() {
        println!("Unable to parse .desktop file");
        return;
    }

    let deskop_file = desktop_file.unwrap();
    if !Path::new(&deskop_file.exec).is_file() {
        println!(
            "Unable to locate nextcloud client executable: {}",
            &deskop_file.exec
        );
        return;
    }

    let output = Command::new(&deskop_file.exec).arg("--version").output();

    if output.is_err() {
        println!(
            "Failed to extract version information from nextcloud executable: {}",
            output.err().unwrap()
        );
        return;
    }

    let output = output.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.split("\n");

    for line in lines {
        if line.starts_with(NEXTCLOUD_VERSIN_DELIM) {
            println!("{line}");
            break;
        }
    }
}

async fn version_from_arg(arg: Option<String>) -> Option<String> {
    match arg {
        Some(version) => Some(version),
        None => {
            let latest_version = fetch_latest_version().await;
            if latest_version.is_none() {
                return None;
            }
            let version = latest_version.unwrap();
            Some(version)
        }
    }
}

async fn install() {
    let nextcloud_desktop_file = nextcloud_desktop_file();
    if nextcloud_desktop_file.is_some() {
        println!(
            "Nextcloud seems to be installed already. Use 'update' to update the installed client."
        );
        return;
    }
    let nextcloud_desktop_file = nextcloud_desktop_file.unwrap();

    assert_additional_arg_limit(1);
    let version = version_from_arg(get_additonal_arg(1)).await;
    if version.is_none() {
        println!("Unable to determine latest Nextlcoud client version.");
        return;
    }
    let mut version = version.unwrap();
    if version.starts_with("v") {
        version.remove(0);
    }

    let app_dir = nextcloud_app_dir();
    if app_dir.is_none() {
        println!("Error while determining application location");
        return;
    }
    let app_dir = app_dir.unwrap();

    if app_dir.is_dir() {
        println!("Directory '{}' already exists.", app_dir.display());
        return;
    }

    let create_dir_res = std::fs::create_dir_all(&app_dir);
    if create_dir_res.is_err() {
        println!(
            "Error while trying to create the application directory '{}': {}",
            app_dir.display(),
            create_dir_res.err().unwrap()
        );
        return;
    }

    let exec_path = download_nextcloud_client(version, app_dir).await;
    if exec_path.is_none() {
        return;
    }
    let exec_path = exec_path.unwrap();

    let desktop_file = DesktopFile::new(
        "Nextcloud".to_string(),
        exec_path.to_str().unwrap().to_string(),
        "@APPLICATION_EXECUTABLE@".to_string(),
        false,
        "Application".to_string(),
        "Utility".to_string(),
    );
    let file_create = desktop_file.save(nextcloud_desktop_file);
    if file_create.is_err() {
        println!(
            "Error while creating .desktop file: {}",
            file_create.err().unwrap()
        );
        return;
    }
}

async fn update() {
    let nextcloud_desktop_file = nextcloud_desktop_file();
    if nextcloud_desktop_file.is_none() {
        println!("Nextcloud client was not found. Use 'install' to install it on the system.");
        return;
    }
    let nextcloud_desktop_file = nextcloud_desktop_file.unwrap();

    assert_additional_arg_limit(1);
    let version = version_from_arg(get_additonal_arg(1)).await;
    if version.is_none() {
        println!("Unable to determine latest Nextlcoud client version.");
        return;
    }
    let mut version = version.unwrap();
    if version.starts_with("v") {
        version.remove(0);
    }

    let desktop_file = DesktopFile::from_file(nextcloud_desktop_file.clone());
    if desktop_file.is_none() {
        println!("Unable to read '{}'.", nextcloud_desktop_file.display());
        return;
    }
    let desktop_file = desktop_file.unwrap();

    let old_executable = Path::new(&desktop_file.exec);
    if !old_executable.is_file() {
        println!(
            "Nextcloud installation seems corrupted, unable to find '{}'.",
            &desktop_file.exec
        );
        return;
    }

    let installation_dir: PathBuf = old_executable.parent().unwrap().into();
    let exec_path = download_nextcloud_client(version.clone(), installation_dir).await;
    if exec_path.is_none() {
        return;
    }
    let exec_path = exec_path.unwrap();
    let new_file = DesktopFile::new(
        desktop_file.name,
        exec_path.to_str().unwrap().to_string(),
        desktop_file.icon,
        desktop_file.terminal,
        desktop_file.app_type,
        desktop_file.categories,
    );

    let file_update = new_file.save(nextcloud_desktop_file);
    if file_update.is_err() {
        println!(
            "Error while updating .desktop file: {}",
            file_update.err().unwrap()
        );
        return;
    }

    println!(
        "Sucessfully updated Nextcloud client to version {}",
        version
    );
}

async fn download_nextcloud_client(version: String, target_folder: PathBuf) -> Option<PathBuf> {
    let download_url = NEXTCLOUD_DOWNLOAD_URL_TEMPLATE.replace("{version}", &version);
    println!("Downloading {download_url}...");

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", format!("{}/{}", NAME, VERSION))
        .send()
        .await;

    if response.is_err() {
        println!(
            "Failed to download Nextcloud client: {}",
            response.err().unwrap()
        );
        return None;
    }
    let response = response.unwrap();

    if !response.status().is_success() {
        println!(
            "Failed to download Nextcloud client: HTTP {}",
            response.status()
        );
        return None;
    }

    let file_name = format!("Nextcloud-{}-x86_64.AppImage", version);
    let file_path = target_folder.join(file_name);
    let file_name = file_path.to_str().unwrap();
    let mut file = match std::fs::File::create(file_name) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to create file {}: {}", file_name, e);
            return None;
        }
    };

    let content = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("Failed to read downloaded content: {}", e);
            return None;
        }
    };
    if let Err(e) = file.write_all(&content) {
        println!("Failed to write to file {}: {}", file_name, e);
        return None;
    }

    // Set the file as executable
    #[cfg(unix)]
    {
        if let Err(e) = std::fs::set_permissions(&file_name, std::fs::Permissions::from_mode(0o755))
        {
            println!(
                "Failed to set executable permissions on {}: {}",
                file_name, e
            );
            return None;
        }
    }

    println!("Downloaded and saved as {}", file_name);
    return Some(file_path);
}

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
        "install" => install().await,
        "update" => update().await,
        _ => {
            println!("unknown command: {}", command);
            help();
        }
    }
}
