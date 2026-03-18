extern crate workspace_dep as dep_crate;

use dep_crate as dep;
use dep::ExtType as Ty;
use Ty as NestedTy;

pub const AKA: i32 = dep::EXT_CONST;
pub const AKA2: i32 = NestedTy::EXT_ASSOC;

/// Local documentation overrides dependency documentation.
pub const AKA3: i32 = dep::EXT_CONST;

#[no_mangle]
pub extern "C" fn consume_ext(_ext: dep::ExtType) {
}
