#!/usr/bin/env python3

from miner import DEFAULT_MINER, Miner
from search import Search
from env import load_env

import sys

# Setup logging
import logging
logger = logging.getLogger(__name__)


def main():
    # Setup logging
    logging.basicConfig(filename="./log", level=logging.INFO)
    logging.getLogger().addHandler(logging.StreamHandler(sys.stdout))

    # Load environment variables
    env = load_env()

    # Search for repos
    # searcher = Search(env)
    # searcher.run()

    # Test miner
    miner = Miner(env)
    miner.mine_all()

    pass


if __name__ == "__main__":
    main()
