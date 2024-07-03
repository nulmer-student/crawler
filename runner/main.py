#!/usr/bin/env python3

from miner import DEFAULT_MINER, Miner
from search import Search

import sys

# Setup logging
import logging
logger = logging.getLogger(__name__)


def main():
    # Setup logging
    logging.basicConfig(filename="./log", level=logging.INFO)
    logging.getLogger().addHandler(logging.StreamHandler(sys.stdout))

    # Search for repos
    # searcher = Search()
    # searcher.run()

    # Test miner
    miner = Miner()
    miner.mine_all()
    # miner._mine_one("https://github.com/wg/wrk.git", "wrk")
    # miner._mine_one("https://github.com/git/git", "git")

    pass


if __name__ == "__main__":
    main()
