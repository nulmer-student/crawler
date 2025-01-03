# Crawler

This tool allows user-supplied code to be run on a large number of GitHub
repositories.

## Building The Crawler

The crawler depends on the following programs:

- Rust / Cargo
- Git
- MariaDB
- `find`
- `tar`

Instructions for building each crawler can be found in the crawler's sub
directory. For example, the SI crawler is located in `crawler_si`.

## Command Line Usage

The following examples assume you have a valid config located at `config.toml`.
Mine a single repository on disk with the following command:

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
  - `password`: Database user password. Leave blank if none.
  - `host`: Database host.
  - `database`: Database to use on the host.

## Writing Your Own Interface

The user supplied mining code is written as a trait object with the following methods:

- `init()`: Called once before any repositories are mined. Does nothing by default.
- `preprocess()`: Called once for each file, with the result being using for all further compilations. Loads the file verbatim by default.
- `compile()`: Called for each file with each header combination. Results are collected and passed to the `intern()` method.
- `intern()`: Called after all compilation has finished on the results of all `compile()` calls.

Only the `compile()` and `intern()` methods are required.
The definition of the interface can be found in
[`src/interface/mod.rs`](src/interface/mod.rs), and an example implementation
can be found at [`src/interface/si.rs`](src/interface/si.rs).


## Structure of the Database

The database contains a number of tables by default:

- The `repos` table contains all repositories that match the search criteria.
  - `repo_id`: Unique integer identifier for each repository.
  - `name`: Human readable repository name. For example "nulmer-student/crawler".
  - `clone_url`: URL used to clone the repository.
  - `stars`: Repository star-count.
- The `mined` table contains the id's of repositories that have been successfully mined.
  - `repo_id`: Unique id of the repository.
- The `stats` table contains statistics about each mined repository.
  - `repo_id`: Unique id of the repository.
  - `n_files`: Number of source files mined in the repository.
  - `n_success`: Number of successfully compiled source files.
  - `n_errors`: Number of source files that failed to compile.
  - `time`: Time taken to mine this repository in milliseconds.

## Creating the Database

The following commands can be used to initialize the database with a user named
`user`, and a database named `db`:

``` shell
systemctl enable --now mariadb
mariadb -e "create database db"
mariadb -e "create user user@localhost"

# For running the crawler
mariadb -e "grant all privileges on db.* to user@localhost"

# If you only want to read the data:
mariadb -e "grant select on db.* to user@localhost"
```

A database dump can be read into some database `db` with the following command:

``` shell
mariadb -u user -h localhost db < dump.sql
```
