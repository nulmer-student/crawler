#!/usr/bin/env python3

from miner import DEFAULT_MINER, Miner
from search import Search
from env import load_env

import sys
from pathlib import Path

# Setup logging
import logging
logger = logging.getLogger(__name__)


def main():
    # Load environment variables
    env = load_env()

    # Setup logging
    logfile = Path(env["LOG_DIR"]) / "main"
    logging.basicConfig(filename=logfile, level=logging.INFO)
    logging.getLogger().addHandler(logging.StreamHandler(sys.stdout))

    # Search for repos
    searcher = Search(env)
    searcher.run()

    # Run the miner on each repository
    miner = Miner(env)
    miner.mine_all()


if __name__ == "__main__":
    main()
