use anyhow::{bail, Result};
use colored::Colorize;
use core::fmt;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tabwriter::TabWriter;

use crate::polling;
use crate::util::{get_powertoys_path, kill_ptr, start_ptr};
use crate::{add, error, exit, gh_dl, remove, up_to_date, CONFIG_PATH, PLUGIN_PATH};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
	arch: Arch,
	pt_path: PathBuf,
	#[serde(serialize_with = "sort_keys")]
	plugins: HashMap<String, Plugin>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportConfig {
	plugins: HashMap<String, Plugin>,
}

fn sort_keys<T, S>(value: &HashMap<String, T>, serializer: S) -> Result<S::Ok, S::Error>
where
	T: Serialize,
	S: Serializer,
{
	value
		.iter()
		.collect::<BTreeMap<_, _>>()
		.serialize(serializer)
}

impl Config {
	pub fn new() -> Result<Self> {
		if CONFIG_PATH.exists() {
			Ok(toml::from_str(&fs::read_to_string(&*CONFIG_PATH).unwrap())?)
		} else {
			let pt_path = get_powertoys_path()?;
			Ok(Self {
				arch: Arch::default(),
				pt_path,
				plugins: HashMap::new(),
			})
		}
	}

	/// Ignore configs unrelated to plugins.
	pub fn import() -> Result<Self> {
		let pt_path = get_powertoys_path()?;
		let import_config: ImportConfig =
			toml::from_str(&fs::read_to_string(&*CONFIG_PATH).unwrap())?;
		Ok(Self {
			arch: Arch::default(),
			pt_path,
			plugins: import_config.plugins,
		})
	}

	/// Note: This method already used in the other methods.
	pub fn save(&self) -> Result<()> {
		fs::write(&*CONFIG_PATH, toml::to_string(self)?)?;
		Ok(())
	}

	pub fn restart(&self) {
		kill_ptr().unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
		start_ptr(&self.pt_path).unwrap_or_else(|e| exit!("Failed to start PowerToys: {}", e));
	}

	pub fn add(&mut self, name: String, repo: String, version: Option<String>) -> Result<()> {
		if let Entry::Vacant(e) = self.plugins.entry(name.clone()) {
			kill_ptr().unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
			let version = &e
				.insert(Plugin::add(&name, repo, version, self.arch.clone())?)
				.version;
			add!("{}@{}", name, version);
			start_ptr(&self.pt_path).unwrap_or_else(|e| error!("Failed to start PowerToys: {}", e));
			self.save()?;
			Ok(())
		} else {
			bail!("Plugin already exists")
		}
	}

	pub fn update(&mut self, names: Vec<String>, versions: Option<Vec<String>>) {
		kill_ptr().unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));

		// Update plugins with versions first.
		let without_versions = if let Some(versions) = versions {
			let (with_versions, without_versions) = names
				.split_at_checked(versions.len())
				.unwrap_or((&names, &[]));
			for (name, version) in with_versions.iter().zip(versions) {
				if let Some(plugin) = self.plugins.get_mut(name) {
					match plugin.update_to(name, self.arch.clone(), version) {
						Ok(updated) => {
							if updated {
								add!("{}@{}", name, plugin.version)
							} else {
								up_to_date!("{}@{}", name, plugin.version)
							}
						}
						Err(e) => error!("Failed to update {}: {}", name, e),
					}
				}
			}
			without_versions
		} else {
			&names
		};
		for name in without_versions {
			if let Some(plugin) = self.plugins.get_mut(name) {
				match plugin.update(name, self.arch.clone()) {
					Ok(updated) => {
						if updated {
							add!("{}@{}", name, plugin.version)
						} else {
							up_to_date!("{}@{}", name, plugin.version)
						}
					}
					Err(e) => error!("Failed to update {}: {}", name, e),
				}
			}
		}
		start_ptr(&self.pt_path).unwrap_or_else(|e| error!("Failed to start PowerToys: {}", e));
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}

	pub fn update_all(&mut self) {
		kill_ptr().unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
		for (name, plugin) in &mut self.plugins {
			match plugin.update(name, self.arch.clone()) {
				Ok(updated) => {
					if updated {
						add!("{}@{}", name, plugin.version)
					} else {
						up_to_date!("{}@{}", name, plugin.version)
					}
				}
				Err(e) => error!("Failed to update {}: {}", name, e),
			}
		}
		start_ptr(&self.pt_path).unwrap_or_else(|e| error!("Failed to start PowerToys: {}", e));
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}

	pub fn remove(&mut self, names: Vec<String>) {
		kill_ptr().unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
		for name in names {
			if let Some(plugin) = self.plugins.get(&name) {
				match plugin.remove(&name) {
					Ok(_) => {
						self.plugins.remove(&name);
						remove!(name);
					}
					Err(e) => error!("Failed to remove {}: {}", name, e),
				}
			}
		}
		start_ptr(&self.pt_path).unwrap_or_else(|e| error!("Failed to start PowerToys: {}", e));
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}

	pub fn import_plugins(&mut self) {
		let mut new_plugins: HashMap<String, Plugin> = HashMap::new();
		kill_ptr().unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
		for (name, plugin) in &self.plugins {
			match Plugin::add(
				name,
				plugin.repo.clone(),
				Some(plugin.version.clone()),
				self.arch.clone(),
			) {
				Ok(plugin) => {
					add!("{}@{}", name, &plugin.version);
					new_plugins.insert(name.clone(), plugin);
				}
				Err(e) => exit!("Failed to import {}: {}", name, e),
			}
		}
		start_ptr(&self.pt_path).unwrap_or_else(|e| error!("Failed to start PowerToys: {}", e));
		self.plugins = new_plugins;
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}
}

impl fmt::Display for Config {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut tw = TabWriter::new(vec![]);
		writeln!(&mut tw, "{}", "Plugins:".bright_green()).unwrap();
		let btree_map: BTreeMap<_, _> = self.plugins.iter().collect();
		for (name, plugin) in &btree_map {
			writeln!(
				&mut tw,
				"  {}\t{}\t{}",
				name.bright_cyan(),
				plugin.repo,
				plugin.version
			)
			.unwrap();
		}
		tw.flush().unwrap();
		write!(
			f,
			"{}",
			String::from_utf8(tw.into_inner().unwrap()).unwrap()
		)
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Arch {
	#[serde(rename = "x64")]
	X64,
	#[serde(rename = "arm64")]
	ARM64,
}

impl Default for Arch {
	fn default() -> Self {
		match std::env::consts::ARCH {
			"x86_64" => Arch::X64,
			"aarch64" => Arch::ARM64,
			_ => unreachable!(),
		}
	}
}

impl From<Arch> for &str {
	fn from(val: Arch) -> Self {
		match val {
			Arch::X64 => "x64",
			Arch::ARM64 => "arm64",
		}
	}
}

impl fmt::Display for Arch {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Arch::X64 => write!(f, "x64"),
			Arch::ARM64 => write!(f, "arm64"),
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Plugin {
	repo: String,
	version: String,
}

impl Plugin {
	fn add(name: &str, repo: String, version: Option<String>, arch: Arch) -> Result<Self> {
		let version = gh_dl!(name, &repo, version, &arch)?;
		Ok(Self { repo, version })
	}

	/// Update the plugin to the latest version.
	/// Return `true` if the version is updated.
	fn update(&mut self, name: &str, arch: Arch) -> Result<bool> {
		let version = gh_dl!(name, &self.repo, None, &arch, self.version.clone())?;
		if version != self.version {
			self.version = version;
			Ok(true)
		} else {
			Ok(false)
		}
	}

	/// Update the plugin to specific version.
	/// Return `true` if the version is updated.
	fn update_to(&mut self, name: &str, arch: Arch, version: String) -> Result<bool> {
		let version = gh_dl!(name, &self.repo, Some(version), &arch, self.version.clone())?;
		if version != self.version {
			self.version = version;
			Ok(true)
		} else {
			Ok(false)
		}
	}

	/// Remove the `PLUGIN_PATH/name` directory.
	fn remove(&self, name: &str) -> Result<()> {
		polling::remove_dir_all(&*PLUGIN_PATH.join(name))?;
		Ok(())
	}
}
