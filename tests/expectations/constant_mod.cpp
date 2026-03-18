#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

constexpr static const uint32_t AKA_FROM_MODULE = 123;

constexpr static const uint32_t AKA_FROM_SELF_PATH = 123;

constexpr static const uint32_t AKA_FROM_CRATE_PATH = 123;

constexpr static const uint32_t AKA_FROM_NESTED_MODULE = 456;

constexpr static const uint32_t AKA_FROM_SUPER_PATH = 123;

constexpr static const uint32_t AKA_FROM_PRIVATE = 321;

struct Foo {
  uint32_t x;
};

extern "C" {

uint32_t root(Foo x);

}  // extern "C"
