#!/usr/bin/env python3

import database
import repository

import requests
import time

# Setup logging
import logging
logger = logging.getLogger(__name__)


INITIAL_MAX=10000000


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
            logger.info(f"Already seen: {id}, {name}, {clone_url}, {stars}")
            logger.info(self.cursor.fetchone())
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
            logger.info(f"Error in transaction: {e}")
            self.connection.rollback()

        return count

    def min_stars(self):
        """Return the minimum start count of all repositories."""
        self.cursor.execute("select min(stars) from repos")

        if self.cursor.rowcount == 1:
            return self.cursor.fetchone()[0]
        else:
            return INITIAL_MAX


class Search:
    def __init__(self, env):
        self.db = SearchDB()
        self.env = env

        self._max_repos = 40000
        self._per_page = 100
        self._min_stars = 500

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
            logger.info(f"Rate limiting for {diff:.2f} seconds")
            time.sleep(diff)
            return True

        # Limit not reached
        return False

    def _get_page(self, page_no, per_page, max):
        """Get the n'th page of the query."""
        # Query the most starred C repositories
        query = f"q=language:c" \
            + f"+stars:{self._min_stars}..{max}" \
            + f"&sort=stars" \
            + f"&order=desc" \
            + f"&per_page={per_page}" \
            + f"&page={page_no}"
        logger.info(query)

        # Convert to repository objects
        while True:
            api_key = self.env['GITHUB_API_KEY']
            headers = {
                "Authorization": f"Bearer {api_key}",
                "X-GitHub-Api-Version": "2022-11-28",
            }

            # Query GitHub to get a single page of results
            # FIXME: Proper url formatting
            results = requests.get(
                f"https://api.github.com/search/repositories?{query}",
                headers=headers
            )

            # Check the headers for rate limiting
            limit = self._rate_limit(results.headers)
            if limit:
                continue

            # Parse the data
            parsed = results.json()
            if "items" in parsed:
                repos = [repository.Repo(data) for data in parsed["items"]]
                return (repos, False)
            else:
                return ([], True)

    def run(self):
        found = 0       # Number we have found
        page_no = 1     # Current page number

        max_stars = INITIAL_MAX

        # If there are already, results, set the max
        old_min = self.db.min_stars()
        if old_min:
            max_stars = old_min

        while found < self._max_repos:
            # Get the next page & insert into the database
            page_size = min(self._per_page, self._max_repos - found)
            repos, need_new = self._get_page(page_no, page_size, max_stars)

            # If there are results, insert them
            if not need_new:
                new = self.db.insert_repos(repos)

                # Decide if we should continue fetching
                found += new
                page_no += 1

                logger.info(f"Found '{found}' repositories")

            # If we have used the search size, create new upper bound
            else:
                max_stars = self.db.min_stars()
                page_no = 1
                logger.info(f"max : {max_stars}")
                time.sleep(5)

            # Exit if we get to the min
            if max_stars <= self._min_stars:
                break
