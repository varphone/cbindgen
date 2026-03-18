pub mod error;
pub use error::*;
pub use error::OKAY;
pub use error::{OKAY as ALRIGHT, OKAY as GROUP_OKAY};
pub use error::{nested::DEEP_OKAY as DEEP_GROUP_OKAY};
pub use self::error::OKAY as SELF_OKAY;

#[repr(C)]
pub struct ExtType {
    pub data: u32,
}

impl ExtType {
    /// Associated documentation from the dependency.
    pub const EXT_ASSOC: i32 = 7;
}

/// Dependency documentation for EXT_CONST.
pub const EXT_CONST: i32 = 0;

/// Dependency documentation for AKA.
pub const AKA: i32 = 11;
