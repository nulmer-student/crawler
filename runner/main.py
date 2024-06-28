#!/usr/bin/env python3

from miner import DEFAULT_MINER, Miner

import sys

# Setup logging
import logging
logger = logging.getLogger(__name__)


def main():
    # Setup logging
    logging.basicConfig(filename="./log", level=logging.INFO)
    logging.getLogger().addHandler(logging.StreamHandler(sys.stdout))

    # TODO: Search for repos

    # NOTE: Test miner
    miner = Miner()
    miner._mine_one("https://github.com/wg/wrk.git", "wrk")

    pass


if __name__ == "__main__":
    main()
