mod api;
mod cli;
mod config;
mod db;

use clap::Parser;

use cli::CliArgs;
use config::Config;
use env_logger::Env;

fn main() {
    // Default to log::warn
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = CliArgs::parse();
    let config = Config::load();
    args.execute(config);
}
