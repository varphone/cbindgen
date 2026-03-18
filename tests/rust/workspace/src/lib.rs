extern crate workspace_dep as dep_crate;

use dep_crate as dep;
use dep::ExtType as Ty;
use Ty as NestedTy;

pub const OKAY: i32 = workspace_dep::OKAY;
pub const ALRIGHT: i32 = workspace_dep::ALRIGHT;
pub const GROUP_OKAY: i32 = workspace_dep::GROUP_OKAY;
pub const SELF_OKAY: i32 = workspace_dep::SELF_OKAY;
pub const WILDCARD_OKAY: i32 = workspace_dep::WILDCARD_OKAY;
pub const DEEP_GROUP_OKAY: i32 = workspace_dep::DEEP_GROUP_OKAY;
pub const AKA: i32 = workspace_dep::AKA;
pub const AKA2: i32 = NestedTy::EXT_ASSOC;

/// Local documentation overrides dependency documentation.
pub const AKA3: i32 = dep::EXT_CONST;
pub const AKA4: i32 = workspace_dep::EXT_CONST;

#[no_mangle]
pub extern "C" fn consume_ext(_ext: dep::ExtType) {
}
