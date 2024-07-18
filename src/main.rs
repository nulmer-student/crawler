mod config;
mod miner;

use clap::{arg, Command, ArgMatches};
use std::path::PathBuf;

fn cli() -> Command {
    Command::new("crawler")
        .about("Crawl github repositories")
        .arg_required_else_help(true)
        .arg(arg!(config: <CONFIG>)
             .value_parser(clap::value_parser!(PathBuf))
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("mine")
                .about("Mine a given repository")
                .arg_required_else_help(true)
                .arg(arg!(path: <PATH>)
                     .value_parser(clap::value_parser!(PathBuf)))
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
        _ => unreachable!(),
    }
}
