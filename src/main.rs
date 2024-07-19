#[allow(dead_code)]

mod config;
mod interface;
mod miner;

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
        // Interface
        .arg_required_else_help(true)
        .arg(arg!(interface: <INTERFACE>)
             .value_parser(clap::value_parser!(String))
        )
        // Mine a single repository
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

    // Load the interface
    let name = matches.get_one::<String>("interface")
        .expect("required")
        .to_string();
    let interface = interface::get_interface(&name);

    match matches.subcommand() {
        Some(("mine", sub)) => {
            let path = get_path(sub, "path");
            miner::mine(&path, &config, &interface);
        },
        _ => unreachable!(),
    }
}
