# Rust Nextcloud Updater
I wrote this small program for a very simple use case: quickly install a new AppImage version of the Nextcloud desktop client for Linux. I did it in Rust to get more familiar with the language.

## Installation
To install this program you have to build it first.
- Clone the repository:
  ```bash
  git clone https://github.com/dowerner/rust-nextcloud-updater.git
  ```
- Make sure you have Rust [installed and setup](https://www.rust-lang.org)
- Then open the repository and run
  ```bash
  cargo build --release
  ```
- Ideally you move the executable `target/release/rust-nextcloud-updater` to `~/.local/bin/nextcloud-updater`.

## Usage
To use the tool make sure you have it installed first. On the terminal, type:
```bash
rust-nextcloud-updater help
```
or if you renamed it like in the previous section:
```bash
nextcloud-updater help
```
This will show you the help:
```bash
rust-nextcloud-updater v0.1.0
=============================
Usage: rust-nextcloud-updater <command>
Commands:
  help                Print this help message
  version             Print version information
  list [all]          List available Nextcloud desktop client versions
  status              Displays the status of the currently installed Nextcloud desktop client
  install [version]   Installs the Nextcloud desktop client if not already installed
  update [version]    Updates to the latest Nextcloud desktop client or the version specified
```

## Affiliation
This project does not have or claim any affiliation to the Nextcloud project or any partner projects thereof. Visit the [repository of the Nextcloud desktop client](https://github.com/nextcloud/desktop) to find the official instructions on how to use the client on Linux.