mod config;
mod util;

use clap::{builder::styling, Parser, Subcommand};
use colored::Colorize;
use std::{fs, process};
use std::{path::PathBuf, sync::LazyLock};

static PLUGIN_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    home::home_dir()
        .unwrap()
        .join("AppData/Local/Microsoft/PowerToys/PowerToys Run/Plugins")
});
static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    home::home_dir()
        .unwrap()
        .join("AppData/Local/Microsoft/PowerToys/PowerToys Run/Plugins/ptr.toml")
});

#[derive(Parser)]
#[clap(
    version,
    name = "ptr",
    about = "PowerToys Run Plugin Manager",
    styles = get_styles(),
    arg_required_else_help = true
)]
struct App {
    #[clap(subcommand)]
    cmd: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    #[clap(visible_alias = "a", arg_required_else_help = true)]
    Add {
        /// The name of the plugin, same as the extracted folder name.
        name: String,
        /// The GitHub repository to download from.
        repo: String,
        #[clap(short, long)]
        /// The target version of the plugin.
        version: Option<String>,
    },

    #[clap(visible_alias = "u", arg_required_else_help = true)]
    Update {
        #[clap(num_args = 1..)]
        /// The name of the plugins to update.
        name: Vec<String>,
        #[clap(short, long)]
        /// Update all plugins.
        all: bool,
    },

    #[clap(visible_alias = "r", arg_required_else_help = true)]
    Remove {
        #[clap(num_args = 1..)]
        /// The name of the plugins to remove.
        name: Vec<String>,
    },

    #[clap(visible_alias = "l")]
    /// List all installed plugins.
    List,
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::default()
        .usage(styling::AnsiColor::BrightGreen.on_default())
        .header(styling::AnsiColor::BrightGreen.on_default())
        .literal(styling::AnsiColor::BrightCyan.on_default())
        .invalid(styling::AnsiColor::BrightYellow.on_default())
        .error(styling::AnsiColor::BrightRed.on_default().bold())
        .valid(styling::AnsiColor::BrightGreen.on_default())
        .placeholder(styling::AnsiColor::Cyan.on_default())
}

/// Print an error message to stderr.
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        eprintln!("{} {}", "error:".bright_red().bold(), $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!("{} {}", "error:".bright_red().bold(), format!($fmt, $($arg)*))
    };
}

fn error_exit0<T>(msg: T)
where
    T: std::fmt::Display,
{
    error!(msg);
    process::exit(0);
}

fn main() {
    let args = App::parse();
    match config::Config::new() {
        Ok(mut config) => match args.cmd {
            TopCommand::Add {
                name,
                repo,
                version,
            } => config.add(name, repo, version).unwrap_or_else(error_exit0),
            TopCommand::Update { name, all } => {
                if all {
                    println!("Updating all plugins");
                } else {
                    println!("Updating plugins: {:?}", name);
                }
            }
            TopCommand::Remove { name } => {
                println!("Removing plugins: {:?}", name);
            }
            TopCommand::List => {
                println!("Listing plugins");
            }
        },
        Err(e) => error_exit0(e),
    }
}
