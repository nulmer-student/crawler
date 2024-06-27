#!/usr/bin/env python3

import argparse
import subprocess
import multiprocessing
from pathlib import Path
import re
import sys


def find_files(dir, extension):
    """Return a list of all files with EXTENSION in DIR."""
    pipe = subprocess.Popen(
        ["find", dir, "-name", f"*.{extension}"],
        stdout=subprocess.PIPE
    )
    files = pipe.communicate()[0].decode().splitlines()

    return [Path(f) for f in files]


class Header:
    def __init__(self, name, type="user") -> None:
        assert(type in ["user", "system"])
        self.name = name
        self.type = type

    def __repr__(self) -> str:
        return f"<Header name: {self.name}, type: {self.type}>"


def find_includes(file):
    """Return the files from #include declarations in FILE."""
    acc = []
    try:
        with open(file, "r") as infile:
            for line in infile.readlines():
                m = re.match(r"""#include\s+(["<])(.+)[">]$""", line)
                if m:
                    type = "user"
                    if m.group(1) == "<":
                        type = "system"

                    name = m.group(2)
                    acc.append(Header(name, type))
            return acc
    except Exception:
        return []


class FileDeps:
    def __init__(self, files) -> None:
        self.map = dict()
        for f in files:
            self.map[f] = find_includes(f)

    def get(self, file):
        if self.map[file]:
            return self.map[file]
        else:
            return []


    def __str__(self) -> str:
        acc = ""
        for k, v in self.map.items():
            acc += f"file: {k}, deps: {v}\n"
        return acc


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
        except Exception:
            return None

    def __str__(self) -> str:
        acc = ""
        for key, value in self.map.items():
            acc += f"name: {key}, headers: {value}\n"

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
    print(command, file=sys.stderr)
    pipe = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    error = pipe.communicate()[1]
    if error != "":
        try:
            error = error.decode()
            print(f"=== Error in: {cc_file}", file=sys.stderr)
            print(error, file=sys.stderr)
        except:
            print(f"=== Error in: {cc_file}", file=sys.stderr)
            print(error, file=sys.stderr)

    if pipe.returncode == 0:
        return True
    else:
        return False


class Work:
    def __init__(self, repo, inc_map, dep_map):
        self.repo    = repo
        self.inc_map = inc_map
        self.dep_map = dep_map

    def __call__(self, file):
        print(f"==== {file}")
        deps = self.find_deps(file)
        # return compile(file, self.repo, self.map)

    def find_deps(self, file):
        def helper(file, acc):
            if file in acc:
                return

            acc = acc | {file}
            print(file)
            print("acc", acc)
            deps = set(self.dep_map.get(file))
            deps = deps - acc
            # acc = deps | acc
            for f in deps:
                paths = self.inc_map.get(f.name)
                if paths is not None and len(paths) > 0:
                    print("here")
                    helper(paths[0], acc)

        acc = set()
        helper(file, acc)
        print(acc)


def main():
    # Get the CLI arguments
    parser = argparse.ArgumentParser()

    parser.add_argument("dir",
                        help="Directory of the repository")
    parser.add_argument("--threads",
                        help="Number of threads",
                        type=int)

    args = parser.parse_args()
    repo = args.dir

    # Find all *.c/*.h files in the repository
    cc_files = find_files(repo, "c")
    header_files = find_files(repo, "h")

    # Find the dependencies of each file
    dep_map = FileDeps(cc_files + header_files)
    # print(dep_map)

    # Map include macros to their respective files
    inc_map = IncludeMap(repo, header_files)

    print("===== Starting Compilation =====")

    # Compile each file in the repo
    with multiprocessing.Pool(args.threads) as pool:
        results = pool.map(Work(repo, inc_map, dep_map), cc_files)

        # # Count failures
        # err = 0
        # for r in results:
        #     if not r:
        #         err += 1

        # # Print the results
        # cc = len(cc_files)
        # prop = (err/cc) * 100
        # print(f"File count: {cc}")
        # print(f"Error count: {err} ({prop:0.3}%)")


if __name__ == "__main__":
    main()
