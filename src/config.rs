use anyhow::{bail, Result};
use colored::Colorize;
use core::fmt;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;
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
	/// Kill and run as admin
	admin: bool,
	pin: Option<HashSet<String>>,
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
				admin: true,
				pin: None,
				plugins: HashMap::new(),
			})
		}
	}

	/// Try to find plugins and add to config
	pub fn init() -> Result<()> {
		todo!()
	}

	/// Ignore configs unrelated to plugins.
	pub fn import() -> Result<Self> {
		let pt_path = get_powertoys_path()?;
		let import_config: ImportConfig =
			toml::from_str(&fs::read_to_string(&*CONFIG_PATH).unwrap())?;
		Ok(Self {
			arch: Arch::default(),
			pt_path,
			admin: true,
			pin: None,
			plugins: import_config.plugins,
		})
	}

	/// Note: This method already used in the other methods.
	pub fn save(&self) -> Result<()> {
		fs::write(&*CONFIG_PATH, toml::to_string(self)?)?;
		Ok(())
	}

	pub fn restart(&self) {
		kill_ptr(self.admin).unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
		start_ptr(&self.pt_path).unwrap_or_else(|e| exit!("Failed to start PowerToys: {}", e));
	}

	pub fn import_plugins(&mut self) {
		let mut new_plugins: HashMap<String, Plugin> = HashMap::new();
		kill_ptr(self.admin).unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
		for (name, plugin) in &mut self.plugins {
			match plugin.force_update(name, &self.arch) {
				Ok(()) => {
					add!(name, &plugin.version);
					new_plugins.insert(name.clone(), plugin.clone());
				}
				Err(e) => exit!("Failed to import {}: {}", name, e),
			}
		}
		start_ptr(&self.pt_path).unwrap_or_else(|e| error!("Failed to start PowerToys: {}", e));
		self.plugins = new_plugins;
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}

	pub fn add(
		&mut self,
		name: &str,
		repo: String,
		version: Option<String>,
		pattern: Option<String>,
	) -> Result<()> {
		if let Entry::Vacant(e) = self.plugins.entry(name.to_string()) {
			kill_ptr(self.admin).unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
			let version = &e
				.insert(Plugin::add(name, repo, version, &self.arch, pattern)?)
				.version;
			add!(name, version);
			start_ptr(&self.pt_path).unwrap_or_else(|e| error!("Failed to start PowerToys: {}", e));
			self.save()?;
			Ok(())
		} else {
			bail!("Plugin already exists")
		}
	}

	pub fn update(&mut self, names: Vec<String>, versions: Option<Vec<String>>) {
		kill_ptr(self.admin).unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));

		// Update plugins with versions first.
		let without_versions = if let Some(versions) = versions {
			let (with_versions, without_versions) = names
				.split_at_checked(versions.len())
				.unwrap_or((&names, &[]));
			for (name, version) in with_versions.iter().zip(versions) {
				if let Some(plugin) = self.plugins.get_mut(name) {
					match plugin.update_to(name, &self.arch, &version) {
						Ok(updated) => {
							if updated {
								add!(name, plugin.version)
							} else {
								up_to_date!(name, plugin.version)
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
				match plugin.update(name, &self.arch) {
					Ok(updated) => {
						if updated {
							add!(name, plugin.version)
						} else {
							up_to_date!(name, plugin.version)
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
		kill_ptr(self.admin).unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
		for (name, plugin) in &mut self.plugins {
			if let Some(pins) = &self.pin {
				if pins.contains(name) {
					continue;
				}
			}
			match plugin.update(name, &self.arch) {
				Ok(updated) => {
					if updated {
						add!(name, plugin.version)
					} else {
						up_to_date!(name, plugin.version)
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
		kill_ptr(self.admin).unwrap_or_else(|e| exit!("Failed to kill PowerToys: {}", e));
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

	pub fn pin_add(&mut self, names: Vec<String>) {
		if let Some(pins) = self.pin.as_mut() {
			names.into_iter().for_each(|n| {
				pins.insert(n);
			});
		} else {
			self.pin = Some(HashSet::from_iter(names));
		}
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}

	pub fn pin_remove(&mut self, names: Vec<String>) {
		if let Some(pins) = self.pin.as_mut() {
			names.iter().for_each(|n| {
				pins.remove(n);
			});
		} else {
			self.pin = Some(HashSet::from_iter(names));
		}
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}

	pub fn pin_list(&self) {
		if let Some(pins) = self.pin.as_ref() {
			pins.iter().for_each(|n| println!("{n}"));
		}
	}

	pub fn pin_reset(&mut self) {
		self.pin = None;
		self.save()
			.unwrap_or_else(|e| exit!("Failed to save config: {}", e));
	}
}

impl fmt::Display for Config {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut tw = TabWriter::new(vec![]);
		writeln!(&mut tw, "{}", "Plugins:".bright_green()).unwrap();
		let btree_map: BTreeMap<_, _> = self.plugins.iter().collect();
		for (name, plugin) in btree_map {
			writeln!(
				&mut tw,
				"  {}\t{}\t{}\t{}",
				name.bright_cyan(),
				plugin.repo,
				plugin.version,
				match plugin.pattern.as_ref() {
					Some(pattern) => pattern,
					None => "",
				},
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

impl fmt::Display for Arch {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Arch::X64 => write!(f, "x64"),
			Arch::ARM64 => write!(f, "arm64"),
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Plugin {
	repo: String,
	version: String,
	pattern: Option<String>,
}

impl Plugin {
	/// Add a plugin with the specified version, None for the latest version.
	fn add(
		name: &str,
		repo: String,
		version: Option<String>,
		arch: &Arch,
		pattern: Option<String>,
	) -> Result<Self> {
		let version = gh_dl!(name, &repo, version.as_deref(), arch, pattern.as_deref())?;
		Ok(Self {
			repo,
			version,
			pattern,
		})
	}

	/// Update the plugin to the latest version.
	/// Return `true` if the version is updated.
	fn update(&mut self, name: &str, arch: &Arch) -> Result<bool> {
		let version = gh_dl!(
			name,
			&self.repo,
			None,
			arch,
			&self.version,
			self.pattern.as_deref()
		)?;
		if version != self.version {
			self.version = version;
			Ok(true)
		} else {
			Ok(false)
		}
	}

	/// Update the plugin to specific version.
	/// Return `true` if the version is updated.
	fn update_to(&mut self, name: &str, arch: &Arch, version: &str) -> Result<bool> {
		let version = gh_dl!(
			name,
			&self.repo,
			Some(version),
			arch,
			&self.version,
			self.pattern.as_deref()
		)?;
		if version != self.version {
			self.version = version;
			Ok(true)
		} else {
			Ok(false)
		}
	}

	/// Update without checking current version.
	fn force_update(&mut self, name: &str, arch: &Arch) -> Result<()> {
		let version = gh_dl!(
			name,
			&self.repo,
			None,
			arch,
			&self.version,
			self.pattern.as_deref()
		)?;
		self.version = version;
		Ok(())
	}

	/// Remove the `PLUGIN_PATH/name` directory.
	fn remove(&self, name: &str) -> Result<()> {
		polling::remove_dir_all(&*PLUGIN_PATH.join(name))?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::io::Read;

	use super::*;

	#[test]
	fn generate_toml() {
		let config = Config {
			arch: Arch::X64,
			admin: true,
			pin: None,
			pt_path: "C:/Program Files/PowerToys/PowerToys.exe".into(),
			plugins: HashMap::new(),
		};
		let toml = toml::to_string_pretty(&config).unwrap();
		let mut file = fs::File::create("./test/test.toml").unwrap();
		file.write_all(toml.as_bytes()).unwrap();
	}

	#[test]
	fn test_breaking_config() {
		let mut file = fs::File::open("./test/test.toml").unwrap();
		let mut toml = String::new();
		file.read_to_string(&mut toml).unwrap();
		let config: Config = toml::from_str(&toml).unwrap();
		println!("{:?}", config);
	}
}
