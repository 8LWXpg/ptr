pub mod config;
pub mod polling;
pub mod util;

use std::{env, path::PathBuf, sync::LazyLock};

pub static PLUGIN_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
	PathBuf::from(&env::var("LOCALAPPDATA").unwrap())
		.join(r"Microsoft\PowerToys\PowerToys Run\Plugins")
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
	PathBuf::from(&env::var("LOCALAPPDATA").unwrap())
		.join(r"Microsoft\PowerToys\PowerToys Run\Plugins\version.toml")
});
