#!/usr/bin/env python3

import database
import search


def main():
    # Search for repositories
    db = database.Database()
    query = search.Search(db)
    query.run()



if __name__ == "__main__":
    main()
