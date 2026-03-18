#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Dependency documentation for AKA.
 */
#define AKA 11

/**
 * Associated documentation from the dependency.
 */
#define AKA2 7

/**
 * Local documentation overrides dependency documentation.
 */
#define AKA3 0

/**
 * Dependency documentation for EXT_CONST.
 */
#define AKA4 0

/**
 * Dependency documentation for EXT_CONST.
 */
#define EXT_CONST 0

typedef struct {
  uint32_t data;
} ExtType;
/**
 * Associated documentation from the dependency.
 */
#define ExtType_EXT_ASSOC 7

void consume_ext(ExtType _ext);
