use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::api;

pub const LOCK_FILE: &str = "knzhou.lock.toml";

const APP_NAME: &str = "knzhou";
const CONFIG_NAME: &str = "knzhou";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: "{handout}".to_string(),
        }
    }
}

impl Config {
    /// Loads the configuration from disk, handing io errors gracefully.
    pub fn load() -> Self {
        let config: Result<Config, _> = confy::load(APP_NAME, CONFIG_NAME);
        if let Ok(config) = config {
            config.validate();
            return config;
        }
        let e = config.unwrap_err();
        match e {
            confy::ConfyError::BadTomlData(e) => {
                log::error!("Syntax error in configuration file: {}", e);
            }
            _ => log::error!("Error loading configuration: {}", e),
        };
        std::process::exit(1);
    }

    fn validate(&self) {
        if !self.format.contains("{handout}") {
            log::error!("Configuration format must contain {{handout}}.");
            std::process::exit(1);
        }
    }

    pub fn disk_location(&self) -> std::path::PathBuf {
        match confy::get_configuration_file_path(APP_NAME, CONFIG_NAME) {
            Ok(path) => path,
            Err(e) => {
                log::error!("Error getting configuration file path: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Lockfile {
    version: u8,
    tree: Vec<api::WebsiteTreeEntry>,
}

impl Lockfile {
    pub fn load() -> Self {
        let path = std::path::PathBuf::from(LOCK_FILE);
        if !path.exists() {
            return Lockfile::default();
        }
        let lock = std::fs::read_to_string(path);
        if lock.is_err() {
            log::error!("Error reading lockfile: {}", lock.unwrap_err());
            std::process::exit(1);
        }
        let lockfile = toml::from_str(&lock.unwrap());
        if lockfile.is_err() {
            log::warn!("Error parsing lockfile: {}", lockfile.unwrap_err());
            return Default::default();
        }
        lockfile.unwrap()
    }

    pub fn update_entry(&mut self, entry: api::WebsiteTreeEntry) {
        let index = self.tree.iter().position(|e| e.path == entry.path);
        if let Some(index) = index {
            self.tree[index] = entry;
        } else {
            self.tree.push(entry);
        }
    }

    #[inline]
    pub fn check_exists(&self, entry: &api::WebsiteTreeEntry) -> bool {
        self.tree.iter().any(|e| e == entry)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = std::path::PathBuf::from("knzhou.lock.toml");
        let mut file = std::fs::File::create(path)?;
        file.write_all(toml::to_string(self).unwrap().as_bytes())?;
        Ok(())
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            version: 1,
            tree: vec![],
        }
    }
}

impl From<api::WebsiteTree> for Lockfile {
    fn from(tree: api::WebsiteTree) -> Self {
        Self {
            version: 1,
            tree: tree.tree,
        }
    }
}
