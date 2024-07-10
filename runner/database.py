#!/usr/bin/env python3

import mariadb

class Database:
    def __init__(self, env):
        """Connect to & initialize the database."""
        # Initialize the database
        self.connection = mariadb.connect(
            user=env["DB_USER"],
            password="",
            host="localhost",
            database="crawler"
        )
        self.cursor = self.connection.cursor()
        self._init_db()

    def _init_db(self):
        """Setup the database."""
        self.cursor.execute(
            """
            create table if not exists repos (
                repo_id     int,
                name        text,
                clone_url   text,
                stars       int,
                primary key (repo_id)
            )
            """)

        self.cursor.execute(
            """
            create table if not exists mined (
                repo_id     int,
                n_success   int,
                n_error     int,
                time        float,
                primary key (repo_id),
                foreign key (repo_id) references repos
            )
            """)

        self.cursor.execute(
            """
            create table if not exists files (
                file_id     int,
                repo_id     int,
                path        text,
                primary key (file_id),
                foreign key (repo_id) references repos
            )
            """
        )

        self.cursor.execute(
            """
            create table if not exists matches (
                match_id    int,
                file_id     int,
                line        int,
                col         int,
                vector      int,
                width       int,
                si          int,
                primary key (match_id),
                foreign key (file_id) references files
            )
            """
        )
        self.connection.commit()
