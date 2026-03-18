#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct Foo {

};
constexpr static const int32_t Foo_GA = 10;
#define Foo_BU "hello world"
constexpr static const float Foo_ZO = 3.14;

extern "C" {

void root(Foo x);

}  // extern "C"
