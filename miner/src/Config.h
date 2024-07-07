#ifndef CRAWLER_CONFIG_H_
#define CRAWLER_CONFIG_H_

#include <argp.h>
#include <bits/types/error_t.h>
#include <string>
#include <filesystem>

// Number of required arguments
#define N_REQUIRED 2

using namespace std;

namespace Miner {

static const char *argp_program_version = "0.1.0";
static char doc[] = "Using CLANG, search for vectorization opportunities in REPO.";

// Required options
static char args_doc[] = "CLANG REPO";

// Switch options
static struct argp_option options[] = {
    {"threads",   't',    "N", 0, "Number of threads to use"},
    {"log",       'l', "FILE", 0, "Path to the log"},
    {"max-tries", 'm',    "N", 0, "Maximum number of tries to compile a given file"},
    { 0 }
};

struct arguments {
    char *args[N_REQUIRED];
    int threads;
    int max_tries;
    filesystem::path log;
};

static error_t parse_opt(int key, char *arg, struct argp_state *state) {
    struct arguments *arguments = static_cast<struct arguments*>(state->input);

    switch (key) {
        case 't':
            arguments->threads = stoi(arg);
            break;
        case 'l':
            arguments->log = filesystem::path(arg);
            break;
        case 'm':
            arguments->max_tries = stoi(arg);
            break;
        case ARGP_KEY_ARG:
            if (state->arg_num >= N_REQUIRED)
                argp_usage(state);
            arguments->args[state->arg_num] = arg;
            break;
        case ARGP_KEY_END:
            if (state->arg_num < N_REQUIRED)
                argp_usage(state);
            break;
        default:
            return ARGP_ERR_UNKNOWN;
    }

    return 0;
};

static struct argp arg_p = { options, parse_opt, args_doc, doc };

}

#endif // CONFIG_H_
