#!/usr/bin/env python3

import mariadb


class Database:
    def __init__(self):
        """Connect to & initialize the database."""
        # Initialize the database
        self.connection = mariadb.connect(
            user="nju",
            password="",
            host="localhost",
            database="crawler"
        )
        self.cursor = self.connection.cursor()

        # Add tables if they don't exist
        self.cursor.execute("drop table if exists repos")
        self.cursor.execute(
            """
            create table repos (
                id          int,
                name        text,
                clone_url   text,
                stars       int,
                primary key (id)
            )
            """)
        self.connection.commit()

    def _insert_repo(self, repo):
        """Insert a repository into the database."""
        # Check if we already have this repo
        self.cursor.execute(
            "select * from repos where id = ?",
            (repo.id,)
        )

        # Don't insert if the repo is already there
        if self.cursor.rowcount == 0:
            self.cursor.execute(
                "insert into repos values (?, ?, ?, ?)",
                (repo.id, repo.name, repo.clone_url, repo.stars)
            )
        else:
            print(f"New: {repo.id}, {repo.name}, {repo.clone_url} {repo.stars}")
            print(self.cursor.fetchone())

    def insert_repos(self, repo_list):
        """Insert a list of repositories into the database."""
        try:
            # Attempt to insert each repo
            for repo in repo_list:
                self._insert_repo(repo)
            self.connection.commit()

        except Exception as e:
            print(f"Error in transaction: {e}")
            self.connection.rollback()
