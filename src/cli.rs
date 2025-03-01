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
            api::fetch_handout(&client, handout, output);
            return;
        }
        let files = api::fetch_handouts(&client);
        files
            .tree
            .into_par_iter()
            .filter(|entry| {
                entry
                    .path
                    .parent()
                    .map(|p| p == PathBuf::from("handouts"))
                    .is_some_and(|x| x)
            })
            .for_each(|op| {
                let handout = op.path.file_name().unwrap().to_str().unwrap();
                let output = handout_output_file(&config, handout);
                api::fetch_handout(&client, handout, output);
            });
    }
}

fn handout_output_file(config: &crate::Config, handout: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    cwd.join(config.format.replace("{handout}", handout))
}
