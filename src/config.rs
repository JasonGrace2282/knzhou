use serde::{Deserialize, Serialize};

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
