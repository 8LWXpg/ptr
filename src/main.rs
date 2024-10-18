mod config;
mod polling;
mod util;

use clap::{builder::styling, Parser, Subcommand};
use std::{env, path::PathBuf, sync::LazyLock};

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
		/// The GitHub repository of the plugin.
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

	#[clap(visible_alias = "i")]
	/// Import plugins from configuration file.
	Import {
		#[clap(short, long)]
		/// Dry run, can be used to update the configuration file.
		dry_run: bool,
	},

	#[clap()]
	/// Restart PowerToys
	Restart,
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
		TopCommand::Import { dry_run } => match config::Config::import() {
			Ok(mut config) => {
				if dry_run {
					config.save().unwrap_or_else(|e| exit!(e));
				} else {
					config.import_plugins();
				}
			}
			Err(e) => exit!(e),
		},
		_ => match config::Config::new() {
			Ok(mut config) => match args.cmd {
				TopCommand::Add {
					name,
					repo,
					version,
				} => config.add(name, repo, version).unwrap_or_else(|e| exit!(e)),
				TopCommand::Update { name, all } => {
					if all {
						config.update_all();
					} else {
						config.update(name);
					}
				}
				TopCommand::Remove { name } => config.remove(name),
				TopCommand::List => print!("{}", config),
				TopCommand::Restart => config.restart(),
				_ => unreachable!(),
			},
			Err(e) => exit!(e),
		},
	}
}
