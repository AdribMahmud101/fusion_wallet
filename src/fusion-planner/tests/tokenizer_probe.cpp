#include <dlfcn.h>
#include <iostream>
#include <regex>
#include <string>
#include <vector>

namespace MNN { namespace Transformer {
class Llm {
public:
    std::vector<int> tokenizer_encode(const std::string&);
    std::string tokenizer_decode(int);
    std::string dump_config();
};
}}
extern "C" void ahem_llm_destroy(void*);
extern "C" void* planner_create_greedy(void*, void*, void*, void (*)(void*), const char*);

int main(int argc, char** argv) {
    if (argc != 2) return 2;
    auto create = dlsym(RTLD_DEFAULT, "_ZN3MNN11Transformer3Llm9createLLMERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE");
    auto configure = dlsym(RTLD_DEFAULT, "_ZN3MNN11Transformer3Llm10set_configERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE");
    auto load = dlsym(RTLD_DEFAULT, "_ZN3MNN11Transformer3Llm4loadEv");
    if (!create || !configure || !load) return 3;
    auto handle = planner_create_greedy(create, configure, load, ahem_llm_destroy, argv[1]);
    if (!handle) return 4;
    auto model = static_cast<MNN::Transformer::Llm*>(handle);
    if (!std::regex_search(model->dump_config(),
            std::regex("\"sampler_type\"\\s*:\\s*\"greedy\""))) {
        std::cerr << "effective sampler is not greedy\n";
        ahem_llm_destroy(handle);
        return 5;
    }
    std::cout << "effective sampler_type=greedy\n";
    auto prefix = model->tokenizer_encode("{\"choice\":\"");
    auto empty = model->tokenizer_encode("");
    for (char letter = 'A'; letter <= 'H'; ++letter) {
        auto ids = model->tokenizer_encode(std::string(1, letter));
        std::string continuation;
        for (size_t i = empty.size(); i < prefix.size(); ++i)
            continuation += model->tokenizer_decode(prefix[i]);
        continuation += model->tokenizer_decode(ids.back());
        if (ids.size() != empty.size() + 1 ||
            model->tokenizer_decode(ids.back()) != std::string(1, letter) ||
            continuation != std::string("{\"choice\":\"") + letter) {
            std::cerr << "letter continuation failure " << letter << "\n";
            ahem_llm_destroy(handle);
            return 6;
        }
        std::cout << letter << "=" << ids.back() << "\n";
    }
    ahem_llm_destroy(handle);
}
