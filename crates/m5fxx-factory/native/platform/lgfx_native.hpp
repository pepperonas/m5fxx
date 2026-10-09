#pragma once
#include <cstdlib>
#include <cstdio>
#include <cstdint>
#include <lgfx/v1/misc/DataWrapper.hpp>
#include <lgfx/v1/misc/enum.hpp>
#include <lgfx/utility/result.hpp>
namespace lgfx { inline namespace v1 {
unsigned long millis(); unsigned long micros(); void delay(unsigned long); void delayMicroseconds(unsigned int);
inline void* heap_alloc(size_t n){return malloc(n);} inline void* heap_alloc_psram(size_t n){return malloc(n);} inline void* heap_alloc_dma(size_t n){return malloc(n);} inline void heap_free(void* p){free(p);} inline bool heap_capable_dma(const void*){return false;}
inline void gpio_hi(uint32_t){} inline void gpio_lo(uint32_t){} inline bool gpio_in(uint32_t){return false;}
enum pin_mode_t {output,input,input_pullup,input_pulldown};inline void pinMode(int_fast16_t,pin_mode_t){}inline void lgfxPinMode(int_fast16_t,pin_mode_t){}
struct FileWrapper:DataWrapper {FILE* fp=nullptr; FileWrapper(){need_transaction=false;} bool open(const char* p)override{return (fp=fopen(p,"rb"));}int read(uint8_t* b,uint32_t n)override{return fread(b,1,n,fp);}void skip(int32_t n)override{fseek(fp,n,SEEK_CUR);}bool seek(uint32_t n)override{return fseek(fp,n,SEEK_SET)==0;}void close()override{if(fp){fclose(fp);fp=nullptr;}}int32_t tell()override{return ftell(fp);}};
namespace spi {inline cpp::result<void,error_t> init(int,int,int,int,int){return {};}inline void beginTransaction(int){}}
namespace i2c {inline cpp::result<void,error_t> setPins(int,int,int){return {};}inline cpp::result<void,error_t> init(int){return {};}}
}}
