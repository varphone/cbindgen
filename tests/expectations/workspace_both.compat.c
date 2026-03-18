#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Dependency documentation for EXT_CONST.
 */
#define AKA 0

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
#define EXT_CONST 0

typedef struct ExtType {
  uint32_t data;
} ExtType;
/**
 * Associated documentation from the dependency.
 */
#define ExtType_EXT_ASSOC 7

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void consume_ext(struct ExtType _ext);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
