#!/usr/bin/env python3

import os
from dotenv import load_dotenv

def load_env():
    load_dotenv(".env")

    acc = dict()
    acc["CLANG"]          = os.getenv("CLANG")
    acc["GITHUB_API_KEY"] = os.getenv("GITHUB_API_KEY")
    acc["LOG_DIR"]        = os.getenv("LOG_DIR")
    return acc
