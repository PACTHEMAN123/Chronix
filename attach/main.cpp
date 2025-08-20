#include <iostream>

int main(int argc, const char* argv[], const char* envp[]) {

    for(auto envp_iter = envp; *envp_iter != nullptr; ++envp_iter) {
        std::cout << *envp_iter << "\n";
    }

    return 0;
}
