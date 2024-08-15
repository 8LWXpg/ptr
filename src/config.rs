use anyhow::Result;
use colored::Colorize;
use core::fmt;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use tabwriter::TabWriter;
use tokio::runtime::Runtime;

use crate::{add, error, gh_dl, remove, CONFIG_PATH, PLUGIN_PATH};

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Arch {
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

#[derive(Serialize, Deserialize, Debug)]
struct Plugin {
    repo: String,
    version: String,
}

impl Plugin {
    fn add(repo: String, version: Option<String>, arch: Arch) -> Result<Self> {
        let rt = Runtime::new()?;
        let version = rt.block_on(gh_dl!(&repo, version, arch.into()))?;
        Ok(Self { repo, version })
    }

    fn update(&mut self, arch: Arch) -> Result<()> {
        let rt = Runtime::new()?;
        self.version = rt.block_on(gh_dl!(&self.repo, None, arch.into(), self.version.clone()))?;
        Ok(())
    }

    /// Remove the `PLUGIN_PATH/name` directory.
    fn remove(&self, name: &str) -> Result<()> {
        fs::remove_dir_all(&*PLUGIN_PATH.join(name))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    arch: Arch,
    #[serde(serialize_with = "sort_keys")]
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
            Ok(Self {
                arch: Arch::default(),
                plugins: HashMap::new(),
            })
        }
    }

    fn save(&self) -> Result<()> {
        fs::write(&*CONFIG_PATH, toml::to_string(self)?)?;
        Ok(())
    }

    pub fn add(&mut self, name: String, repo: String, version: Option<String>) -> Result<()> {
        if let Entry::Vacant(e) = self.plugins.entry(name.clone()) {
            let version = &e
                .insert(Plugin::add(repo, version, self.arch.clone())?)
                .version;
            add!("{}@{}", name, version);
            self.save()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin already exists"))
        }
    }

    pub fn update(&mut self, names: Vec<String>) {
        for name in names {
            if let Some(plugin) = self.plugins.get_mut(&name) {
                match plugin.update(self.arch.clone()) {
                    Ok(_) => add!("{}@{}", name, plugin.version),
                    Err(e) => error!("Failed to update {}: {}", name, e),
                }
            }
        }
        self.save()
            .unwrap_or_else(|e| error!("Failed to save config: {}", e));
    }

    pub fn update_all(&mut self) {
        for (name, plugin) in &mut self.plugins {
            match plugin.update(self.arch.clone()) {
                Ok(_) => add!("{}@{}", name, plugin.version),
                Err(e) => error!("Failed to update {}: {}", name, e),
            }
        }
        self.save()
            .unwrap_or_else(|e| error!("Failed to save config: {}", e));
    }

    pub fn remove(&mut self, names: Vec<String>) {
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
        self.save()
            .unwrap_or_else(|e| error!("Failed to save config: {}", e));
    }

    pub fn import(&mut self) {
        let mut new_plugins: HashMap<String, Plugin> = HashMap::new();
        for (name, plugin) in &self.plugins {
            match Plugin::add(
                plugin.repo.clone(),
                Some(plugin.version.clone()),
                self.arch.clone(),
            ) {
                Ok(plugin) => {
                    add!("{}@{}", name, &plugin.version);
                    new_plugins.insert(name.clone(), plugin);
                }
                Err(e) => error!("Failed to import {}: {}", name, e),
            }
        }
        self.plugins = new_plugins;
        self.save()
            .unwrap_or_else(|e| error!("Failed to save config: {}", e));
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
