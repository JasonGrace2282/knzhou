use clap::{Parser, Subcommand, ValueEnum};
use rayon::iter::IntoParallelIterator;
use reqwest::blocking::Client;

use crate::api;
use rayon::prelude::*;
use std::path::PathBuf;

/// Knzhou is a command-line tool for keeping knzhou handouts
/// up to date.
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Actions,
}

#[derive(Debug, Subcommand)]
pub enum Actions {
    Update { handout: Option<String> },
    Config { action: ConfigActions },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ConfigActions {
    Get,
}

impl Args {
    pub fn execute(&self, config: crate::Config) {
        self.command.execute(config);
    }
}

impl Actions {
    pub fn execute(&self, config: crate::Config) {
        match self {
            Self::Update { handout } => self.update(config, handout),
            Self::Config { action } => self.handle_config(config, action),
        }
    }

    fn update(&self, config: crate::Config, handout: &Option<String>) {
        let client = Client::new();
        if let Some(handout) = handout {
            let output = handout_output_file(&config, handout);
            if let Err(e) = api::fetch_handout(&client, handout, output) {
                log::error!("{}", e);
                std::process::exit(1);
            }
            return;
        }
        let files = api::fetch_handouts(&client);
        files
            .tree
            .into_par_iter()
            .filter(|entry| {
                log::debug!(
                    "Checking if file path is a handout: {}",
                    entry.path.display()
                );
                let path = &entry.path;
                path.extension().is_some_and(|ext| ext == "pdf")
                    && path
                        .parent()
                        .is_some_and(|p| p == PathBuf::from("handouts"))
            })
            .for_each(|op| {
                log::debug!("Fetching handout: {op:?}");
                let handout = match op.path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s,
                    None => {
                        return;
                    }
                };
                let output = handout_output_file(&config, handout);
                if let Err(e) = api::fetch_handout(&client, handout, output) {
                    log::error!("{}", e);
                }
            });
    }

    fn handle_config(&self, config: crate::Config, action: &ConfigActions) {
        match action {
            ConfigActions::Get => {
                println!(
                    "Config file located at {}",
                    config.disk_location().display()
                );
            }
        }
    }
}

fn handout_output_file(config: &crate::Config, handout: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    cwd.join(format!(
        "{}.pdf",
        config.format.replace("{handout}", handout)
    ))
}
