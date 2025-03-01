mod api;
mod cli;
mod config;

use clap::Parser;

use cli::Args;
use config::Config;

fn main() {
    let args = Args::parse();
    let config = Config::load();
    args.execute(config);
}
