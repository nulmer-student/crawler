#!/usr/bin/env python3

from database import Database
from repository import DBRepo

import datetime
import git
import os
from pathlib import Path
import subprocess
import re
import time

# Setup logging
import logging
logger = logging.getLogger(__name__)


REPO_DIR=Path("/tmp/crawler-repos")
DEFAULT_MINER=Path("../miner/build/bin/miner")

class MineDB(Database):
    def _init_db(self):
        super()._init_db()
        # Clear the tables
        # self.cursor.execute("delete from mined")
        # self.cursor.execute("delete from matches")
        # self.cursor.execute("delete from files")
        self.connection.commit()

    def next_repo(self):
        """Return a repo_id that hasn't been mined yet."""
        self.cursor.execute(
            """
            select repo_id from repos
            except
            select repo_id from mined
            limit 1
            """)

        # Return None if there are no more repos
        if self.cursor.rowcount == 1:
            return self.cursor.fetchone()[0]
        else:
            return None

    def set_mined(self, repo_id, success, error, time):
        try:
            self.cursor.execute(
                "insert into mined values (?, ?, ?, ?)",
                (repo_id, success, error, time)
            )
            self.connection.commit()

        except Exception as e:
            logger.error(f"Failed to add repo '{repo_id}' to mined: {e}")
            self.connection.rollback()

    def get_repo(self, repo_id):
        self.cursor.execute(
            "select * from repos where repo_id = ?",
            (repo_id,)
        )

        if self.cursor.rowcount == 1:
            repo = self.cursor.fetchone()
            return DBRepo(repo[0], repo[1], repo[2], repo[3])
        else:
            logger.exception(f"Missing repo '{repo_id}'")
            raise RuntimeError


class InternDB(Database):
    def add_match(self, path, line, col, vector, tile, si, repo_id):
        # Ensure that there is an entry in the file table
        file_id = self._ensure_file(path, repo_id)

        # Insert the match
        match_id = self._new_match_id(file_id)
        self.cursor.execute(
            "insert into matches values (?, ?, ?, ?, ?, ?, ?)",
            (match_id, file_id, line, col, vector, tile, si)
        )

        self.connection.commit()

    def _ensure_file(self, path, repo_id):
        "Ensure that file PATH exists, & return it's id."
        # If the file exists, return its id
        self.cursor.execute(
            "select file_id from files where path = ? and repo_id = ?",
            (path, repo_id)
        )

        if self.cursor.rowcount == 1:
            return self.cursor.fetchone()[0]

        # Otherwise, insert the file
        if self.cursor.rowcount == 0:
            id = self._new_file_id()
            self.cursor.execute(
                "insert into files values (?, ?, ?)",
                (id, repo_id, path)
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
        self.cursor.execute("select ifnull(max(match_id) + 1, 0) from matches")
        id = self.cursor.fetchone()
        self.connection.commit()
        return id[0]


class Miner:
    def __init__(self, env, repo_dir=REPO_DIR, miner=DEFAULT_MINER, intern=None):
        self.repo_dir = repo_dir
        self.env = env

        self.log_dir = self._log_dir()

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
        self.intern_db = InternDB()
        self.mine_db = MineDB()

    def _log_dir(self):
        dir = Path(self.env["LOG_DIR"]).resolve()
        now = datetime.datetime.now()
        dir = dir / Path(f"{now.year}-{now.month:02}-{now.day:02}--{now.hour:02}:{now.minute:02}:{now.second:02}")
        return dir

    def _repo_log(self, id, name):
        safe_name = name.replace("/", "-")
        return self.log_dir / Path(f"{safe_name}-{id}.log")

    def mine_all(self):
        while True:
            # Get the next repository
            id = self.mine_db.next_repo()
            if id is None:
                break

            # Mine the repository
            repo = self.mine_db.get_repo(id)
            logger.info(f"Mining '{repo}'")
            self._mine_one(repo)
            logger.info(f"Finished mining '{repo}'")

    def _mine_one(self, repo):
        real_repo = self._clone(repo.url, repo.name)
        self._mine(real_repo, repo.id, self._repo_log(repo.id, repo.name))

    def _mine(self, repo, id, logfile):
        s_time = time.time()

        # Create the log directory
        try:
            os.makedirs(logfile.parent)
        except FileExistsError:
            pass

        # Run the miner
        pipe = subprocess.Popen(
            [
                self.miner, self.env["CLANG"], repo.working_dir,
                "--log", logfile,
                "--threads", "12"
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        output = pipe.communicate()[0]

        # Run the result interning script on the output of the miner
        if self.intern is None:
            self._intern(output, id)
        else:
            self._run_intern_script(output, id)

        e_time = time.time()

        # Set this repository as mined
        suc, err = self._get_status(output)
        delta = e_time - s_time
        self.mine_db.set_mined(id, suc, err, delta)

    def _run_intern_script(self, output, id):
        pass

    def _intern(self, output, id):
        """Store the results of running the miner into the database."""
        # Add each match
        pattern = r"([^,]+),(\d+),(\d+),(\d+),(\d+),(\d+)\n"
        for match in re.finditer(pattern, output.decode("utf-8")):
            assert(len(match.groups()) == 6)
            path   = match.group(1)
            line   = match.group(2)
            col    = match.group(3)
            vector = match.group(4)
            tile   = match.group(5)
            si     = match.group(6)
            self.intern_db.add_match(path, line, col, vector, tile, si, id)

    def _get_status(self, output):
        output = output.decode("utf-8")
        suc = 0
        for m in re.finditer(r"Successful:\s+(\d+)", output):
            suc = int(m.group(1))

        err = 0
        for m in re.finditer(r"Errors:\s+(\d+)", output):
            err = int(m.group(1))

        return (suc, err)

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
