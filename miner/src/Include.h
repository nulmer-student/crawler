#ifndef INCLUDE_H_
#define INCLUDE_H_

#include <filesystem>

using namespace std;
namespace Miner {

enum IncludeType {User, System};

class Include {
public:
    Include(string input);

    IncludeType type;
    filesystem::path path;
};

struct IncludeHash {
    size_t operator()(const Include &k) const {
        return hash<string>{}(k.path.string());
    }
};

struct IncludeEq {
    size_t operator()(const Include &a, const Include &b) const {
        return a.path == b.path && a.type == b.type;
    }
};

}

#endif // INCLUDE_H_
