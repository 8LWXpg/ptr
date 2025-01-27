mod config;
mod polling;
mod util;

use clap::{builder::styling, CommandFactory, Parser, Subcommand};
use clap_complete::aot::PowerShell;
use std::{env, io, path::PathBuf, sync::LazyLock};
use util::self_update;

static PLUGIN_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
	PathBuf::from(&env::var("LOCALAPPDATA").unwrap())
		.join(r"Microsoft\PowerToys\PowerToys Run\Plugins")
});
static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
	PathBuf::from(&env::var("LOCALAPPDATA").unwrap())
		.join(r"Microsoft\PowerToys\PowerToys Run\Plugins\version.toml")
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
	/// Add a plugin.
	Add {
		/// The name of the plugin, can be anything.
		name: String,
		/// The GitHub repository identifier or URL of the plugin.
		repo: String,
		#[clap(short, long)]
		/// The target version of the plugin.
		version: Option<String>,
	},

	#[clap(visible_alias = "u", arg_required_else_help = true)]
	/// Update plugins.
	Update {
		#[clap(num_args = 1..)]
		/// The name of the plugins to update.
		name: Vec<String>,
		#[clap(short, long)]
		/// Update all plugins.
		all: bool,
		#[clap(short, long)]
		/// Version to update to.
		version: Option<Vec<String>>,
	},

	#[clap(visible_alias = "r", arg_required_else_help = true)]
	/// Remove plugins.
	Remove {
		#[clap(num_args = 1..)]
		/// The name of the plugins to remove.
		name: Vec<String>,
	},

	#[clap(visible_alias = "l")]
	/// List all installed plugins.
	List,

	#[clap(visible_alias = "p", arg_required_else_help = true)]
	/// Pin plugins so it's not updated with `update --all`.
	Pin {
		#[clap(subcommand)]
		cmd: PinSubcommand,
	},

	#[clap(visible_alias = "i")]
	/// Import plugins from configuration file.
	Import {
		#[clap(short, long)]
		/// Update the configuration file without downloading the plugin.
		dry_run: bool,
	},

	#[clap()]
	/// Restart PowerToys.
	Restart,

	#[clap()]
	/// Self update to latest.
	SelfUpdate,

	#[clap()]
	/// Generate shell completion (PowerShell).
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
		TopCommand::Import { dry_run } => {
			let mut config = config::Config::import().unwrap_or_else(|e| exit!(e));
			if dry_run {
				config.save().unwrap_or_else(|e| exit!(e));
			} else {
				config.import_plugins();
			}
		}
		TopCommand::SelfUpdate => self_update().unwrap_or_else(|e| exit!(e)),
		_ => {
			let mut config = config::Config::new().unwrap_or_else(|e| exit!(e));
			match args.cmd {
				TopCommand::Add {
					name,
					repo,
					version,
				} => config
					.add(
						&name,
						if let Some(repo) = repo.strip_prefix("https://github.com/") {
							repo.to_string()
						} else {
							repo
						},
						version,
					)
					.unwrap_or_else(|e| exit!(e)),
				TopCommand::Update { name, all, version } => {
					if all {
						config.update_all();
					} else {
						config.update(name, version);
					}
				}
				TopCommand::Remove { name } => config.remove(name),
				TopCommand::Pin { cmd } => match cmd {
					PinSubcommand::Add { name } => config.pin_add(name),
					PinSubcommand::List => config.pin_list(),
					PinSubcommand::Remove { name } => config.pin_remove(name),
					PinSubcommand::Reset => config.pin_reset(),
				},
				TopCommand::List => print!("{}", config),
				TopCommand::Restart => config.restart(),
				TopCommand::Completion => clap_complete::generate(
					PowerShell,
					&mut App::command(),
					"ptr",
					&mut io::stdout(),
				),
				_ => unreachable!(),
			}
		}
	}
}
