#!/usr/bin/env python3

class Repo:
    def __init__(self, data):
        self.data = data

    def get(self, attribute):
        if attribute in self.data:
            return self.data[attribute]
        else:
            return None


class DBRepo:
    def __init__(self, id, name, url, stars):
        self.id    = id
        self.name  = name
        self.url   = url
        self.stars = stars

    def __str__(self):
        return f"<{DBRepo.__name__} id: {self.id}, name: {self.name}, url: {self.url}, stars: {self.stars}>"
