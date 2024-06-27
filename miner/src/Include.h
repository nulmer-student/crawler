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

}

#endif // INCLUDE_H_
