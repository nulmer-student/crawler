# Crawler

This tool allows a user-supplied program to be run on a large number of GitHub repositories.

## Building The Crawler

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

## Writing Your Own Instance
