#!/usr/bin/env python3

import os
from dotenv import load_dotenv

def load_env():
    env_file = ".env"
    if os.path.exists(env_file):
        load_dotenv(env_file)

    acc = dict()
    acc["CLANG"]          = os.getenv("CLANG")
    acc["GITHUB_API_KEY"] = os.getenv("GITHUB_API_KEY")
    acc["LOG_DIR"]        = os.getenv("LOG_DIR")
    acc["DB_PATH"]        = os.getenv("DB_PATH")
    acc["N_THREADS"]      = os.getenv("N_THREADS")
    return acc
