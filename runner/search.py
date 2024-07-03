#!/usr/bin/env python3

import database
import repository

import requests
import time

class SearchDB(database.Database):
    def _insert_repo(self, repo):
        """Insert a repository into the database."""
        id        = repo.get("id")
        name      = repo.get("full_name")
        clone_url = repo.get("clone_url")
        stars     = repo.get("stargazers_count")


        # Check if we already have this repo
        self.cursor.execute(
            "select * from repos where repo_id = ?",
            (id,)
        )

        # Only insert if the repo is not present
        if self.cursor.rowcount == 0:
            self.cursor.execute(
                "insert into repos values (?, ?, ?, ?)",
                (id, name, clone_url, stars)
            )
            return True
        else:
            print(f"Already seen: {id}, {name}, {clone_url}, {stars}")
            print(self.cursor.fetchone())
            return False

    def insert_repos(self, repo_list):
        """Insert a list of repositories into the database."""
        count = 0
        try:
            # Attempt to insert each repo
            for repo in repo_list:
                result = self._insert_repo(repo)
                if result:
                    count += 1

            self.connection.commit()

        except Exception as e:
            print(f"Error in transaction: {e}")
            self.connection.rollback()

        return count


class Search:
    def __init__(self):
        self.db = SearchDB()
        self._max_repos = 10

    def _rate_limit(self, headers):
        """Rate limit based on HEADERS & return true if there was rate limiting."""
        diff = 0

        # Parse the headers
        if "retry-after" in headers:
            diff = int(headers["retry-after"])
        if int(headers["x-ratelimit-remaining"]) == 0:
            diff = int(headers["x-ratelimit-reset"]) - time.time() + 1

        # Sleep if required
        if diff != 0:
            print(f"Rate limiting for {diff:.2f} seconds")
            time.sleep(diff)
            return True

        # Limit not reached
        return False

    def _get_page(self, page_no, per_page):
        """Get the n'th page of the query."""
        # Query the most starred C repositories
        query = f"q=language:c&sort=stars&order=desc&per_page={per_page}&page={page_no}"

        # Convert to repository objects
        while True:
            # Query GitHub to get a single page of results
            # FIXME: Proper url formatting
            results = requests.get(f"https://api.github.com/search/repositories?{query}")

            # Check the headers for rate limiting
            limit = self._rate_limit(results.headers)
            if limit:
                continue

            # Parse the data
            parsed = results.json()
            if "items" in parsed:
                repos = [repository.Repo(data) for data in parsed["items"]]
                incomplete = parsed["incomplete_results"]
                return (repos, incomplete)
            else:
                print(f"Items are missing, waiting")
                time.sleep(5)

    def run(self):
        found = 0       # Number we have found
        page_no = 0     # Current page number

        while found < self._max_repos:
            # Get the next page & insert into the database
            page_size = min(60, self._max_repos - found)
            repos, incomplete = self._get_page(page_no, page_size)
            new = self.db.insert_repos(repos)

            # Decide if we should continue fetching
            found += new
            page_no += 1

            print(f"Found '{found}' repositories")

            # Exit if there are no more results
            # if not incomplete:
            #     break
