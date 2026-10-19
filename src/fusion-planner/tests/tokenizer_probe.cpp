#include <string>
#include <vector>
#include <iostream>
namespace MNN { namespace Transformer { class Llm { public: std::vector<int> tokenizer_encode(const std::string&); std::string tokenizer_decode(int); }; }}
extern "C" void* ahem_llm_create(const char*);
extern "C" void ahem_llm_destroy(void*);
int main(int argc,char**argv){if(argc!=2)return 2;auto h=ahem_llm_create(argv[1]);if(!h)return 3;auto p=static_cast<MNN::Transformer::Llm*>(h);auto prefix=p->tokenizer_encode("{\"choice\":\"");for(char c='A';c<='H';++c){auto ids=p->tokenizer_encode(std::string(1,c));auto both=p->tokenizer_encode(std::string("{\"choice\":\"")+c);auto empty=p->tokenizer_encode("");if(ids.size()!=empty.size()+1 || p->tokenizer_decode(ids.back())!=std::string(1,c)){std::cerr<<"letter tokenizer failure "<<c<<"\n";return 4;}std::cout<<c<<"="<<ids.back()<<"\n";}ahem_llm_destroy(h);}
