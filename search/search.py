#!/usr/bin/env python3

import repository

import requests
import os

class Search():
    def __init__(self, db):
        self.db = db
        self._max_repos = 1000

    def _create_repo(self, data):
        return repository.Repo(
            data["id"],
            data["full_name"],
            data["clone_url"],
            data["stargazers_count"]
        )

    def _get_page(self, page_no, per_page):
        """Get the n'th page of the query."""
        # Query the most starred C/C++ repositories
        query = f"q=language:c&sort=stars&order=desc&per_page={per_page}&page={page_no}"

        # Query GitHub to get a single page of results
        # FIXME: Proper url formatting
        results = requests.get(f"https://api.github.com/search/repositories?{query}")
        results = results.json()

        # Convert to repository objects
        repos = [self._create_repo(data) for data in results["items"]]
        incomplete = results["incomplete_results"]
        return (repos, incomplete)

    def run(self):
        found = 0       # Number we have found
        page_no = 1     # Current page number

        while found < self._max_repos:
            # Get the next page & insert into the database
            page_size = min(30, self._max_repos - found)
            repos, incomplete = self._get_page(page_no, page_size)
            self.db.insert_repos(repos)

            # Decide if we should continue fetching
            found += len(repos)
            page_no += 1

            # Exit if there are no more results
            if not incomplete:
                break
