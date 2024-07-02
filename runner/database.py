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
        pass


class InternDB(Database):
    def _init_db(self):
        super()._init_db()

        # Add tables
        self.cursor.execute("drop table if exists files")
        self.cursor.execute("drop table if exists matches")

        self.cursor.execute(
            """
            create table files (
                file_id     int,
                path        text,
                primary key (file_id)
            )
            """
        )

        self.cursor.execute(
            """
            create table matches (
                file_id     int,
                line        int,
                col         int,
                vector      int,
                width       int,
                si          int,
                primary key (file_id, line, col)
            )
            """
        )

    def add_match(self, path, line, col, vector, tile, si):
        # Ensure that there is an entry in the file table
        id = self._ensure_file(path)

        # Insert the match
        self.cursor.execute(
            "insert into matches values (?, ?, ?, ?, ?, ?)",
            (id, line, col, vector, tile, si)
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
            return self.cursor.fetchone()

        # Otherwise, insert the file
        if self.cursor.rowcount == 0:
            id = self._new_file_id()[0]
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
        return id
