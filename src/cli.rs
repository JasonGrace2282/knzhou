use std::sync::RwLock;

use clap::{Parser, Subcommand, ValueEnum};
use reqwest::blocking::Client;

use crate::{api, config::Lockfile};
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
                log::error!("{e}");
                std::process::exit(1);
            }
            return;
        }
        let lockfile = RwLock::new(Lockfile::load());
        let files = api::fetch_handouts(&client);
        files
            .tree
            .par_iter()
            .filter(|&entry| {
                let path = &entry.path;
                path.extension().is_some_and(|ext| ext == "pdf")
                    && path
                        .parent()
                        .is_some_and(|p| p == PathBuf::from("handouts"))
            })
            .for_each(|op| {
                let Some(handout) = op.path.file_stem().and_then(|s| s.to_str()) else {
                    return;
                };
                let output = handout_output_file(&config, handout);

                if lockfile.read().unwrap().check_exists(op) && output.exists() {
                    log::debug!("Skipping handout {handout}: already up to date");
                    return;
                }
                log::debug!("Fetching handout: {handout}");
                if let Err(e) = api::fetch_handout(&client, handout, output) {
                    log::error!("{e}");
                } else {
                    // update the lockfile upon success
                    lockfile.write().unwrap().update_entry(op.clone());
                }
            });
        if let Err(e) = lockfile.write().unwrap().save() {
            log::warn!("Error saving lockfile: {e}");
        }
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

pub fn handout_output_file(config: &crate::Config, handout: &str) -> PathBuf {
    PathBuf::from(format!(
        "{}.pdf",
        config.format.replace("{handout}", handout)
    ))
}
