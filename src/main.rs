#[allow(dead_code)]

mod config;
mod interface;
mod miner;
mod runner;

use clap::{arg, Command, ArgMatches};
use std::path::PathBuf;

fn cli() -> Command {
    Command::new("crawler")
        .about("Crawl github repositories")
        // Configuration
        .arg_required_else_help(true)
        .arg(arg!(config: <CONFIG>)
             .value_parser(clap::value_parser!(PathBuf))
        )
        // Mine a single repository
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("mine")
                .about("Only mine a given repository")
                .arg_required_else_help(true)
                .arg(arg!(path: <PATH>)
                     .value_parser(clap::value_parser!(PathBuf)))
        )
        // Start the crawler
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("crawl")
                .about("Mine all matching repositories")
        )
        // Search for repositories
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("search")
                .about("Only search for repositories")
        )
}

fn get_path(args: &ArgMatches, name: &str) -> PathBuf {
    args.get_one::<PathBuf>(name)
        .expect("required")
        .to_path_buf()
}

fn main() {
    // Parse arguments
    let matches = cli().get_matches();

    // Load the configuration file
    let config_path = matches.get_one::<PathBuf>("config")
        .expect("required")
        .to_path_buf();
    let config = config::read_config(config_path);

    match matches.subcommand() {
        Some(("mine", sub)) => {
            let path = get_path(sub, "path");
            miner::mine(&path, &config);
        },
        Some(("crawl", _sub)) => {
            runner::crawl(&config);
        },
        Some(("search", _sub)) => {
            runner::search(&config);
        },
        _ => unreachable!(),
    }
}
