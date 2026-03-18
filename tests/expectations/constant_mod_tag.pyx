from libc.stdint cimport int8_t, int16_t, int32_t, int64_t, intptr_t
from libc.stdint cimport uint8_t, uint16_t, uint32_t, uint64_t, uintptr_t
cdef extern from *:
  ctypedef bint bool
  ctypedef struct va_list

cdef extern from *:

  const uint32_t AKA_FROM_MODULE # = 123

  const uint32_t AKA_FROM_SELF_PATH # = 123

  const uint32_t AKA_FROM_CRATE_PATH # = 123

  const uint32_t AKA_FROM_NESTED_MODULE # = 456

  const uint32_t AKA_FROM_SUPER_PATH # = 123

  const uint32_t AKA_FROM_PRIVATE # = 321

  cdef struct Foo:
    uint32_t x;

  uint32_t root(Foo x);
