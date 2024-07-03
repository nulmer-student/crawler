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
            create table if not exists files (
                file_id     int,
                repo_id     int,
                path        text,
                primary key (file_id, repo_id),
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
                primary key (match_id, file_id),
                foreign key (file_id) references files
            )
            """
        )
        self.connection.commit()


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
