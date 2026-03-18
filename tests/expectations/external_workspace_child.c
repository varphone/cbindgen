#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct {
  uint32_t data;
} ExtType;
/**
 * Associated documentation from the dependency.
 */
#define ExtType_EXT_ASSOC 7

void consume_ext(ExtType _ext);
