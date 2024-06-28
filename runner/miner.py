#!/usr/bin/env python3

from database import InternDB

import git
from pathlib import Path
import os
import subprocess

# Setup logging
import logging
logger = logging.getLogger(__name__)


REPO_DIR=Path("/tmp/crawler-repos")
DEFAULT_MINER=Path("../miner/build/bin/miner")


class Miner:
    def __init__(self, repo_dir=REPO_DIR, miner=DEFAULT_MINER, intern=None):
        self.repo_dir = repo_dir

        # Check if the miner exists
        self.miner = miner
        if not os.path.exists(self.miner):
            logger.error(f"Failed to find miner: '{self.miner}'")
            exit(1)

        # Check if the intern script exists
        self.intern = intern
        if self.intern is not None:
            if not os.path.exists(self.intern):
                logger.error(f"Failed to find intern script: '{self.intern}'")
                exit(1)

    def _mine_one(self, url, name):
        repo = self._clone(url, name)
        self._mine(repo)

    def _mine(self, repo):
        # Run the miner
        pipe = subprocess.Popen(
            [self.miner, repo.working_dir],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        output = pipe.communicate()[0]

        # Run the result interning script on the output of the miner
        if self.intern is None:
            self._intern(output)
        else:
            pass

    def _intern(self, output):
        """Store the results of running the miner into the database."""
        print(output)

    def _clone(self, url, name):
        """Clone a repository to REPO_DIR."""

        # If the repository is already cloned, skip it
        logger.info(f"Cloning '{name}'")
        dir = self.repo_dir / name
        repo = None

        # FIXME: Update to latest version
        if os.path.exists(dir):
            logger.info(f"Repository '{name}' exists at '{dir}'")
            repo = git.Repo.init(dir)

        # Otherwise, clone the repository
        else:
            try:
                logger.info(f"Cloning {name} to '{dir}'")
                repo = git.Repo.clone_from(url, dir, depth=1)
            except git.exc.GitCommandError:
                logger.info(f"Failed to clone '{name}'")
            else:
                logger.info(f"Finished cloning '{name}'")

        return repo