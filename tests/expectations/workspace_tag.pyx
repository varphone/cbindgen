from libc.stdint cimport int8_t, int16_t, int32_t, int64_t, intptr_t
from libc.stdint cimport uint8_t, uint16_t, uint32_t, uint64_t, uintptr_t
cdef extern from *:
  ctypedef bint bool
  ctypedef struct va_list

cdef extern from *:

  # Dependency documentation for re-exported OKAY.
  const int32_t OKAY # = 0

  # Dependency documentation for re-exported OKAY.
  const int32_t ALRIGHT # = 0

  # Dependency documentation for re-exported OKAY.
  const int32_t GROUP_OKAY # = 0

  # Dependency documentation for re-exported OKAY.
  const int32_t SELF_OKAY # = 0

  # Dependency documentation for glob-re-exported WILDCARD_OKAY.
  const int32_t WILDCARD_OKAY # = 1

  # Dependency documentation for deeply re-exported DEEP_OKAY.
  const int32_t DEEP_GROUP_OKAY # = 2

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

  # Dependency documentation for deeply re-exported DEEP_OKAY.
  const int32_t DEEP_OKAY # = 2

  cdef struct ExtType:
    uint32_t data;
  # Associated documentation from the dependency.
  const int32_t ExtType_EXT_ASSOC # = 7

  void consume_ext(ExtType _ext);
