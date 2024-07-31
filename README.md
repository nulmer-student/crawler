# Crawler

This tool allows a user-supplied code to be run on a large number of GitHub
repositories.

## Building The Crawler

The crawler depends on the following programs:

- Rust / Cargo
- Git
- MariaDB

After cloning the repository, run the following command to build the crawler.

``` sh
cargo build
```

## Example Command Line Usage

The following examples assume you have a valid config located at `config.toml`.
Mine a repository on disk with the following command:

``` sh
cargo run -- config.toml mine /path/to/repo
```

All repositories matching the search criteria can be crawled using the following
command:

``` sh
cargo run -- config.toml crawl 
```

## Configuration File

The behaviour of the crawler is modified through the use of a configuration
file.
A sample configuration file for the `si` interface can be found
[here](example-config.toml). 
The fields of the configuration file have the following meanings:

- Interface
  - `name`: Name of the interface to use.
  - `args`: Table of interface specific configuration options.
- Miner
  - `threads`: Number of threads to use to mine each repository.
  - `tries`: Maximum number of possible header combinations to try for a single file.
- Runner
  - `threads`: Number of repositories to mine in parallel.
  - `min_stars`: The minimum number of stars for a repository to be searched.
  - `languages`: List of languages' source files to mine. Only supports `c`.
  - `github_api_key`: GitHub API key.
  - `log_level`: Level of log messages to print. Can be any of: `error`, `warn`, `info`, `debug`, and `trace`.
  - `log_dir`: Top level directory to place log files.
  - `temp_dir`: Directory where repositories are cloned to.
- Database
  - `user`: Database user.
  - `password`: Database password. Leave blank if none.
  - `host`: Database host.
  - `database`: Database to use on the host.

## Writing Your Own Interface

