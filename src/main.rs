mod config;
mod polling;
mod util;

use clap::{builder::styling, CommandFactory, Parser, Subcommand};
use clap_complete::aot::PowerShell;
use colored::Colorize;
use std::{env, io, path::PathBuf, process::Command, sync::LazyLock};
use util::{self_update, ResultExit};

static PLUGIN_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
	PathBuf::from(&env::var("LOCALAPPDATA").unwrap()).join(r"Microsoft\PowerToys\PowerToys Run\Plugins")
});
static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
	PathBuf::from(&env::var("LOCALAPPDATA").unwrap()).join(r"Microsoft\PowerToys\PowerToys Run\Plugins\version.toml")
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

	#[clap(default_value = "false", long)]
	/// Do not restart PowerToys after plugin modification
	no_restart: bool,
}

#[derive(Subcommand)]
enum TopCommand {
	#[clap()]
	/// Try to find and add existing plugins to config
	Init,

	#[clap(visible_alias = "a", arg_required_else_help = true)]
	/// Add a plugin
	Add {
		/// Plugin name, can be anything
		name: String,
		/// GitHub repository identifier or URL of the plugin
		repo: String,
		#[clap(short, long)]
		/// Target version
		version: Option<String>,
		#[clap(short, long)]
		/// Asset match pattern (rust regex)
		pattern: Option<String>,
	},

	#[clap(visible_alias = "u", arg_required_else_help = true)]
	/// Update plugins
	Update {
		#[clap(num_args = 1..)]
		/// Name of the plugins to update
		name: Vec<String>,
		#[clap(short, long)]
		/// Update all plugins
		all: bool,
		#[clap(short, long)]
		/// Version to update
		version: Option<Vec<String>>,
	},

	#[clap(visible_alias = "r", arg_required_else_help = true)]
	/// Remove plugins
	Remove {
		#[clap(num_args = 1..)]
		/// Name of the plugins to remove.
		name: Vec<String>,
	},

	#[clap(visible_alias = "l")]
	/// List all installed plugins
	List,

	#[clap(visible_alias = "p", arg_required_else_help = true)]
	/// Pin plugins so it's not updated with `update --all`.
	Pin {
		#[clap(subcommand)]
		cmd: PinSubcommand,
	},

	#[clap(visible_alias = "i")]
	/// Import plugins from configuration file
	Import {
		#[clap(short, long)]
		/// Update the configuration file without downloading the plugin
		dry_run: bool,
	},

	#[clap()]
	/// Restart PowerToys
	Restart,

	#[clap()]
	/// Open config file in default editor
	Edit,

	#[clap()]
	/// Self update to latest
	SelfUpdate,

	#[clap()]
	/// Generate shell completion (PowerShell)
	Completion,
}

#[derive(Subcommand)]
enum PinSubcommand {
	#[clap(visible_alias = "a")]
	/// Add pins.
	Add {
		#[clap(num_args = 1..)]
		/// The name of the plugins to pin.
		name: Vec<String>,
	},
	#[clap(visible_alias = "r")]
	/// Remove pins
	Remove {
		#[clap(num_args = 1..)]
		/// The name of the plugins to pin.
		name: Vec<String>,
	},
	#[clap(visible_alias = "l")]
	/// List pins.
	List,
	/// Clear all pins.
	Reset,
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

fn main() {
	let args = App::parse();
	match args.cmd {
		TopCommand::Init => {
			if PathBuf::from(&*CONFIG_PATH).exists()
				&& util::prompt("Found existing config, override? [y/N]: ").exit_on_error() != "y"
			{
				return;
			}
			let config = config::Config::init().exit_on_error();
			println!("{config}");
			println!(
				"{} Some plugin may failed to find due to incomplete metadata.",
				"Note:".bright_blue()
			);
			println!("      If that occurs, please contact the plugin author.");
			config.save().exit_on_error();
		}
		TopCommand::Import { dry_run } => {
			let mut config = config::Config::import().exit_on_error();
			if dry_run {
				config.save().exit_on_error();
			} else {
				config.import_plugins(args.no_restart);
			}
		}
		TopCommand::SelfUpdate => self_update().exit_on_error(),
		_ => {
			let mut config = config::Config::new().exit_on_error();
			match args.cmd {
				TopCommand::Add {
					name,
					repo,
					version,
					pattern,
				} => config
					.add(
						&name,
						if let Some(repo) = repo.strip_prefix("https://github.com/") {
							repo.to_string()
						} else {
							repo
						},
						version,
						pattern,
						args.no_restart,
					)
					.exit_on_error(),
				TopCommand::Update { name, all, version } => {
					if all {
						config.update_all(args.no_restart);
					} else {
						config.update(name, version, args.no_restart);
					}
				}
				TopCommand::Remove { name } => config.remove(name, args.no_restart),
				TopCommand::Edit => {
					_ = Command::new("cmd")
						.args(["/c", (*CONFIG_PATH).to_str().unwrap()])
						.status()
						.unwrap_or_else(|e| exit!(e))
				}
				TopCommand::Pin { cmd } => match cmd {
					PinSubcommand::Add { name } => config.pin_add(name),
					PinSubcommand::List => config.pin_list(),
					PinSubcommand::Remove { name } => config.pin_remove(name),
					PinSubcommand::Reset => config.pin_reset(),
				},
				TopCommand::List => print!("{}", config),
				TopCommand::Restart => config.restart(),
				TopCommand::Completion => {
					clap_complete::generate(PowerShell, &mut App::command(), "ptr", &mut io::stdout())
				}
				_ => unreachable!(),
			}
		}
	}
}
