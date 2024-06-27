#ifndef INCLUDE_H_
#define INCLUDE_H_

#include <cstddef>
#include <filesystem>
#include <set>

using namespace std;
namespace Miner {

enum IncludeType {User, System};

class Include {
public:
    Include(string input);
    size_t operator==(const Include &) const;

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
