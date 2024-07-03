#!/usr/bin/env python3

class Repo:
    def __init__(self, data):
        self.data = data

    def get(self, attribute):
        if attribute in self.data:
            return self.data[attribute]
        else:
            return None
