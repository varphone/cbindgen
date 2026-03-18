#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

struct ExtType {
  uint32_t data;
};
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
