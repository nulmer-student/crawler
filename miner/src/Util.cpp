#include "Util.h"
#include "Include.h"

#include <cstdio>
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

ProcessResult test_run(string command, string stdin) {
    // FIXME: Use pipes instead of temp files

    // Create temp files for stdout / stderr
    filesystem::path out_path = std::tmpnam(nullptr);
    filesystem::path err_path = std::tmpnam(nullptr);

    // Modify the command to use redirects
    string mod_command = format(
        "{} > '{}' 2> '{}'",
        command,
        out_path.string(),
        err_path.string());

    // Run the command
    FILE *fp = popen(mod_command.c_str(), "w");
    if (fp == nullptr)
        throw std::runtime_error("Command failure");

    // Write to stdin
    for (char &c : stdin) {
        fputc(c, fp);
    }

    // Read in stdout
    cout << out_path << "\n";
    ifstream out_file(out_path);
    if (!out_file.is_open())
        throw runtime_error("Failed to open stdout file");

    string out; string line;
    while (getline(out_file, line)) {
        out += line;
        out += "\n";
    }

    out_file.close();
    remove(out_path);

    return ProcessResult(0, out, "");
}

ProcessResult run_process(string command) {
    // FIXME: Capture stderr in a non-stupid way
    filesystem::path tmp_path = std::tmpnam(nullptr);

    // Append stderr redirect
    command += format(" 2> '{}'", tmp_path.string());

    // Run the command
    FILE *fp = popen(command.c_str(), "r");
    if (fp == nullptr)
        throw std::runtime_error("Command failure");

    // Read in stdout
    int c; string acc;
    while((c = fgetc(fp)) != EOF)
        acc += c;

    // Read in stderr
    ifstream err_file(tmp_path);
    if (!err_file.is_open())
        throw runtime_error("Tmp file not found");

    string err; string line;
    while (getline(err_file, line)) {
        err += line;
        err += "\n";    // geline removes newlines
    }

    err_file.close();
    remove(tmp_path);

    int code = WEXITSTATUS(pclose(fp));
    return ProcessResult(code, acc, err);
}

vector<filesystem::path> find_files(filesystem::path dir, string extension) {
    // Find files
    string command = format("find {} -name '*.{}'", dir.string(), extension);
    string output = run_process(command).stdout;

    // Convert to a vector of strings
    string line;
    vector<filesystem::path> acc;
    stringstream stream(output);

    while (getline(stream, line, '\n')) {
        acc.push_back(filesystem::path(line));
    }

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
