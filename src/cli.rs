use clap::{Parser, Subcommand};
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
        }
    }

    fn update(&self, config: crate::Config, handout: &Option<String>) {
        let client = Client::new();
        if let Some(handout) = handout {
            let output = handout_output_file(&config, handout);
            if let Err(e) = api::fetch_handout(&client, handout, output) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            return;
        }
        let files = api::fetch_handouts(&client);
        files
            .tree
            .into_par_iter()
            .filter(|entry| {
                let path = &entry.path;
                path.extension().is_some_and(|ext| ext == "pdf")
                    && path
                        .parent()
                        .map(|p| p == PathBuf::from("handouts"))
                        .is_some_and(|x| x)
            })
            .for_each(|op| {
                let handout = match op.path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s,
                    None => {
                        return;
                    }
                };
                let output = handout_output_file(&config, handout);
                if let Err(e) = api::fetch_handout(&client, handout, output) {
                    eprintln!("{}", e);
                }
            });
    }
}

fn handout_output_file(config: &crate::Config, handout: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    cwd.join(
        config
            .format
            .replace("{handout}", &format!("{}.pdf", handout)),
    )
}
