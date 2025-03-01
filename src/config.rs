use serde::{Deserialize, Serialize};

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
        let config: Result<Config, _> = confy::load("knzhou", "knzhou");
        if let Ok(config) = config {
            config.validate();
            return config;
        }
        let e = config.unwrap_err();
        match e {
            confy::ConfyError::BadTomlData(e) => {
                eprintln!("Syntax error in configuration file: {}", e);
            }
            _ => eprintln!("Error loading configuration: {}", e),
        };
        std::process::exit(1);
    }

    fn validate(&self) {
        if !self.format.contains("{handout}") {
            eprintln!("Configuration format must contain {{handout}}.");
            std::process::exit(1);
        }
    }
}
