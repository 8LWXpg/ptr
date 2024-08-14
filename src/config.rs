use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{self, File},
};
use tokio::runtime::Runtime;

use crate::{add, gh_dl, util, CONFIG_PATH};

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Arch {
    #[serde(rename = "x64")]
    X64,
    #[serde(rename = "arm64")]
    ARM64,
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    arch: Arch,
    plugins: HashMap<String, Plugin>,
}

impl Config {
    pub fn new() -> Result<Self> {
        if CONFIG_PATH.exists() {
            Ok(toml::from_str(&fs::read_to_string(&*CONFIG_PATH).unwrap())?)
        } else {
            File::create(&*CONFIG_PATH)?;
            Ok(Self {
                arch: Arch::X64,
                plugins: HashMap::new(),
            })
        }
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

    fn save(&self) -> Result<()> {
        fs::write(&*CONFIG_PATH, toml::to_string(self)?)?;
        Ok(())
    }
}
