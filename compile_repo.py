#!/usr/bin/env python3

import argparse
import subprocess
import multiprocessing
from pathlib import Path
import re
import sys

N_THREADS = 12


class IncludeMap:
    """Store & map file includes."""
    def __init__(self, repo, header_files):
        # Store all headers file paths in a map
        self.map = dict()
        for h in header_files:
            # Cut the include paths off at the base of the repo
            dirs = list(h.relative_to(repo).parts)

            # Add keys for each possible header path
            for i in range(0, len(dirs)):
                key = Path("/".join(dirs[i:]))
                if key in self.map:
                    self.map[key].append(h)
                else:
                    self.map[key] = [h]

    def get(self, include):
        """Get all possible include files with name INCLUDE."""
        # Clean up the path
        include = include.replace("../", "")
        include = include.replace("./", "")
        include = include.replace("..", "")
        include = Path(include)

        # Get the possibilities
        try:
            return self.map[include]
        except:
            return None


def find_files(dir, extension):
    """Return a list of all files with EXTENSION in DIR."""
    pipe = subprocess.Popen(
        ["find", dir, "-name", f"*.{extension}"],
        stdout=subprocess.PIPE
    )
    files = pipe.communicate()[0].decode().splitlines()

    return [Path(f) for f in files]


def find_includes(file):
    """Return the files from #include declarations in FILE."""
    acc = []
    with open(file, "r") as infile:
        for line in infile.readlines():
            m = re.match(r"""#include\s+["<](.+)[">]$""", line)
            if m:
                acc.append(m.group(1))
    return acc


def compile(cc_file, repo, m):
    # Find the includes of the cc file
    includes = find_includes(cc_file)
    possible = [m.get(i) for i in includes]

    # Remove missing/system headers (to try)
    possible = [i for i in possible if i is not None]
    possible = list(map(lambda x: x[0], possible)) # FIXME: Other header possibilities (when ambiguous)

    # Get the compile command
    fmt_inc = list({f"-I{p.parent}" for p in possible})
    fmt_inc.append(f"-I{repo}")
    command = ["clang", "-c", cc_file] + fmt_inc + ["-emit-llvm", "-o", "-"]

    # Run the compilation
    # print(command)
    pipe = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    error = pipe.communicate()[1].decode()

    if error != "":
        print(f"=== Error in: {cc_file}", file=sys.stderr)
        print(error, file=sys.stderr)

    if pipe.returncode == 0:
        return True
    else:
        return False

class Work:
    def __init__(self, repo, map):
        self.repo = repo
        self.map  = map

    def __call__(self, file):
        return compile(file, self.repo, self.map)


def main():
    # Get the CLI arguments
    parser = argparse.ArgumentParser()
    parser.add_argument("dir", help="Directory of the repository")
    repo = parser.parse_args().dir

    # Find all *.c/*.h files in the repository
    cc_files = find_files(repo, "c")
    header_files = find_files(repo, "h")
    map = IncludeMap(repo, header_files)

    # Compile each file in the repo
    with multiprocessing.Pool(N_THREADS) as pool:
        results = pool.map(Work(repo, map), cc_files)

        # Count failures
        err = 0
        for r in results:
            if not r:
                err += 1

        # Print the results
        cc = len(cc_files)
        prop = (err/cc) * 100
        print(f"File count: {cc}")
        print(f"Error count: {err} ({prop:0.3}%)")


if __name__ == "__main__":
    main()
