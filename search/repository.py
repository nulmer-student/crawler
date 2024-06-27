#!/usr/bin/env python3

class Repo:
    def __init__(self, id, name, url, stars) -> None:
        self.id        = int(id)
        self.name      = name
        self.clone_url = url
        self.stars     = int(stars)
