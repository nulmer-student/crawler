#!/usr/bin/env python3

import repository

import requests
import os
import time

class Search():
    def __init__(self, db):
        self.db = db
        self._max_repos = 10000

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
        query = f"q=language:c+language:cpp&sort=stars&order=desc&per_page={per_page}&page={page_no}"

        # Convert to repository objects
        while True:
            # Query GitHub to get a single page of results
            # FIXME: Proper url formatting
            results = requests.get(f"https://api.github.com/search/repositories?{query}")

            # Check the headers for rate limiting
            headers = results.headers
            if "retry-after" in headers:
                time.sleep(int(headers["retry-after"]))
            if int(headers["x-ratelimit-remaining"]) == 0:
                diff = int(headers["x-ratelimit-reset"]) - time.time() + 1
                print(f"rate limiting for {diff} seconds")
                time.sleep(diff)
                continue

            # Parse the data
            parsed = results.json()
            if "items" in parsed:
                repos = [self._create_repo(data) for data in parsed["items"]]
                incomplete = parsed["incomplete_results"]
                return (repos, incomplete)
            else:
                time.sleep(5)

    def run(self):
        found = 0       # Number we have found
        page_no = 0     # Current page number

        while found < self._max_repos:
            # Get the next page & insert into the database
            page_size = min(60, self._max_repos - found)
            repos, incomplete = self._get_page(page_no, page_size)
            self.db.insert_repos(repos)

            # Decide if we should continue fetching
            found += len(repos)
            page_no += 1

            # Exit if there are no more results
            # if not incomplete:
            #     break
