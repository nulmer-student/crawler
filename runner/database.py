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
    pass
