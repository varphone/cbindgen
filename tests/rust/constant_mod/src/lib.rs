mod aka;

pub(crate) const PRIVATE_FORWARD_SOURCE: u32 = 321;

pub const AKA_FROM_MODULE: u32 = aka::AKA;
pub const AKA_FROM_SELF_PATH: u32 = aka::AKA_FROM_SELF;
pub const AKA_FROM_CRATE_PATH: u32 = crate::aka::AKA;
pub const AKA_FROM_NESTED_MODULE: u32 = crate::aka::nested::AKA;
pub const AKA_FROM_SUPER_PATH: u32 = crate::aka::nested::AKA_FROM_SUPER;
pub const AKA_FROM_PRIVATE: u32 = PRIVATE_FORWARD_SOURCE;

#[repr(C)]
pub struct Foo {
    pub x: u32,
}

#[no_mangle]
pub extern "C" fn root(x: Foo) -> u32 {
    x.x
}
