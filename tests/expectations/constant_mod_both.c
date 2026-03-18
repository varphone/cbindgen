#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define AKA_FROM_MODULE 123

#define AKA_FROM_SELF_PATH 123

#define AKA_FROM_CRATE_PATH 123

#define AKA_FROM_NESTED_MODULE 456

#define AKA_FROM_SUPER_PATH 123

#define AKA_FROM_PRIVATE 321

typedef struct Foo {
  uint32_t x;
} Foo;

uint32_t root(struct Foo x);
