#include <string>
#include <vector>
// The exact pinned Itanium C++ ABI is checked by dynamic lookup. Returning
// std::vector stays entirely within C++, never across the Rust ABI.
extern "C" int planner_token_count(void* encode,void* handle,const char* text) {
 try {using Fn=std::vector<int>(*)(void*,const std::string&);
 return static_cast<int>(reinterpret_cast<Fn>(encode)(handle,std::string(text)).size());
 } catch (...) {return -1;}
}
