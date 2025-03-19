use std::sync::RwLock;

use clap::{Args, Parser, Subcommand, ValueEnum};
use reqwest::blocking::Client;

use crate::{api, config::Lockfile, db};
use anyhow::Result;
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
    Show {
        #[clap(long, default_value_t)]
        detailed: bool,
    },
}

#[derive(Debug, Clone, Args)]
#[group(required = true)]
pub struct HoursLogged {
    #[clap(default_value_t, short, long)]
    focused: f32,
    #[clap(default_value_t, short, long)]
    unfocused: f32,
}

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
        let db_result = db::Database::new();
        if db_result.is_err() {
            log::error!("Error opening database: {:?}", db_result.unwrap_err());
            return;
        }
        let db = db_result.unwrap();
        match action {
            HoursActions::Log { hours } => {
                let res = db.add_hours(&hours.clone().into());
                if res.is_err() {
                    log::error!("Error adding hours: {:?}", res.unwrap_err());
                }
            }
            HoursActions::Show { detailed } => {
                let result = self.show_hours_logged(db, *detailed);
                if result.is_err() {
                    log::error!("{:?}", result.unwrap_err());
                }
            }
        }
    }

    fn show_hours_logged(&self, db: db::Database, detailed: bool) -> Result<()> {
        if detailed {
            let mut empty = true;
            let lines = "-".repeat(20);
            for row in db.detailed_hours_logged()? {
                let db::StudySession {
                    focused,
                    unfocused,
                    day,
                } = row;
                // a nice separator for formatting
                if !empty {
                    println!("{lines}")
                }

                if let Some(datetime) = day {
                    let day = datetime.to_zoned(jiff::tz::TimeZone::system()).unwrap();
                    println!("Study session on {}", day.date());
                }
                println!("Focused hours: {focused}\nUnfocused hours: {unfocused}");
                empty = false;
            }
            if empty {
                println!("No study sessions logged, lock in!!");
            }
        } else {
            let (focused, unfocused) = db.total_hours_logged()?;
            println!("Focused Studying: {focused} hours\nAdditional Studying: {unfocused} hours");
        }
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

impl HoursLogged {
    /// Returns the number of focused and unfocused hours
    pub fn hours(&self) -> (f32, f32) {
        (self.focused, self.unfocused)
    }
}

impl From<db::StudySession> for HoursLogged {
    fn from(session: db::StudySession) -> Self {
        Self {
            focused: session.focused,
            unfocused: session.unfocused,
        }
    }
}

pub fn handout_output_file(config: &crate::Config, handout: &str) -> PathBuf {
    PathBuf::from(format!(
        "{}.pdf",
        config.format.replace("{handout}", handout)
    ))
}
