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
