from libc.stdint cimport int8_t, int16_t, int32_t, int64_t, intptr_t
from libc.stdint cimport uint8_t, uint16_t, uint32_t, uint64_t, uintptr_t
cdef extern from *:
  ctypedef bint bool
  ctypedef struct va_list

cdef extern from *:

  # Dependency documentation for AKA.
  const int32_t AKA # = 11

  # Associated documentation from the dependency.
  const int32_t AKA2 # = 7

  # Local documentation overrides dependency documentation.
  const int32_t AKA3 # = 0

  # Dependency documentation for EXT_CONST.
  const int32_t AKA4 # = 0

  # Dependency documentation for EXT_CONST.
  const int32_t EXT_CONST # = 0

  cdef struct ExtType:
    uint32_t data;
  # Associated documentation from the dependency.
  const int32_t ExtType_EXT_ASSOC # = 7

  void consume_ext(ExtType _ext);
