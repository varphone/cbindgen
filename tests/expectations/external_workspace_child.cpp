#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct ExtType {
  uint32_t data;
};
/// Associated documentation from the dependency.
constexpr static const int32_t ExtType_EXT_ASSOC = 7;

extern "C" {

void consume_ext(ExtType _ext);

}  // extern "C"
