#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Dependency documentation for re-exported OKAY.
constexpr static const int32_t OKAY = 0;

/// Dependency documentation for re-exported OKAY.
constexpr static const int32_t ALRIGHT = 0;

/// Dependency documentation for re-exported OKAY.
constexpr static const int32_t GROUP_OKAY = 0;

/// Dependency documentation for re-exported OKAY.
constexpr static const int32_t SELF_OKAY = 0;

/// Dependency documentation for glob-re-exported WILDCARD_OKAY.
constexpr static const int32_t WILDCARD_OKAY = 1;

/// Dependency documentation for deeply re-exported DEEP_OKAY.
constexpr static const int32_t DEEP_GROUP_OKAY = 2;

/// Dependency documentation for AKA.
constexpr static const int32_t AKA = 11;

/// Associated documentation from the dependency.
constexpr static const int32_t AKA2 = 7;

/// Local documentation overrides dependency documentation.
constexpr static const int32_t AKA3 = 0;

/// Dependency documentation for EXT_CONST.
constexpr static const int32_t AKA4 = 0;

/// Dependency documentation for EXT_CONST.
constexpr static const int32_t EXT_CONST = 0;

/// Dependency documentation for deeply re-exported DEEP_OKAY.
constexpr static const int32_t DEEP_OKAY = 2;

struct ExtType {
  uint32_t data;
};
/// Associated documentation from the dependency.
constexpr static const int32_t ExtType_EXT_ASSOC = 7;

extern "C" {

void consume_ext(ExtType _ext);

}  // extern "C"
