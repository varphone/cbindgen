#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Dependency documentation for re-exported OKAY.
 */
#define OKAY 0

/**
 * Dependency documentation for re-exported OKAY.
 */
#define ALRIGHT 0

/**
 * Dependency documentation for re-exported OKAY.
 */
#define GROUP_OKAY 0

/**
 * Dependency documentation for re-exported OKAY.
 */
#define SELF_OKAY 0

/**
 * Dependency documentation for glob-re-exported WILDCARD_OKAY.
 */
#define WILDCARD_OKAY 1

/**
 * Dependency documentation for deeply re-exported DEEP_OKAY.
 */
#define DEEP_GROUP_OKAY 2

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

/**
 * Dependency documentation for deeply re-exported DEEP_OKAY.
 */
#define DEEP_OKAY 2

typedef struct {
  uint32_t data;
} ExtType;
/**
 * Associated documentation from the dependency.
 */
#define ExtType_EXT_ASSOC 7

void consume_ext(ExtType _ext);
