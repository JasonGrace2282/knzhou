mod api;
mod cli;
mod config;

use clap::Parser;

use cli::Args;
use config::Config;
use env_logger::Env;

fn main() {
    // Default to log::warn
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let args = Args::parse();
    let config = Config::load();
    args.execute(config);
}
