use std::{env, process::ExitCode};

use rbx_install::RobloxStudio;

const USAGE: &str = "\
Usage: rbx_install [target]

Locates the local Roblox Studio installation and prints its paths.
Respects ROBLOX_STUDIO_PATH as an override.

Targets:
  application         Path to the Roblox Studio executable
  content             Path to the content directory
  built-in-plugins    Path to the built-in plugins directory
  plugins             Path to the user plugins directory

With no target, all paths are printed.";

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("-h" | "--help" | "help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(target @ ("application" | "content" | "built-in-plugins" | "plugins")) => run(target),
        None => run(""),
        Some(other) => {
            eprintln!("error: unknown target `{other}`\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run(target: &str) -> ExitCode {
    match RobloxStudio::locate() {
        Ok(studio) => {
            println!(
                "{}",
                match target {
                    "application" => studio.application_path().display().to_string(),
                    "content" => studio.content_path().display().to_string(),
                    "built-in-plugins" => studio.built_in_plugins_path().display().to_string(),
                    "plugins" => studio.plugins_path().display().to_string(),
                    _ => format!(
                        "application:        {}\ncontent:            {}\nbuilt-in-plugins:   {}\nplugins:            {}",
                        studio.application_path().display(),
                        studio.content_path().display(),
                        studio.built_in_plugins_path().display(),
                        studio.plugins_path().display(),
                    ),
                }
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
