#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define FOO 10

#define AKA_FROM_MODULE 123

#define AKA_FROM_SELF_PATH 123

#define AKA_FROM_CRATE_PATH 123

#define AKA_FROM_NESTED_MODULE 456

#define AKA_FROM_SUPER_PATH 123

#define AKA_FROM_PRIVATE 321

#define BAR "hello world"

#define CSTR "hello world"

#define DELIMITER ':'

#define LEFTCURLY '{'

#define QUOTE '\''

#define TAB '\t'

#define NEWLINE '\n'

#define HEART U'\U00002764'

#define EQUID U'\U00010083'

#define ZOM 3.14

/**
 * A single-line doc comment.
 */
#define POS_ONE 1

/**
 * A
 * multi-line
 * doc
 * comment.
 */
#define NEG_ONE -1

#define SHIFT 3

#define XBOOL 1

#define XFALSE ((0 << SHIFT) | XBOOL)

#define XTRUE (1 << (SHIFT | XBOOL))

#define CAST (uint8_t)'A'

#define DOUBLE_CAST (uint32_t)(float)1

typedef struct {
  int32_t x[FOO];
} Foo;

void root(Foo x);
