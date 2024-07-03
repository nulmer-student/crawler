#!/usr/bin/env python3

from database import Database

import git
from pathlib import Path
import os
import subprocess
import re

# Setup logging
import logging
logger = logging.getLogger(__name__)


REPO_DIR=Path("/tmp/crawler-repos")
DEFAULT_MINER=Path("../miner/build/bin/miner")


class InternDB(Database):
    def _init_db(self):
        super()._init_db()

    def add_match(self, path, line, col, vector, tile, si):
        # Ensure that there is an entry in the file table
        file_id = self._ensure_file(path)

        # Insert the match
        match_id = self._new_match_id(file_id)
        self.cursor.execute(
            "insert into matches values (?, ?, ?, ?, ?, ?, ?)",
            (match_id, file_id, line, col, vector, tile, si)
        )

        self.connection.commit()

    def _ensure_file(self, path):
        "Ensure that file PATH exists, & return it's id."
        # If the file exists, return its id
        self.cursor.execute(
            "select file_id from files where path = ?",
            (path,)
        )

        if self.cursor.rowcount == 1:
            return self.cursor.fetchone()[0]

        # Otherwise, insert the file
        if self.cursor.rowcount == 0:
            id = self._new_file_id()
            self.cursor.execute(
                "insert into files values (?, ?)",
                (id, path)
            )
            self.connection.commit()
            return id


    def _new_file_id(self):
        """Gererate a unique file id."""
        self.cursor.execute("select ifnull(max(file_id) + 1, 0) from files")
        id = self.cursor.fetchone()
        self.connection.commit()
        return id[0]

    def _new_match_id(self, file_id):
        """Gererate a unique file id."""
        self.cursor.execute(
            "select ifnull(max(match_id) + 1, 0) from matches where file_id = ?",
            (file_id,)
        )
        id = self.cursor.fetchone()
        self.connection.commit()
        return id[0]


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

        # Connect to the database
        self.db = InternDB()

    def mine_all(self):
        pass

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
            self._run_intern_script(output)

    def _run_intern_script(self, output):
        pass

    def _intern(self, output):
        """Store the results of running the miner into the database."""
        pattern = r"([^,]+),(\d+),(\d+),(\d+),(\d+),(\d+)\n"
        for match in re.finditer(pattern, output.decode("utf-8")):
            assert(len(match.groups()) == 6)
            path   = match.group(1)
            line   = match.group(2)
            col    = match.group(3)
            vector = match.group(4)
            tile   = match.group(5)
            si     = match.group(6)
            self.db.add_match(path, line, col, vector, tile, si)

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
