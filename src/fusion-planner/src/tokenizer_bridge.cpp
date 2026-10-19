#include <string>
#include <vector>
// The exact pinned Itanium C++ ABI is checked by dynamic lookup. Returning
// std::vector stays entirely within C++, never across the Rust ABI.
extern "C" int planner_token_count(void* encode,void* handle,const char* text) {
 try {using Fn=std::vector<int>(*)(void*,const std::string&);
 return static_cast<int>(reinterpret_cast<Fn>(encode)(handle,std::string(text)).size());
 } catch (...) {return -1;}
}

// Mirror ahem_llm_create's synchronous, uncached lifecycle, but select greedy
// BEFORE load constructs the sampler. A post-load set_config is insufficient:
// Sampler copies its configuration at construction. Model files stay untouched.
extern "C" void* planner_create_greedy(void* create, void* configure, void* load,
                                      void (*destroy)(void*), const char* path) {
    void* handle = nullptr;
    try {
        using Create = void* (*)(const std::string&);
        using Configure = bool (*)(void*, const std::string&);
        using Load = bool (*)(void*);
        handle = reinterpret_cast<Create>(create)(std::string(path));
        if (!handle) return nullptr;
        const std::string settings = R"({"tmp_path":"tmp","async":false,"prompt_cache":false,"reuse_kv":false,"sampler_type":"greedy"})";
        if (!reinterpret_cast<Configure>(configure)(handle, settings) ||
            !reinterpret_cast<Load>(load)(handle)) {
            destroy(handle);
            return nullptr;
        }
        if (!reinterpret_cast<Configure>(configure)(handle,
                R"({"async":false,"prompt_cache":false,"reuse_kv":false})")) {
            destroy(handle);
            return nullptr;
        }
        return handle;
    } catch (...) {
        if (handle) destroy(handle);
        return nullptr;
    }
}
