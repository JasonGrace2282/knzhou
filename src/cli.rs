use std::{path, sync::RwLock};

use clap::{Args, Parser, Subcommand, ValueEnum};
use reqwest::blocking::Client;

use crate::{api, config::Lockfile};
use rayon::prelude::*;
use std::path::PathBuf;

/// Knzhou is a command-line tool for keeping knzhou handouts
/// up to date.
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Actions,
}

#[derive(Debug, Subcommand)]
pub enum Actions {
    Update {
        handout: Option<String>,
    },
    Config {
        action: ConfigActions,
    },
    Hours {
        #[command(subcommand)]
        action: HoursActions,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ConfigActions {
    Get,
}

#[derive(Debug, Clone, Subcommand)]
pub enum HoursActions {
    Log {
        #[clap(flatten)]
        hours: HoursLogged,
    },
}

#[derive(Debug, Clone, Args)]
#[group(required = true)]
pub struct HoursLogged {
    #[clap(default_value_t, short, long)]
    focused: u32,
    #[clap(default_value_t, short, long)]
    unfocused: u32,
}

const DB_TABLE: &str = "knzhou_hours";

impl CliArgs {
    pub fn execute(&self, config: crate::Config) {
        self.command.execute(config);
    }
}

impl Actions {
    pub fn execute(&self, config: crate::Config) {
        match self {
            Self::Update { handout } => self.update(config, handout),
            Self::Config { action } => self.handle_config(config, action),
            Self::Hours { action } => self.handle_hours(action),
        }
    }

    fn handle_hours(&self, action: &HoursActions) {
        let Some(mut db_dir) = dirs::data_dir() else {
            log::error!("Could not find directory to store data in!");
            return;
        };
        db_dir.push("knzhou");

        match action {
            HoursActions::Log { hours } => {
                if let Err(e) = self.add_hours(db_dir, hours) {
                    log::error!("Error adding hours: {e}");
                }
            }
        }
    }

    fn add_hours(&self, db_dir: path::PathBuf, hours: &HoursLogged) -> rusqlite::Result<()> {
        std::fs::create_dir_all(&db_dir).expect("Should be able to create data directory.");
        let db_path = db_dir.join("knzhou.db");
        let conn = rusqlite::Connection::open(&db_path)?;
        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {DB_TABLE} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                focused FLOAT,
                unfocused FLOAT,
                day DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
            ),
            [],
        )?;
        conn.execute(
            &format!("INSERT INTO {DB_TABLE} (focused, unfocused) VALUES (?1, ?2)"),
            [hours.focused, hours.unfocused],
        )?;
        Ok(())
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
                    log::info!("Updated handout: {handout}");
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
