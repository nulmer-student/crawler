#include "Util.h"
#include "Include.h"

#include <array>
#include <cstdio>
#include <cstdlib>
#include <fcntl.h>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <stdexcept>
#include <stdio.h>
#include <unistd.h>
#include <format>
#include <sstream>
#include <string>
#include <regex>

#include <sys/wait.h>
#include <unistd.h>

using namespace std;

namespace Miner {

ProcessResult run_process(string command, string stdin) {
    const int READ  = 0;
    const int WRITE = 1;

    int infd[2]  = {0, 0};
    int outfd[2] = {0, 0};
    int errfd[2] = {0, 0};

    // Open the pipes
    int rc = pipe(infd);
    if (rc < 0)
        throw std::runtime_error("Failed to open pipe");

    rc = pipe(outfd);
    if (rc < 0) {
        close(infd[READ]);
        close(infd[WRITE]);
        throw std::runtime_error("Failed to open pipe");
    }

    rc = pipe(errfd);
    if (rc < 0) {
        close(infd[READ]);
        close(infd[WRITE]);
        close(outfd[READ]);
        close(outfd[WRITE]);
        throw std::runtime_error("Failed to open pipe");
    }

    // Increase the size of the pipes
    const int SIZE = 1048576;
    fcntl(infd[READ],   F_SETPIPE_SZ, SIZE);
    fcntl(outfd[WRITE], F_SETPIPE_SZ, SIZE);

    // Fork and run the command
    int pid = fork();

    // Parent path
    if (pid > 0) {
        // Close un-needed ends
        close(infd[READ]);
        close(outfd[WRITE]);
        close(errfd[WRITE]);

        if (write(infd[WRITE], stdin.data(), stdin.size()) < 0)
            throw std::runtime_error("Failed to write to stdin pipe");

        close(infd[WRITE]);
    }

    // Child path
    else if (pid == 0) {
        // Override std files with pipes
        dup2(infd[READ], STDIN_FILENO);
        dup2(outfd[WRITE], STDOUT_FILENO);
        dup2(errfd[WRITE], STDERR_FILENO);

        // Close un-needed ends
        close(infd[WRITE]);
        close(outfd[READ]);
        close(errfd[READ]);

        execl("/bin/bash", "bash", "-c", command.c_str(), nullptr);
        exit(EXIT_SUCCESS);
    }

    // Handle fork errors
    if (pid < 0) {
        close(infd[READ]);
        close(infd[WRITE]);
        close(outfd[READ]);
        close(outfd[WRITE]);
        close(errfd[READ]);
        close(errfd[WRITE]);
        throw std::runtime_error("Failed to fork");
    }

    // Get the results
    int status = 0;
    waitpid(pid, &status, 0);

    // Read in stdout
    std::array<char, 256> out_buf;
    int bytes = 0;

    string out = "";
    do {
        bytes = read(outfd[READ], out_buf.data(), out_buf.size());
        out.append(out_buf.data(), bytes);
    } while (bytes > 0);

    // Read in stderr
    std::array<char, 256> err_buf;
    bytes = 0;

    string err = "";
    do {
        bytes = read(errfd[READ], err_buf.data(), err_buf.size());
        err.append(err_buf.data(), bytes);
    } while (bytes > 0);

    // Set the exit code
    int code = 0;
    if (WIFEXITED(status)) {
        code = WEXITSTATUS(status);
    }

    close(outfd[READ]);
    close(errfd[READ]);
    return ProcessResult(code, out, err);
}

ProcessResult run_process(string command) {
    return run_process(command, "");
}

vector<filesystem::path> find_files(filesystem::path dir, string extension) {
    // The output may be too large to use a pipe, so we will use redirects
    filesystem::path tmp_path = std::tmpnam(nullptr);

    // Find files
    string command = format(
        "find {} -name '*.{}' > {}",
        dir.string(),
        extension,
        tmp_path.string());
    run_process(command);

    // Read in the input
    vector<filesystem::path> acc;
    ifstream tmp_file(tmp_path);
    string line;

    while (getline(tmp_file, line, '\n')) {
        acc.push_back(filesystem::path(line));
    }

    // Cleanup
    tmp_file.close();
    remove(tmp_path);

    return acc;
}

vector<Include> find_includes(filesystem::path file) {
    // Load the file
    ifstream infile(file);

    if (!infile.is_open())
        throw runtime_error("File not found");

    // Look for #include on each line
    vector<Include> acc;
    string line;
    regex pattern("#include ([\"<][^\">]+[\">])");
    smatch m;
    while (getline(infile, line)) {
        if (regex_search(line, m, pattern)) {
            acc.push_back(Include(m[1]));
        }
    }

    infile.close();
    return acc;
}

}
