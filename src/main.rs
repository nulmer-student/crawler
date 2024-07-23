#[allow(dead_code)]

mod config;
mod interface;
mod miner;
mod runner;

use config as crawler_config;

use clap::{arg, Command, ArgMatches};
use std::path::PathBuf;
use log;
use log4rs::config::{Appender, Config, Root};
use log4rs::append::{console::{ConsoleAppender, Target}, file::FileAppender};
use log4rs::encode::pattern::PatternEncoder;
use log4rs;

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

/// Setup application logging
fn setup_logging(config: &crawler_config::Config) -> log4rs::Handle {
    // let level = log::LevelFilter::Info;
    let level = log::LevelFilter::Debug;

    // Log entry pattern
    let pattern = Box::new(PatternEncoder::new(
        "{d(%Y-%m-%d %H:%M:%S)} {h({l}): <5} {T: <8} {({t}:{L}): <30} - {m}{n}"
    ));

    // Log to stderr
    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(pattern.clone())
        .build();


    // Log to a file
    let path = config.runner.log_dir.join("main.log");
    let logfile = FileAppender::builder()
        .encoder(pattern.clone())
        .build(path)
        .unwrap();

    // Configure
    let log_conf = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder().build("stderr", Box::new(stderr)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(level),
        )
        .unwrap();

    return log4rs::init_config(log_conf)
        .expect("failed to initialize logger");
}

fn main() {
    // Parse arguments
    let matches = cli().get_matches();

    // Load the configuration file
    let config_path = matches.get_one::<PathBuf>("config")
        .expect("required")
        .to_path_buf();
    let config = crawler_config::read_config(config_path);

    // Setup logging
    let _handle = setup_logging(&config);

    match matches.subcommand() {
        Some(("mine", sub)) => {
            let path = get_path(sub, "path");
            miner::mine_one(path, config);
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
