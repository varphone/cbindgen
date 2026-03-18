#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Dependency documentation for EXT_CONST.
constexpr static const int32_t AKA = 0;

/// Associated documentation from the dependency.
constexpr static const int32_t AKA2 = 7;

/// Local documentation overrides dependency documentation.
constexpr static const int32_t AKA3 = 0;

/// Dependency documentation for EXT_CONST.
constexpr static const int32_t EXT_CONST = 0;

struct ExtType {
  uint32_t data;
};
/// Associated documentation from the dependency.
constexpr static const int32_t ExtType_EXT_ASSOC = 7;

extern "C" {

void consume_ext(ExtType _ext);

}  // extern "C"
